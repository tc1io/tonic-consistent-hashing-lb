use std::task::{Context, Poll};

use tonic::body::BoxBody;
use tonic::client::GrpcService;
use tonic::codegen::http;
use tonic::transport::{Body, Error};
use tonic::transport::channel::ResponseFuture;
use tower::Service;

pub struct SillyChannel;

impl Service<http::Request<BoxBody>> for SillyChannel {
    type Response = http::Response<Body>;
    type Error = Error;
    type Future = ResponseFuture;


    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: http::Request<BoxBody>) -> Self::Future {
        todo!()
    }
}

