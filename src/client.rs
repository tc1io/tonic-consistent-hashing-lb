use tonic::transport::Channel;
use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

// Reference - https://github.com/hyperium/tonic/blob/master/examples/src/load_balance/client.rs

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoints = ["http://[::1]:8086"]
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