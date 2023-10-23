# gRPC-consistent-hashing
Implements gPRC Load Balancing with consistent hashing in Rust Tonic

-----------------------------------------------------------------------
Run Server

grpcurl -plaintext -import-path ./proto -proto helloworld.proto -d '{"name": "Tonic"}' '[::1]:8080' helloworld.Greeter/SayHello

-----------------------------------------------------------------------
Test Client and Server

cargo run --bin helloworld-server
cargo run --bin helloworld-client

----------------------------------------------------------------------
Helm Install

helm install <chart-name> helm-chart/grpc

----------------------------------------------------------------------
Port forward command

kubectl --namespace default port-forward <pod-name> <forwarding-port>:8086