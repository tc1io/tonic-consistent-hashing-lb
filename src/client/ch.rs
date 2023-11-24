use std::convert::Infallible;
use tower::discover::{Change, Discover};
use tokio::sync::mpsc::{Sender, channel};
use std::hash::Hash;
use crate::executor::SharedExec;
use crate::executor::Executor;
use std::pin::Pin;
use std::future::{Future, Ready, ready};
use std::task::{Context, Poll};
use http::{Request, Response, StatusCode};
use crate::dynamicservicestream::DynamicServiceStream;
use crate::endpoint::Endpoint;
use crate::connection::Connection;
use tower::{buffer, Service};
use tonic::body::BoxBody;

#[derive(Clone)]
pub struct Channelx {
    svc: Vec<String>,
}

pub struct ResponseFuture;
// {
//     inner: buffer::future::ResponseFuture<<Svc as Service<Request<BoxBody>>>::Future>,
// }

//let endpoints = ["http://0.0.0.0:8087", "http://0.0.0.0:8088", "http://0.0.0.0:8089"];
impl Channelx {

    pub fn balance_list(list: impl Iterator<Item = Endpoint>) -> Self {
        let (channel, tx) = Self::balance_channel(1024);
        list.for_each(|ep| {
            tx.try_send(Change::Insert(ep.uri.clone(), ep))
                .unwrap();
        });

        channel
    }

    pub fn balance_channel<K>(capacity: usize) -> (Self, Sender<Change<K, Endpoint>>)
        where
            K: Hash + Eq + Send + Clone + 'static,
    {
        Self::balance_channel_with_executor(capacity, SharedExec::tokio())
    }

    pub fn balance_channel_with_executor<K, E>(
        capacity: usize,
        executor: E,
    ) -> (Self, Sender<Change<K, Endpoint>>)
        where
            K: Hash + Eq + Send + Clone + 'static,
            E: Executor<Pin<Box<dyn Future<Output = ()> + Send>>> + Send + Sync + 'static,
    {
        let (tx, rx) = channel(capacity);
        let list = DynamicServiceStream::new(rx);
        (Self::balance(list, 1024, executor), tx)
    }

    pub(crate) fn balance<D, E>(discover: D, buffer_size: usize, executor: E) -> Self
        where
            D: Discover<Service = Connection> + Unpin + Send + 'static,
            D::Error: Into<crate::Error>,
            D::Key: Hash + Send + Clone,
            E: Executor<crate::BoxFuture<'static, ()>> + Send + Sync + 'static,
    {
        //let svc = Balance::new(discover);

        let mut svc = vec![];
        svc.push(String::from("http://0.0.0.0:8087"));
        svc.push(String::from("http://0.0.0.0:8088"));
        svc.push(String::from("http://0.0.0.0:8089"));

        //let (svc, worker) = Buffer::pair(Either::B(svc), buffer_size);
        //executor.execute(Box::pin(worker));

        Channelx { svc }
    }
}

impl Service<http::Request<BoxBody>> for Channelx {
    type Response = http::Response<hyper::Body>;
    type Error = super::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.svc[0], cx).map_err(super::Error::from_source)
    }

    fn call(&mut self, request: http::Request<BoxBody>) -> Self::Future {
        //let inner = Service::call(&mut self.svc, request);
        println!("request in");
        dbg!(&request);
        let inner = Service::call(&mut self.svc[0], request);
        // let body: Vec<u8> = "hello, world!\n"
        //     .as_bytes()
        //     .to_owned();
        // // Create the HTTP response
        // let resp = Response::builder()
        //     .status(StatusCode::OK)
        //     .body(body)
        //     .expect("Unable to create `http::Response`");

        let fut = async {
            Ok(inner)
        };
        Box::pin(fut)
    }
}


pub struct DebugService;

impl Service<()> for DebugService {
    type Response = ();
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _req: ()) -> Self::Future {
        ready(Ok(()))
    }
}
