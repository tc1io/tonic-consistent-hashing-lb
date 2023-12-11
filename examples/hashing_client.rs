use crate::pb::{HelloReply, HelloRequest};
use crate::pb::greeter_client::GreeterClient;
use crate::server::start_server;
use tonic::transport::Endpoint;
use std::collections::BTreeMap;
use fasthash::murmur3;
use prost::Message;

pub mod pb {
    tonic::include_proto!("helloworld");
}

struct StaticSetConsitentHashingLBClient<T> {
    clients: BTreeMap<Vec<u8>, Vec<GreeterClient<T>>>,
}


fn create_hash(val: &[u8]) -> Vec<u8> {
    murmur3::hash32(val).encode_to_vec()
}

impl StaticSetConsitentHashingLBClient<tonic::transport::Channel> {
    pub async fn new(uris: &'static [&'static str]) -> Self {
        let mut s = Self { clients: BTreeMap::new() };
        for (i,u) in uris.iter().enumerate() {
            let k = create_hash("TODO_Key".as_bytes());
            println!("hash in new{:?}", k);
            let u = Endpoint::from_static(u);
            let mut c = Vec::new();
            let client = GreeterClient::connect(u).await.unwrap();
            c.push(client);
            s.clients.insert(k, c);
        }
        s
    }

    pub async fn
    say_hello(
        &mut self,
        request: impl tonic::IntoRequest<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        let hash =  create_hash("TODO_Key".as_bytes());;// calculate hash from HelloRequest
        println!("hash {:?}", hash);
        //let idx = hash as usize % self.clients.len();
        let c: &Vec<GreeterClient<_>> = self.clients.get(&*hash).unwrap();
        c[0].clone().say_hello(request).await
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server();

    let mut balancing_client = StaticSetConsitentHashingLBClient::new(&["http://[::1]:8080","http://[::1]:8081","http://[::1]:8082","http://[::1]:8083","http://[::1]:8084","http://[::1]:8085"]).await;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    println!("Saying Hello");
    let response = balancing_client.say_hello(request).await?;
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
