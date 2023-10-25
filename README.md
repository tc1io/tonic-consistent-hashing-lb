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
Build Image

docker build -t helloworld-server:latest -f Dockerfile .


For Minikube
eval $(minikube docker-env)
docker build -t localhost:32000/tonic-consistent-hashing-lb/helloworld-server:latest -f Dockerfile .

---------------------------------------------------------------------
Helm Install

helm install <chart-name> helm-chart/grpc

----------------------------------------------------------------------
Port forward command

kubectl --namespace default port-forward <pod-name> <forwarding-port>:8086