#!/usr/bin/env bash


eval $(minikube docker-env)

docker build -t localhost:32000/tonic-consistent-hashing-lb/helloworld-server:latest -f Dockerfile .



helm install my-chart-grpc helm-chart/grpc


#docker build -t helloworld-server -f Dockerfile .