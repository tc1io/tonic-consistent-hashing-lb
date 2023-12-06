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
        let e = tonic::transport::Endpoint::new(uri).unwrap();
        let chan = e.connect().await.unwrap();
        Self {
            chan
        }
    }

    fn select_a_channel(&self, hash: u64) -> Channel {
        todo!()
    }

    fn hash(req: http::Request<BoxBody>) -> u64 {
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
        GrpcService::call(&mut self.chan, request)

        // let hash = Self::hash(request);
        // let channel = self.select_a_channel(hash);
        // GrpcService::call(&mut channel, request)
    }
}

