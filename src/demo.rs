use std::task::{Context, Poll};

use tonic::body::BoxBody;
use tonic::client::GrpcService;
use tonic::codegen::http;
use tonic::transport::{Body, Channel, Error};
use tonic::transport::channel::ResponseFuture;
use tower::Service;

pub struct DemoChannel {
    chan: Channel,
}

impl DemoChannel {
    pub async fn new(uri: &'static str) -> Self {
        // TODO Setup K8S based upstream discovery and
        // TODO consistent hashing ring
        let endpoint = tonic::transport::Endpoint::new(uri).unwrap();
        let chan = endpoint.connect().await.unwrap();
        Self {
            chan
        }
    }

    fn select_a_channel(&self, hash: u64) -> Channel {
        // TODO Select channel from Bbtree ring based on hash value.
        self.chan.clone()
    }

    fn hash(req: &http::Request<BoxBody>) -> u64 {
        // TODO Calculate has value based on request meta data
        // TODO Issue: we only see an HTTP request here it seems
        // TODO and are missing meta data.
        0
    }
}

impl Service<http::Request<BoxBody>> for DemoChannel {
    type Response = http::Response<Body>;
    type Error = Error;
    type Future = ResponseFuture;


    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: http::Request<BoxBody>) -> Self::Future {
        let hash = Self::hash(&request);
        let mut channel = self.select_a_channel(hash);
        GrpcService::call(&mut channel, request)
    }
}

