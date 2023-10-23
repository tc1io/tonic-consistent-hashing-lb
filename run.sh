#!/bin/sh

docker build -t helloworld-server:latest -f Dockerfile .
docker build -t helloworld-client:latest -f Dockerfile .

helm install my-chart-grpc
