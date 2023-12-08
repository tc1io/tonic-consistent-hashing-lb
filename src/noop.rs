use std::task::{Context, Poll};

use tonic::body::BoxBody;
use tonic::codegen::http;
use tonic::transport::{Body, Error};
use tonic::transport::channel::ResponseFuture;
use tower::Service;

pub struct NoopChannel;

impl Service<http::Request<BoxBody>> for NoopChannel {
    type Response = http::Response<Body>;
    type Error = Error;
    type Future = ResponseFuture;


    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _request: http::Request<BoxBody>) -> Self::Future {
        todo!()
    }
}

