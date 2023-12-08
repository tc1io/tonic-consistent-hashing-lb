use std::task::{Context, Poll};

use tonic::body::BoxBody;
use tonic::client::GrpcService;
use tonic::codegen::http;
use tonic::transport::{Body, Channel, Error};
use tonic::transport::channel::ResponseFuture;
use tower::Service;

pub struct StaticSetConsistentHashingLBChannel {
    channels: Vec<Channel>,
}

impl StaticSetConsistentHashingLBChannel {
    pub async fn new(uris: &'static [&'static str]) -> Self {
        // TODO Setup K8S based upstream discovery and
        // TODO consistent hashing ring
        let endpoints = uris.iter().map(|s| tonic::transport::Endpoint::new(*s).unwrap());
        let mut channels: Vec<Channel> = Vec::new();
        for e in endpoints {
            let c = e.connect().await.unwrap();
            channels.push(c)
        }
        Self {
            channels
        }
    }

    fn select_a_channel(&self, hash: i32) -> Channel {
        let idx = hash as usize % self.channels.len();
        let c: &Channel = self.channels.get(idx).unwrap();
        c.clone()
    }

    fn hash(_req: &http::Request<BoxBody>) -> i32 {
        // TODO Calculate has value based on request meta data
        // TODO Issue: we only see an HTTP request here it seems
        // TODO and are missing meta data.
        0
    }
}

impl Service<http::Request<BoxBody>> for StaticSetConsistentHashingLBChannel {
    type Response = http::Response<Body>;
    type Error = Error;
    type Future = ResponseFuture;


    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, request: http::Request<BoxBody>) -> Self::Future {
        let hash = Self::hash(&request);
        let mut channel = self.select_a_channel(hash);
        GrpcService::call(&mut channel, request)
    }
}

