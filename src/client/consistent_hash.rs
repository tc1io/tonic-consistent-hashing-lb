use std::collections::{BTreeMap};
use fasthash::murmur3;
use prost::Message;

fn create_hash(val: &[u8]) -> Vec<u8> {
    murmur3::hash32(val).encode_to_vec()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Node {
    host: String,
    port: u16,
}

impl Node {
    pub(crate) fn new(host: &str, port: u16) -> Node {
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