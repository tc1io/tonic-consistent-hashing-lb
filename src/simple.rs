// use std::task::{Context, Poll};
// use futures_util::future::FutureExt;
// use tonic::body::BoxBody;
// use tonic::client::GrpcService;
// use tonic::codegen::http;
// use tonic::transport::{Body, Channel, Error};
// use tower::Service;
//
pub struct SimpleChannel;
//
// impl Service<http::Request<BoxBody>> for SimpleChannel {
//     type Response = http::Response<Body>;
//     type Error = Error;
//     type Future = i32;
//
//
//     fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         Poll::Ready(Ok(()))
//     }
//
//     fn call(&mut self, request: http::Request<BoxBody>) -> Self::Future {
//         // let inner = Service::call(&mut self.svc, request);
//         let e = tonic::transport::Endpoint::new("http://0.0.0.0:8087").unwrap();
//         let f2 = e.connect().map(|res: Result<Channel, _>| {
//             let chan = res.unwrap();
//             let f = GrpcService::call(&mut chan, request);
//             f
//         });
//
//         f2
//     }
// }
//
