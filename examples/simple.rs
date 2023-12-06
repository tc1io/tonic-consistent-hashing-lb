use tonic_consistent_hashing_lb::DemoChannel;

use crate::pb::greeter_client::GreeterClient;
use crate::pb::HelloRequest;

pub mod pb {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    //let channel = SimpleChannel;
    let channel = DemoChannel::new("https://foo.bar").await;
    //let channel = ConsistentHashingK8SChannel();

    let mut client = GreeterClient::new(channel);
    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
