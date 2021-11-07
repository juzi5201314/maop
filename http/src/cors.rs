use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::http::Request;
use compact_str::CompactStr;
use hyper::header::{
    HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN, ORIGIN,
};
use hyper::service::Service;
use hyper::Response;
use tower::Layer;

#[derive(Clone, Debug)]
pub struct CorsLayer {
    origins: Vec<CompactStr>,
    allow_all: bool,
}

#[derive(Clone, Debug)]
pub struct Cors<S> {
    inner: S,
    layer: CorsLayer,
}

#[pin_project::pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    fut: F,
    allow_origin: Option<HeaderValue>,
}

impl CorsLayer {
    pub fn new(origins: Vec<CompactStr>) -> Self {
        if origins.contains(&CompactStr::new_inline("*")) {
            CorsLayer {
                origins: Vec::new(),
                allow_all: true,
            }
        } else {
            CorsLayer {
                origins,
                allow_all: false,
            }
        }
    }
}

impl<S> Layer<S> for CorsLayer {
    type Service = Cors<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Cors {
            inner,
            layer: self.clone(),
        }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for Cors<S>
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

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let headers = req.headers_mut();

        let mut allow_origin = None;
        if self.layer.allow_all {
            allow_origin = Some(HeaderValue::from_static("*"));
        } else {
            let origin = headers
                .get(ORIGIN)
                .map(|val| val.to_str().ok())
                .flatten()
                .map(CompactStr::new);
            if let Some(origin) = origin {
                if self.layer.origins.contains(&origin) {
                    allow_origin =
                        Some(HeaderValue::from_str(&origin).unwrap());
                }
            }
        }

        ResponseFuture {
            fut: self.inner.call(req),
            allow_origin,
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
        let mut response: Response<B> =
            futures::ready!(this.fut.poll(cx))?;

        if let Some(header) = this.allow_origin.take() {
            response
                .headers_mut()
                .insert(ACCESS_CONTROL_ALLOW_ORIGIN, header);
        }

        Poll::Ready(Ok(response))
    }
}
