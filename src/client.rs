use tonic::transport::Channel;
use tonic::transport::Endpoint;
use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use tokio::time::timeout;
use tower::discover::Change;


// Reference - https://github.com/hyperium/tonic/blob/master/examples/src/dynamic_load_balance/client.rs
// Reference - https://github.com/grpc/grpc-go/blob/master/examples/features/proto/echo/echo.proto
pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let mut client = GreeterClient::new("http://[::1]:8080").await?;

    let e1 = Endpoint::from_static("http://[::1]:8080");
    let e2 = Endpoint::from_static("http://[::1]:8081");

    let (channel, rx) = Channel::balance_channel(10);
    let mut client = GreeterClient::new(channel);

    let done = Arc::new(AtomicBool::new(false));
    let demo_done = done.clone();

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        println!("Added first endpoint");
        let change = Change::Insert("1", e1);
        let res = rx.send(change).await;
        println!("{:?}", res);
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        println!("Added second endpoint");
        let change = Change::Insert("2", e2);
        let res = rx.send(change).await;
        println!("{:?}", res);
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        println!("Removed first endpoint");
        let change = Change::Remove("1");
        let res = rx.send(change).await;
        println!("{:?}", res);

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        println!("Removed second endpoint");
        let change = Change::Remove("2");
        let res = rx.send(change).await;
        println!("{:?}", res);

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        println!("Added third endpoint");
        let e3 = Endpoint::from_static("http://[::1]:50051");
        let change = Change::Insert("3", e3);
        let res = rx.send(change).await;
        println!("{:?}", res);

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        println!("Removed third endpoint");
        let change = Change::Remove("3");
        let res = rx.send(change).await;
        println!("{:?}", res);
        demo_done.swap(true, SeqCst);
    });

    while !done.load(SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let request = tonic::Request::new(HelloRequest {
            message: "hello".into(),
        });

        let rx = client.say_hello(request);
        if let Ok(resp) = timeout(tokio::time::Duration::from_secs(10), rx).await {
            println!("RESPONSE={:?}", resp);
        } else {
            println!("did not receive value within 10 secs");
        }
    }

    println!("... Bye");

    Ok(())

    // let request = tonic::Request::new(HelloRequest {
    //     name: "Tonic".into(),
    // });
    //
    // let response = client.say_hello(request).await?;
    //
    // println!("RESPONSE={:?}", response);
    //
    // Ok(())
}