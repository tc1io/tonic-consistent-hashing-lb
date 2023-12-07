use std::task::{Context, Poll};

use tonic::body::BoxBody;
use tonic::client::GrpcService;
use tonic::codegen::http::{Request, Response};
use tonic::transport::{Body, Channel, Error};
use tonic::transport::channel::ResponseFuture;
use tower::Service;

pub struct JanChannel {
    chan: Channel,
}

impl JanChannel {
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

    fn hs(req: &Request<BoxBody>) -> u64 {
        // TODO Calculate has value based on request meta data
        // TODO Issue: we only see an HTTP request here it seems
        // TODO and are missing meta data.
        0
    }
}

impl Service<Request<BoxBody>> for JanChannel {
    type Response = Response<Body>;
    type Error = Error;
    type Future = ResponseFuture;


    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        dbg!("aaa");
        let hash = 0; //Self::hs(&request);
        let mut channel = self.select_a_channel(hash);
        let x = Service::poll_ready(&mut channel, cx); //.map_err(super::Error::from_source)
        //Service::poll_ready(self, cx) //.map_err(super::Error::from_source)
        dbg!("bbb");
        x
    }

    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        dbg!("cc");
        let hash = Self::hs(&request);
        let mut channel = self.select_a_channel(hash);
        let y = GrpcService::call(&mut channel, request);
        dbg!("dd");
        y
    }
}

