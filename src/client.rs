use tonic::transport::Channel;
use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use fasthash::murmur3;
use prost::Message;
use std::collections::{BTreeMap, HashMap};

// Tonic LB Reference - https://github.com/hyperium/tonic/blob/master/examples/src/load_balance/client.rs
// Consistent hash reference - https://github.com/zonyitoo/conhash-rs/blob/master/src/conhash.rs

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

fn create_hash(val: &[u8]) -> Vec<u8> {
    murmur3::hash32(val).encode_to_vec()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Node {
    host: String,
    port: u16,
}

impl Node {
    fn new(host: &str, port: u16) -> Node {
        Node {
            host: host.to_string(),
            port,
        }
    }
}

pub struct ConsistentHash {
    ring: BTreeMap<Vec<u8>, Node>,
    replicas: HashMap<String, usize>,
}

impl ConsistentHash {

    pub fn new() -> ConsistentHash {
        ConsistentHash {
            ring: BTreeMap::new(),
            replicas: HashMap::new(),
        }
    }

    pub fn add(&mut self, node: Node, rep_count: usize) {
        let node_name = format!("{}:{}", node.host, node.port);
        self.replicas.insert(node_name.clone(), rep_count);
        for replica in 0..rep_count {
            let node_id = format!("{}:{}", node_name, replica);
            let key = create_hash(node_id.as_bytes());
            self.ring.insert(key, node.clone());
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

   let mut ch = ConsistentHash::new();

    let mut nodes = vec![];
    // TODO: Get pods from kubeapi (Check for appropriate crate)
    nodes.push(Node::new("http://0.0.0.0", 8087));
    nodes.push(Node::new("http://0.0.0.0", 8088));
    nodes.push(Node::new("http://0.0.0.0", 8089));

    for node in nodes {
        ch.add(node, 3);
    }

    //let endpoints = ["http://[::1]:8080", "http://[::1]:8081",  "http://[::1]:8082"]

    let endpoints = ["http://0.0.0.0:8087", "http://0.0.0.0:8088", "http://0.0.0.0:8089"]

        .iter()
        .map(|a| Channel::from_static(a));

    let channel = Channel::balance_list(endpoints);

    let mut client = GreeterClient::new(channel);

    for _ in 0..10usize {
        let request = tonic::Request::new(HelloRequest {
            name: "Hello gPRC".into(),
        });

        let response = client.say_hello(request).await?;

        println!("RESPONSE={:?}", response);
    }

    Ok(())
}