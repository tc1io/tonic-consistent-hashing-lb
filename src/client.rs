use tonic::transport::Channel;
use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use fasthash::murmur3;
use prost::Message;
use std::collections::{BTreeMap};
use k8s_openapi::api::core::v1::Pod;
use kube::{Client, ResourceExt};
use kube::api::{Api, ListParams};

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
}

impl ConsistentHash {
    pub fn new() -> ConsistentHash {
        ConsistentHash {
            ring: BTreeMap::new(),
        }
    }
    pub fn add(&mut self, node: &Node) {
        self.remove(node);
        let node_id = format!("{}:{}", node.host, node.port);
        let key = create_hash(node_id.as_bytes());
        self.ring.insert(key, node.clone());
    }

    // pub fn list_ring(&self) {
    //     for (key, value) in self.ring.iter() {
    //         println!("{:?}: {:?}", key, value);
    //     }
    // }

    pub fn get_next_node(&self, k: &str) -> Option<&Node> {
        let key = k.as_bytes();
        if self.ring.is_empty() {
            return None;
        }

        let hashed_key = create_hash(key);

        let entry = self.ring.range(hashed_key..).next();
        //dbg!("whats next {:?}", entry);
        if let Some((_k, v)) = entry {
            return Some(v);
        }
        let first = self.ring.iter().next();
        let (_k, v) = first.unwrap();
        Some(v)
    }

    pub fn remove(&mut self, node: &Node) {
        let node_id = format!("{}:{}", node.host, node.port);
        let key = create_hash(node_id.as_bytes());
        self.ring.remove(&key);

    }
    pub fn len(&self) -> usize {
        self.ring.len()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

   // let mut ch = ConsistentHash::new();
   //
   //  let mut nodes = vec![];
   //  nodes.push(Node::new("http://0.0.0.0", 8087));
   //  nodes.push(Node::new("http://0.0.0.0", 8088));
   //  nodes.push(Node::new("http://0.0.0.0", 8089));
   //
   //  for node in nodes {
   //      ch.add(&node);
   //  }

    // println!("before remove count - {}", ch.len());
    //
    // ch.remove(&Node::new("http://0.0.0.0", 8088));
    //
    // println!("after remove count- {}", ch.len());

    // Test the get logic
    // for j in 0..50usize {
    //     let data = format!("hello-{}", j);
    //     let next = ch.get_next_node(data.as_str()).unwrap();
    //     println!("next {:?}", next);
    // }
    //
    // ch.list_ring();

    // TODO: Refactor k8s impl below
    let k8s_client = Client::try_default().await?;
    let pods: Api<Pod> = Api::default_namespaced(k8s_client);

    let lp = ListParams::default().labels("helm.sh/chart=grpc-0.1.0-server");
    for p in pods.list(&lp).await? {
        println!("Pod name: {}", p.name_any());
        println!("Pod ip: {:?}", p.status.unwrap().pod_ip.unwrap());
        let cont = p.spec.unwrap().containers;
        for c in cont {
            for p in c.ports.unwrap() {
                println!("Ports {:?} - {:?}", p.name.unwrap(), p.container_port)
            }
        }

    }
    //let endpoints = ["http://[::1]:8080", "http://[::1]:8081",  "http://[::1]:8082"]

    let endpoints = ["http://0.0.0.0:8087", "http://0.0.0.0:8088", "http://0.0.0.0:8089"]
    //let endpoints = ["http://10.244.0.205:8086", "http://10.244.0.206:8086", "http://10.244.0.207:8086"]
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