//use tonic::transport::Channel;
use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use consistent_hash::ConsistentHash;
use node::Node;
use tonic::transport::channel as tonic_channel;
use crate::ch::DebugService;

pub mod consistent_hash;
pub mod k8s;
pub mod node;
pub mod channel;
pub mod endpoint;
pub mod error;
pub mod executor;
pub mod connection;
pub mod grpc_timeout;
pub mod reconnect;
pub mod user_agent;
pub mod add_origin;
pub mod dynamicservicestream;
mod tonic_service;
mod ch;

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
type Error = Box<dyn std::error::Error + Send + Sync>;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

const POD_LABEL: &str = "helm.sh/chart=grpc-server";
const PORT_NAME: &str = "grpc-server";
const STATEFULSET_NAME: &str = "tonic-consistent-hashing";

// Tonic LB Reference - https://github.com/hyperium/tonic/blob/master/examples/src/load_balance/client.rs
// Consistent hash reference - https://github.com/zonyitoo/conhash-rs/blob/master/src/conhash.rs

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ch = ConsistentHash::new();

   let nodes: Vec<Node> = ch.get_pods(POD_LABEL, STATEFULSET_NAME, PORT_NAME).await?;

    // let mut nodes = vec![];
    // nodes.push(Node::new("http://test1", 8087));
    // nodes.push(Node::new("http://test2", 8088));
    // nodes.push(Node::new("http://test3", 8089));

    if !nodes.is_empty() {
        for node in nodes.iter() {
            println!("Node: Host-{} Port-{}", node.host, node.port);
            ch.add(&node);
        }
    }

    // Test the get logic
    for j in 0..50usize {
        let data = format!("hello-{}", j);
        let next = ch.get_next_node(data.as_str()).unwrap();
        println!("next {:?}", next);
    }

    let endpoints = ["http://0.0.0.0:8087", "http://0.0.0.0:8088", "http://0.0.0.0:8089"]
        //let endpoints = ["http://10.244.0.205:8086", "http://10.244.0.206:8086", "http://10.244.0.207:8086"]
        .iter()
        .map(|a| tonic_channel::Channel::from_static(a));

    //let channel = channel::Channel::balance_list(endpoints);
    let channel = tonic_channel::Channel::balance_list(endpoints);

    let xxx = DebugService{};

    //let mut client = GreeterClient::new(channel);
    let mut client = GreeterClient::new(xxx);

    for _ in 0..10usize {
        let request = tonic::Request::new(HelloRequest {
            name: "Hello gPRC".into(),
        });

        let response = client.say_hello(request).await?;

        println!("RESPONSE={:?}", response);
    }

    Ok(())
}