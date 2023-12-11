use crate::pb::{HelloReply, HelloRequest};
use crate::pb::greeter_client::GreeterClient;
use crate::server::start_server;
use tonic::transport::Endpoint;

pub mod pb {
    tonic::include_proto!("helloworld");
}

struct StaticSetConsitentHashingLBClient<T> {
    clients: Vec<GreeterClient<T>>,
}

impl StaticSetConsitentHashingLBClient<tonic::transport::Channel> {
    pub async fn new(uris: &'static [&'static str]) -> Self {
        let mut s = Self { clients: Vec::new() };
        for u in uris {
            let u = Endpoint::from_static(u);
            let client = GreeterClient::connect(u).await.unwrap();
            s.clients.push(client)
        }
        s
    }

    pub async fn say_hello(
        &mut self,
        request: impl tonic::IntoRequest<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        let hash = 0; // calculate hash from HelloRequest
        let idx = hash as usize % self.clients.len();
        let c: &GreeterClient<_> = self.clients.get(idx).unwrap();
        c.clone().say_hello(request).await
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server();

    let mut balancing_client = StaticSetConsitentHashingLBClient::new(&["http://[::1]:50053"]).await;

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
        //let addrs = ["[::1]:50051", "[::1]:50052"];
        let addrs = ["[::1]:50053"];
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
