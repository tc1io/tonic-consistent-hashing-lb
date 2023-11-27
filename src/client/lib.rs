// use std::convert::Infallible;
// use std::future::{Ready, ready};
// use std::task::{Context, Poll};
// //use tonic::transport::Channel;
// use hello_world::greeter_client::GreeterClient;
// use hello_world::HelloRequest;
// use consistent_hash::ConsistentHash;
// use node::Node;
// use tonic::transport::channel as tonic_channel;
// use tower_service::Service;
// //use crate::ch::DebugService;
//
// pub mod consistent_hash;
// pub mod k8s;
// pub mod node;
// // pub mod channel;
// // pub mod endpoint;
// // pub mod error;
// // pub mod executor;
// // pub mod connection;
// // pub mod grpc_timeout;
// // pub mod reconnect;
// // pub mod user_agent;
// // pub mod add_origin;
// // pub mod dynamicservicestream;
// // mod tonic_service;
// // mod ch;
//
// type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
// type Error = Box<dyn std::error::Error + Send + Sync>;
//
// pub mod hello_world {
//     tonic::include_proto!("helloworld");
// }
//
// const POD_LABEL: &str = "helm.sh/chart=grpc-server";
// const PORT_NAME: &str = "grpc-server";
// const STATEFULSET_NAME: &str = "tonic-consistent-hashing";
//
// // Tonic LB Reference - https://github.com/hyperium/tonic/blob/master/examples/src/load_balance/client.rs
// // Consistent hash reference - https://github.com/zonyitoo/conhash-rs/blob/master/src/conhash.rs
//
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let mut ch = ConsistentHash::new();
//
//    let nodes: Vec<Node> = ch.get_pods(POD_LABEL, STATEFULSET_NAME, PORT_NAME).await?;
//
//     // let mut nodes = vec![];
//     // nodes.push(Node::new("http://test1", 8087));
//     // nodes.push(Node::new("http://test2", 8088));
//     // nodes.push(Node::new("http://test3", 8089));
//
//     if !nodes.is_empty() {
//         for node in nodes.iter() {
//             println!("Node: Host-{} Port-{}", node.host, node.port);
//             ch.add(&node);
//         }
//     }
//
//     // Test the get logic
//     for j in 0..50usize {
//         let data = format!("hello-{}", j);
//         let next = ch.get_next_node(data.as_str()).unwrap();
//         println!("next {:?}", next);
//     }
//
//     let endpoints = ["http://0.0.0.0:8087", "http://0.0.0.0:8088", "http://0.0.0.0:8089"]
//         //let endpoints = ["http://10.244.0.205:8086", "http://10.244.0.206:8086", "http://10.244.0.207:8086"]
//         .iter()
//         .map(|a| tonic_channel::Channel::from_static(a));
//
//     //let channel = channel::Channel::balance_list(endpoints);
//     let channel = tonic_channel::Channel::balance_list(endpoints);
//
//     let xxx = DebugService{};
//
//     //let mut client = GreeterClient::new(xxx);
//     let mut client = GreeterClient::new(channel);
//
//     for _ in 0..10usize {
//         let request = tonic::Request::new(HelloRequest {
//             name: "Hello gPRC".into(),
//         });
//
//         let response = client.say_hello(request).await?;
//
//         println!("RESPONSE={:?}", response);
//     }
//
//     Ok(())
// }
//
// pub struct DebugService;
//
// impl Service<()> for DebugService {
//     type Response = ();
//     type Error = Infallible;
//     type Future = Ready<Result<Self::Response, Self::Error>>;
//
//     fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         Ok(()).into()
//     }
//
//     fn call(&mut self, _req: ()) -> Self::Future {
//         ready(Ok(()))
//     }
// }














//
// use std::collections::HashMap;
// struct RpcClient {
//     servers: HashMap<String, String>,
// }
//
// impl RpcClient {
//     fn new(servers: HashMap<String, String>) -> Self {
//         RpcClient { servers }
//     }
//     fn make_rpc_call(&self, key: &str, request: &str) -> String {
//         let server = self.get_server(key);
//         format!("Making RPC call to server '{}' with request '{}'", server, request)
//     }
//     fn get_server(&self, key: &str) -> &str {
//         let hash = self.hash_key(key);
//         let servers = self.servers.values().collect::<Vec<&String>>();
//         let mut closest_server = servers[0];
//         let mut closest_distance = self.distance(hash, self.hash_key(servers[0]));
//
//         for server in servers.iter().skip(1) {
//             let distance = self.distance(hash, self.hash_key(server));
//             if distance < closest_distance {
//                 closest_server = server;
//                 closest_distance = distance;
//             }
//         }
//         closest_server
//     }
//     fn hash_key(&self, key: &str) -> u64 {
//         key.len() as u64
//     }
//     fn distance(&self, hash1: u64, hash2: u64) -> u64 {
//         if hash1 > hash2 {
//             hash1 - hash2
//         } else {
//             hash2 - hash1
//         }
//     }
// }
//
// fn main() {
//     let mut servers = HashMap::new();
//     servers.insert("server1".to_string(), "Server 1".to_string());
//     servers.insert("server2".to_string(), "Server 2".to_string());
//     servers.insert("server3".to_string(), "Server 3".to_string());
//
//     let rpc_client = RpcClient::new(servers);
//
//     for j in 0..50usize {
//         let key = format!("example_key_{}", j);
//         let request = format!("example_request_{}", j);
//         let response = rpc_client.make_rpc_call(key.as_str(), request.as_str());
//
//         println!("Response: {}", response);
//     }
// }







// use std::net::SocketAddr;
// use tokio::sync::mpsc;
// use tower::Service;
// use tower_grpc::{Request, Response};
// use tower_hyper::server::{Http, Server};
//
// // Define a service trait for the gRPC server
// //trait MyService: Service<Request<()>,  Response = Response<Vec<u8>>, Error = tower_grpc::Status> + Send + Sync + 'static {}
//
// // Define a struct that implements the MyService trait
// struct MyServiceImpl;
//
// // Implement the Service trait for MyServiceImpl
// impl Service<()> for MyServiceImpl {
//     //type Request = Request<Vec<u8>>;
//     type Response = Response<Vec<u8>>;
//     type Error = tower_grpc::Status;
//     type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;
//
//     fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
//         std::task::Poll::Ready(Ok(()))
//     }
//
//     fn call(&mut self, _request: ()) -> Self::Future {
//         // Implement the logic for handling the gRPC request and generating the response
//         // ...
//
//         // For demonstration purposes, we will simply return a fixed response
//         let response = Response::new(Vec::new());
//         Box::pin(async move { Ok(response) })
//     }
// }
//
// // Define a function to create and run the gRPC server
// async fn run_grpc_server(addr: SocketAddr) {
//     // Create a channel for communication between the server and client
//     let (tx, rx) = mpsc::channel(32);
//
//     // Create a new instance of the MyServiceImpl struct
//     let service = MyServiceImpl;
//
//     // Create a new gRPC server using the MyServiceImpl as the service
//     let server = Server::new(service);
//
//     // Bind the server to the specified address
//     let bind_addr = addr.clone();
//     tokio::spawn(async move {
//         if let Err(e) = server.serve_with_incoming(Http::new().http2_only(true), bind_addr, rx).await {
//             eprintln!("gRPC server error: {}", e);
//         }
//     });
//
//     // Print a message indicating that the server is running
//     println!("gRPC server listening on {}", addr);
//
//     // Wait for the server to finish
//     tokio::signal::ctrl_c().await.unwrap();
// }
//
// // Usage example: Running the gRPC server
// #[tokio::main]
// async fn main() {
//     // Specify the address on which the gRPC server should listen
//     let addr = "127.0.0.1:50051".parse().unwrap();
//
//     // Run the gRPC server
//     run_grpc_server(addr).await;
// }






use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;
use tokio::time::{sleep, Duration};
//use anyhow::{Result};


struct DelayService;

impl Service<&'static str> for DelayService {
    type Response = tokio::time::Sleep;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;


    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: &'static str) -> Self::Future {
        let delay_duration = Duration::from_secs(2);
        println!("received request: '{}', delaying for {} seconds", request, delay_duration.as_secs());

        //Box::new(sleep(delay_duration))
        let fut = async move{
            Ok(sleep(delay_duration))
        };

        Box::pin(fut)
    }
}

#[tokio::main]
async fn main() {
    let mut delay_service = DelayService;

    for j in 0..10usize {
        let request = "Hello, Tower Service!";
        let resp = delay_service.call(request);

        resp.await.expect("TODO: panic message").await;
        println!("response received from tower - {}, after delay", j);

    }
}
