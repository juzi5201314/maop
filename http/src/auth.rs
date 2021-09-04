use async_session::{MemoryStore, SessionStore};
use axum::extract::{Extension, FromRequest, RequestParts};
use hyper::StatusCode;
use hyper::header::COOKIE;

use crate::error::HttpServerError;

pub enum LoginStatus {
    Guest,
    Logged
}

#[async_trait::async_trait]
impl<B> FromRequest<B> for LoginStatus
where
    B: Send,
{
    type Rejection = (StatusCode, String);

    async fn from_request(
        req: &mut RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let Extension(store): Extension::<MemoryStore> =
            Extension::<MemoryStore>::from_request(req)
                .await
                .server_error("`async_session::MemoryStore` extension missing")?;

        if let Some(cookie) = headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string())
        {
            if let Some(session) = store.load_session(cookie).await.server_error("load session failed")? {

            }
        } else {

        }
    }
}
