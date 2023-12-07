use tonic_consistent_hashing_lb::DemoChannel;

use crate::pb::greeter_client::GreeterClient;
use crate::pb::HelloRequest;
use crate::server::start_server;

pub mod pb {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server();

    //let channel = SimpleChannel;
    let channel = DemoChannel::new("http://[::1]:50053").await;

    let mut client = GreeterClient::new(channel);
    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    println!("Saying Hello");
    let response = client.say_hello(request).await?;

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
