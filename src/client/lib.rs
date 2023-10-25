use k8s_openapi::api::apps::v1::StatefulSet;
use tonic::transport::Channel;
use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use consistent_hash::ConsistentHash;
use node::Node;
mod consistent_hash;
mod k8s;
mod node;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

// Tonic LB Reference - https://github.com/hyperium/tonic/blob/master/examples/src/load_balance/client.rs
// Consistent hash reference - https://github.com/zonyitoo/conhash-rs/blob/master/src/conhash.rs

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ch = ConsistentHash::new();

    let mut nodes = vec![];
    nodes.push(Node::new("http://0.0.0.0", 8087));
    nodes.push(Node::new("http://0.0.0.0", 8088));
    nodes.push(Node::new("http://0.0.0.0", 8089));

    for node in nodes {
        ch.add(&node);
    }

    println!("before remove count - {}", ch.len());

    ch.remove(&Node::new("http://0.0.0.0", 8088));

    println!("after remove count- {}", ch.len());

    // Test the get logic
    for j in 0..50usize {
        let data = format!("hello-{}", j);
        let next = ch.get_next_node(data.as_str()).unwrap();
        println!("next {:?}", next);
    }

    //ch.list_ring();

    // TODO: Refactor k8s impl below

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