use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc;
use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use std::net::SocketAddr;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Debug)]
pub struct MyGreeter {
    addr: SocketAddr,
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let host = hostname::get()?;

        let reply = hello_world::HelloReply {
            message: format!("Socket add: {}, {} (from {:?})", self.addr, request.into_inner().name, host),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:8086";

    let (tx, mut rx) = mpsc::unbounded_channel();
        let addr = addr.parse()?;
        let tx = tx.clone();

        let greeter = MyGreeter { addr };
        let serve = Server::builder()
            .add_service(GreeterServer::new(greeter))
            .serve(addr);

        tokio::spawn(async move {
            if let Err(e) = serve.await {
                eprintln!("Error = {:?}", e);
            }

            tx.send(()).unwrap();
        });

    rx.recv().await;

    Ok(())
}