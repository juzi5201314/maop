use async_session::{MemoryStore, SessionStore};
use axum::extract::{Extension, FromRequest, RequestParts};
use hyper::header::COOKIE;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::error::HttpServerError;

#[derive(Serialize, Deserialize)]
pub enum LoginStatus {
    Guest,
    Logged,
}

#[async_trait::async_trait]
impl<B> FromRequest<B> for LoginStatus
where
    B: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(
        req: &mut RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let Extension(store): Extension<MemoryStore> =
            Extension::<MemoryStore>::from_request(req)
                .await
                .server_error(
                    "`async_session::MemoryStore` extension missing",
                )?;

        Ok(
            if let Some(cookie) = req
                .headers()
                .unwrap()
                .get(COOKIE)
                .and_then(|value| value.to_str().ok())
                .map(|value| value.to_string())
            {
                if let Some(session) = store
                    .load_session(cookie)
                    .await
                    .server_error("load session failed")?
                    .map(|s| s.validate())
                    .flatten()
                {
                    if let Some(login_status) =
                        session.get::<LoginStatus>("login_status")
                    {
                        login_status
                    } else {
                        LoginStatus::Guest
                    }
                } else {
                    LoginStatus::Guest
                }
            } else {
                LoginStatus::Guest
            },
        )
    }
}
