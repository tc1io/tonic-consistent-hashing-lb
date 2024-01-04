use tonic::transport::Channel;
use std::collections::BTreeMap;
use std::fmt::Debug;
use fasthash::murmur3;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};

#[derive(Debug)]
pub struct StaticSetConsitentHashingLBClient<T> {
    /// BtreeMap<hashing_key, T refers grpc client channel>
    clients: BTreeMap<u32, T>,
    /// Murmur hash function that return 32-bit unsigned integer for a byte array
    hasher: fn(&[u8]) -> u32,
}

/// Impl this trait on the grpc request struct to fetch the key from request which would be required to find next client in ring
pub trait ConsistentHashingTrait {
    fn get_key(&self) -> String;
}

/// Impl this trait on grpc_client<channel> to create new channel from balance list, which will be inserted in to the ring
pub trait NewFromChannel {
    fn new(channel: Channel) -> Self;
}

/// Based on fasthash crate (https://docs.rs/fasthash/latest/fasthash/murmur3/struct.Hash32.html)
/// TODO: To be investigated and followed up if there any replacement one, as murmur3 had issues with new ARM MAC's (https://docs.rs/fasthash/latest/fasthash/murmur3/struct.Hash32.html)
fn create_hash(val: &[u8]) -> u32 {
    murmur3::hash32(val)
}

impl<T: NewFromChannel> StaticSetConsitentHashingLBClient<T> {
    /// Construct new StaticConsistentHashingLBClient with empty btree and murmur3 hash function
    pub async fn new() -> Self {
        StaticSetConsitentHashingLBClient {
            hasher: create_hash,
            clients: BTreeMap::new(),
        }
    }

    /// Add the static endpoint uri's to ring with chunks of 2 from tonic balance list to insert a new balanced client to the ring
    /// Add the virtual node size as replica's to create virtual upstreams for more availability (Minimum 1)
    /// Ring will be constructed as [(No of Static Endpoints/Chunk size) * Virtual node size]
    /// For Example: No of static endpoints - 6, Virtual node size - 3, Chunk size - 2. So the ring size would be [(6/2)*3] 9 for the mentioned example
    /// Initial 'key' for ring is inserted based on random i32 number generated between provided values in uniform sample distribution
    pub async fn add(&mut self, uris: &'static [&'static str], virtual_node_size: usize) {
        //let mut s = Self { clients: BTreeMap::new() };

        let mut rng = thread_rng();
        let side = Uniform::new(1, 99999);
        for node_id in 0..virtual_node_size {
            for u in uris.chunks(2) {
                let rand = rng.sample(side);
                let k = format!("{}_{}", rand, node_id); // TODO:check if required to format and generate key, (its possible to generate with i and node_id without rand as well)

                let key = (self.hasher)(k.as_bytes());

                let endpoint = u.iter().map(|e| Channel::from_static(e));
                let channel = Channel::balance_list(endpoint);
                let client = T::new(channel);
                self.clients.insert(key, client);
            }
        }
    }

    /// Find next balanced client in ring based on provided key from request, return option of <&Client>
    /// Based on the key <str> provided, hash will be generated and will find the next key in the ring based on the ascending order
    /// If number is with in key's range generated, will return next available key else will send the first key in ring along with respective client
    /// For Example, Keys in ring [2027723236, 2123272817,2950756965] where as new key generated based on str input is '2114948387', so the next key would be '2123272817' and corresponding client
    pub async fn find_next_client(&self, key: &str) -> Option<&T> {
        let key = key.as_bytes();
        if self.clients.is_empty() {
            return None;
        }
        dbg!("all keys {:?}", &self.clients.keys());
        let hashed_key = (self.hasher)(key);
        println!("hashed key from request {}", hashed_key);
        let entry = self.clients.range(hashed_key..).next();
        if let Some((k, v)) = entry {
            println!("Found next key in ring - {:?}", k);
            return Some(v);
        }
        let first = self.clients.iter().next();
        let (k, v) = first.unwrap();
        println!("Found first key in ring - {:?}", k);

        Some(v)
    }

    /// Find the next balanced client from ring based on the key from request, return's Result<&Grpc_Client>
    pub async fn
    find<R>(
        &mut self,
        request: &R,
    ) -> anyhow::Result<&T>
        where
            R: ConsistentHashingTrait
    {
        let key = request.get_key();
        let c: &T = self.find_next_client(key.as_str()).await.unwrap();
        Ok(c)
    }
}

use crate::pb::HelloRequest;
use crate::pb::greeter_client::GreeterClient;
use crate::server::start_server;

const VIRTUAL_NODE_SIZE: usize = 3;

pub mod pb {
    tonic::include_proto!("helloworld");
}

impl ConsistentHashingTrait for HelloRequest {
    fn get_key(&self) -> String {
        (&self.key.as_str()).parse().unwrap()
    }
}

impl NewFromChannel for GreeterClient<Channel> {
    fn new(channel: Channel) -> Self {
        GreeterClient::new(channel)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server();

    let mut bal_client = StaticSetConsitentHashingLBClient::new().await;
    bal_client.add(&["http://[::1]:8080", "http://[::1]:8081", "http://[::1]:8082", "http://[::1]:8083", "http://[::1]:8084", "http://[::1]:8085"], VIRTUAL_NODE_SIZE).await;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".to_string(),
        key: "profile".to_string(),
    });

    let client: &GreeterClient<Channel> = bal_client.find(request.get_ref()).await?;

    let response = client.clone().say_hello(request).await;

    println!("RESPONSE={:?}", response);

    Ok(())
}

mod server {
    use std::thread::sleep;
    use std::time;

    use tonic::{Request, Response, Status};
    use tonic::transport::Server;

    use crate::pb;
    use crate::pb::{HelloReply, HelloRequest};
    use crate::pb::greeter_server::{Greeter, GreeterServer};

    #[derive(Default)]
    pub struct MyGreeter {}

    #[tonic::async_trait]
    impl Greeter for MyGreeter {
        async fn say_hello(
            &self,
            request: Request<HelloRequest>,
        ) -> Result<Response<HelloReply>, Status> {
            println!("Got a request from {:?}", request.remote_addr());
            let reply = pb::HelloReply {
                message: format!("Hello {}!", request.into_inner().name),
            };
            Ok(Response::new(reply))
        }
    }


    pub(crate) fn start_server() {
        let addrs = ["[::1]:8080", "[::1]:8081", "[::1]:8082", "[::1]:8083", "[::1]:8084", "[::1]:8085"];
        //let addrs = ["[::1]:50053"];
        for addr in &addrs {
            let addr = addr.parse().unwrap();

            let greeter = MyGreeter::default();
            let serve = Server::builder()
                .add_service(GreeterServer::new(greeter))
                .serve(addr);

            tokio::spawn(async move {
                println!("GreeterServer listening on {}", addr);
                if let Err(e) = serve.await {
                    eprintln!("Error = {:?}", e);
                }
            });
        }

        println!("GreeterServers running");
        sleep(time::Duration::from_secs(3));
    }
}
