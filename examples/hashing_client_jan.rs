use std::collections::BTreeMap;
use std::fmt::Debug;

use fasthash::murmur3;
use rand::{Rng, thread_rng};
use rand::distributions::Uniform;
use tonic::transport::Channel;

use crate::pb::greeter_client::GreeterClient;
use crate::pb::HelloRequest;
use crate::server::start_server;

const VIRTUAL_NODE_SIZE: usize = 3;

pub mod pb {
    tonic::include_proto!("helloworld");
}

#[derive(Debug)]
pub struct StaticSetConsitentHashingLBClient<T> {
    clients: BTreeMap<u32, T>,
    hasher: fn(&[u8]) -> u32,
}

pub trait ConsistentHashingTrait {
    fn get_key(&self) -> String;
}

impl ConsistentHashingTrait for HelloRequest {
    fn get_key(&self) -> String {
        (&self.key.as_str()).parse().unwrap()
    }
}

fn create_hash(val: &[u8]) -> u32 {
    murmur3::hash32(val)
}

// ASHWIN
pub  trait NewFromChannel {
    fn new(channel: Channel) -> Self;
}

impl<T: NewFromChannel> StaticSetConsitentHashingLBClient<T> {
    // ASHWIN
    pub async fn new() -> Self {
        StaticSetConsitentHashingLBClient::with_hash(create_hash).await
    }

    pub async fn with_hash(hash_fn: fn(&[u8]) -> u32) -> Self {
        StaticSetConsitentHashingLBClient {
            hasher: hash_fn,
            clients: BTreeMap::new(),
        }
    }
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
                let client = T::new(channel); // ASHWIN
                self.clients.insert(key, client);
            }
        }
    }

    pub async fn find_next_client(&self, key: &str) -> Option<&T> { // ASHWIN
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

    pub async fn
    find<R>(
        &mut self,
        request: &R,
    ) -> anyhow::Result<&T> // ASHWIN
        where
            R: ConsistentHashingTrait
    {
        let key = request.get_key();

        let c: &T = self.find_next_client(key.as_str()).await.unwrap();
        Ok(c)
    }
}

// ASHWIN
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
