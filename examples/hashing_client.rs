use crate::pb::{HelloReply, HelloRequest};
use crate::pb::greeter_client::GreeterClient;
use crate::server::start_server;
use tonic::transport::{Channel, Endpoint};
use std::collections::BTreeMap;
use std::fmt::Debug;
use fasthash::murmur3;
use tonic::Request;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};

const VIRTUAL_NODE_SIZE: usize = 3;
pub mod pb {
    tonic::include_proto!("helloworld");
}

#[derive(Debug)]
struct StaticSetConsitentHashingLBClient<T> {
    clients: BTreeMap<u32, Vec<GreeterClient<T>>>,
}


fn create_hash(val: &[u8]) -> u32 {
    murmur3::hash32(val)
}

impl StaticSetConsitentHashingLBClient<Channel> {
    pub async fn new(uris: &'static [&'static str], virtual_node_size: usize) -> Self {
        let mut s = Self { clients: BTreeMap::new() };

        let mut rng = thread_rng();
        let side = Uniform::new(1, 99999);
        for node_id in 0..virtual_node_size {
            for (i, u) in uris.chunks(2).enumerate() {

                let rand = rng.sample(side);
                let k = format!("{}_{}_{}", rand, i, node_id); // TODO:check if required to format and generate key, (its possible to generate with i and node_id without rand as well)

                let key = create_hash(k.as_bytes());
                //dbg!("key {:?}", &key);

                let mut clients = Vec::new();
                for uri in u {
                    let endpoint = Endpoint::from_static(uri);
                    let client = GreeterClient::connect(endpoint).await.unwrap();
                    clients.push(client);
                }
                s.clients.insert(key, clients);
            }
        }
        s
    }

    pub async fn find_next_client(&self, k: &str) -> Option<&Vec<GreeterClient<Channel>>> {
        let key = k.as_bytes();
        if self.clients.is_empty() {
            return None;
        }
        dbg!("all keys {:?}", &self.clients.keys());
        let hashed_key = create_hash(key);
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
    call(
        &mut self,
        request:  Request<HelloRequest>
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        let req = request.get_ref();
        let key = &req.key;

        let c: &Vec<GreeterClient<_>> = self.find_next_client(key).await.unwrap();
        // let channel = Channel::balance_list(c);
        // channel.say_hello(request).await //TODO: Figure out a way to get the first available server & hit
        c[0].clone().say_hello(request).await
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server();

    let mut balancing_client = StaticSetConsitentHashingLBClient::new(&["http://[::1]:8080","http://[::1]:8081","http://[::1]:8082","http://[::1]:8083","http://[::1]:8084","http://[::1]:8085"], VIRTUAL_NODE_SIZE).await;

    let request = tonic::Request::new(HelloRequest {
        name:"Tonic".to_string(),
        key: "profile".to_string()
    });

    println!("Saying Hello");
    let response = balancing_client.call(request).await?;
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
        let addrs = ["[::1]:8080","[::1]:8081","[::1]:8082","[::1]:8083","[::1]:8084","[::1]:8085"];
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
