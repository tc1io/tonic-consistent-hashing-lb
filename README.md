# gRPC-consistent-hashing
Implements gPRC client side Load Balancing with consistent hashing in Rust Tonic.

## Example
```rust
    use StaticSetConsitentHashingLBClient;
    use crate::pb::HelloRequest; //Refer Tonic Helloworld example to generate proto
    use crate::pb::greeter_client::GreeterClient; //Refer Tonic Helloworld example to generate proto

    const VIRTUAL_NODE_SIZE: usize = 3; // 

    pub mod pb {
        tonic::include_proto!("helloworld");
    }
    
    let mut bal_client = StaticSetConsitentHashingLBClient::new().await;
    
    // Provide static Uri's and virtual node size for virtual upstreams
    bal_client.add(&["http://[::1]:8080", "http://[::1]:8081", "http://[::1]:8082", 
    "http://[::1]:8083", "http://[::1]:8084", "http://[::1]:8085"], VIRTUAL_NODE_SIZE).await;

    // Request
    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".to_string(),
        key: "profile".to_string(),
    });

    // Find the balanced client based on key in request using consistent hashing
    let client: &GreeterClient<Channel> = bal_client.find(request.get_ref()).await?;

    // Access the 'say_hello' method from balanced client (say_hello generated based on grpc proto file)
    let response = client.clone().say_hello(request).await;

    println!("RESPONSE={:?}", response);

    Ok(())
```
**Trait's Required**
```rust
// Impl this trait on the grpc request struct to fetch the key from request 
impl ConsistentHashingTrait for HelloRequest {
    fn get_key(&self) -> String {
        (&self.key.as_str()).parse().unwrap()
    }
}
// Impl this trait on grpc_client<channel> to create new channel from balance list
impl NewFromChannel for GreeterClient<Channel> {
    fn new(channel: Channel) -> Self {
        GreeterClient::new(channel)
    }
}
```
**Note**

1. Refer [Tonic HelloWorld](https://github.com/hyperium/tonic/blob/master/examples/helloworld-tutorial.md) example to generate proto files and server side implementation.
2. Ensure Grpc Server is up and running before running the client
