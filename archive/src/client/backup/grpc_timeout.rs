//use crate::metadata::GRPC_TIMEOUT_HEADER;
use http::{HeaderMap, HeaderValue, Request};
use pin_project::pin_project;
use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{ready, Context, Poll},
    time::Duration,
};
use crate::error::Error;
use tokio::time::Sleep;
use tower_service::Service;

const GRPC_TIMEOUT_HEADER: &str = "grpc-timeout";

#[derive(Debug, Clone)]
pub(crate) struct GrpcTimeout<S> {
    inner: S,
    server_timeout: Option<Duration>,
}

impl<S> GrpcTimeout<S> {
    pub(crate) fn new(inner: S, server_timeout: Option<Duration>) -> Self {
        Self {
            inner,
            server_timeout,
        }
    }
}

impl<S, ReqBody> Service<Request<ReqBody>> for GrpcTimeout<S>
where
    S: Service<Request<ReqBody>>,
    S::Error: Into<Error>,
{
    type Response = S::Response;
    type Error = Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let client_timeout = try_parse_grpc_timeout(req.headers()).unwrap_or_else(|e| {
            tracing::trace!("Error parsing `grpc-timeout` header {:?}", e);
            None
        });

        // Use the shorter of the two durations, if either are set
        let timeout_duration = match (client_timeout, self.server_timeout) {
            (None, None) => None,
            (Some(dur), None) => Some(dur),
            (None, Some(dur)) => Some(dur),
            (Some(header), Some(server)) => {
                let shorter_duration = std::cmp::min(header, server);
                Some(shorter_duration)
            }
        };

        ResponseFuture {
            inner: self.inner.call(req),
            sleep: timeout_duration
                .map(tokio::time::sleep)
                .map(Some)
                .unwrap_or(None),
        }
    }
}

#[pin_project]
pub(crate) struct ResponseFuture<F> {
    #[pin]
    inner: F,
    #[pin]
    sleep: Option<Sleep>,
}

impl<F, Res, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Res, E>>,
    E: Into<Error>,
{
    type Output = Result<Res, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if let Poll::Ready(result) = this.inner.poll(cx) {
            return Poll::Ready(result.map_err(Into::into));
        }

        if let Some(sleep) = this.sleep.as_pin_mut() {
            ready!(sleep.poll(cx));
            return Poll::Ready(Err(TimeoutExpired(()).into()));
        }

        Poll::Pending
    }
}

const SECONDS_IN_HOUR: u64 = 60 * 60;
const SECONDS_IN_MINUTE: u64 = 60;

/// Tries to parse the `grpc-timeout` header if it is present. If we fail to parse, returns
/// the value we attempted to parse.
///
/// Follows the [gRPC over HTTP2 spec](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md).
fn try_parse_grpc_timeout(
    headers: &HeaderMap<HeaderValue>,
) -> Result<Option<Duration>, &HeaderValue> {
    match headers.get(GRPC_TIMEOUT_HEADER) {
        Some(val) => {
            let (timeout_value, timeout_unit) = val
                .to_str()
                .map_err(|_| val)
                .and_then(|s| if s.is_empty() { Err(val) } else { Ok(s) })?
                // `HeaderValue::to_str` only returns `Ok` if the header contains ASCII so this
                // `split_at` will never panic from trying to split in the middle of a character.
                // See https://docs.rs/http/0.2.4/http/header/struct.HeaderValue.html#method.to_str
                //
                // `len - 1` also wont panic since we just checked `s.is_empty`.
                .split_at(val.len() - 1);

            // gRPC spec specifies `TimeoutValue` will be at most 8 digits
            // Caping this at 8 digits also prevents integer overflow from ever occurring
            if timeout_value.len() > 8 {
                return Err(val);
            }

            let timeout_value: u64 = timeout_value.parse().map_err(|_| val)?;

            let duration = match timeout_unit {
                // Hours
                "H" => Duration::from_secs(timeout_value * SECONDS_IN_HOUR),
                // Minutes
                "M" => Duration::from_secs(timeout_value * SECONDS_IN_MINUTE),
                // Seconds
                "S" => Duration::from_secs(timeout_value),
                // Milliseconds
                "m" => Duration::from_millis(timeout_value),
                // Microseconds
                "u" => Duration::from_micros(timeout_value),
                // Nanoseconds
                "n" => Duration::from_nanos(timeout_value),
                _ => return Err(val),
            };

            Ok(Some(duration))
        }
        None => Ok(None),
    }
}

/// Error returned if a request didn't complete within the configured timeout.
///
/// Timeouts can be configured either with [`Endpoint::timeout`], [`Server::timeout`], or by
/// setting the [`grpc-timeout` metadata value][spec].
///
/// [`Endpoint::timeout`]: crate::transport::server::Server::timeout
/// [`Server::timeout`]: crate::transport::channel::Endpoint::timeout
/// [spec]: https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md
#[derive(Debug)]
pub struct TimeoutExpired(());

impl fmt::Display for TimeoutExpired {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Timeout expired")
    }
}

// std::error::Error only requires a type to impl Debug and Display
impl std::error::Error for TimeoutExpired {}

