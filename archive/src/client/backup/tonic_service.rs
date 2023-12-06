use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use http::{Request, Response, StatusCode};
use tower::Service;
use crate::channel::Channel;



impl Service<Request<Vec<u8>>> for Channel {
    type Response = Response<Vec<u8>>;
    type Error = http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Vec<u8>>) -> Self::Future {
        let body: Vec<u8> = "hello, world!\n"
            .as_bytes()
            .to_owned();
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .expect("Unable to create `http::Response`");

        let fut = async {
            Ok(resp)
        };

        Box::pin(fut)
    }
}