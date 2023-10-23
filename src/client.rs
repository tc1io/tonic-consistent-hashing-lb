use tonic::transport::Channel;
use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use fasthash::murmur3;
use prost::Message;

// Reference - https://github.com/hyperium/tonic/blob/master/examples/src/load_balance/client.rs

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

fn create_hash(val: &[u8]) -> Vec<u8> {
    murmur3::hash32(val).encode_to_vec()
}

pub struct Ring {
    key: String,
    hash: Vec<u8>,
    server: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {



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