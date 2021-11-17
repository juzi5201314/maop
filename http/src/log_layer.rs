use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::http::{Method, Request, Uri, Version};
use hyper::service::Service;
use hyper::Response;
use tower::Layer;

#[derive(Copy, Clone, Debug)]
pub struct LogLayer;

#[derive(Clone, Debug)]
pub struct LogService<S> {
    inner: S,
}

#[derive(Clone, Debug)]
pub struct LogData {
    method: Method,
    uri: Uri,
    version: Version,
}

#[pin_project::pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    fut: F,
    data: LogData,
}

impl LogLayer {
    pub fn new() -> Self {
        LogLayer
    }
}

impl<S> Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LogService { inner }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for LogService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        ResponseFuture {
            data: LogData {
                method: req.method().clone(),
                uri: req.uri().clone(),
                version: req.version(),
            },
            fut: self.inner.call(req),
        }
    }
}

impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = Result<Response<B>, E>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let this = self.project();

        let now = Instant::now();

        let response: Response<B> =
            futures::ready!(this.fut.poll(cx))?;

        let end = now.elapsed();

        let msg = format!(
            "{:?} {} {} {} ~ {:?}",
            this.data.version,
            response.status(),
            this.data.method.as_str(),
            this.data
                .uri
                .path_and_query()
                .map(|q| q.as_str())
                .unwrap_or("<unknown>"),
            end
        );
        if response.status().is_server_error() {
            log::warn!("{}", msg);
        } else {
            log::info!("{}", msg);
        }

        Poll::Ready(Ok(response))
    }
}
