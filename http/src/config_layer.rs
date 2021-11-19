use std::task::{Context, Poll};

use arc_swap::{ArcSwapAny, Cache};
use axum::http::Request;
use config::MaopConfig;
use hyper::service::Service;
use hyper::Response;
use std::sync::Arc;
use tower::Layer;

#[derive(Copy, Clone, Debug)]
pub struct ConfigLayer;

#[derive(Clone, Debug)]
pub struct ConfigService<S> {
    inner: S,
    cache: Cache<Arc<ArcSwapAny<Arc<MaopConfig>>>, Arc<MaopConfig>>,
}

impl ConfigLayer {
    pub fn new() -> Self {
        ConfigLayer
    }
}

impl<S> Layer<S> for ConfigLayer {
    type Service = ConfigService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ConfigService {
            inner,
            cache: config::get_config_cache(),
        }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>>
    for ConfigService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        req.extensions_mut().insert(Arc::clone(self.cache.load()));

        self.inner.call(req)
    }
}
