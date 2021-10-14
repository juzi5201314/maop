use async_session::SessionStore as _;
use axum::extract::{Extension, FromRequest, RequestParts};
use std::ops::{Deref, DerefMut};

use crate::cookies::Cookies;
use crate::error::HttpError;
use crate::error::HttpServerError;
use crate::session_store::SessionStore;

pub struct Session(async_session::Session);

#[async_trait::async_trait]
impl<B> FromRequest<B> for Session
where
    B: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request(
        req: &mut RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let Extension(store): Extension<SessionStore> =
            Extension::<SessionStore>::from_request(req)
                .await
                .server_error("`SessionStore` extension missing")?;
        let cookie = Cookies::from_request(req)
            .await?
            .0
            .map(|c| c.get("session").map(|s| s.to_owned()))
            .flatten();
        Ok(Session(
            if let Some(cookie) = cookie {
                store
                    .load_session(cookie)
                    .await
                    .server_error("Failed to parse the session. Please clear the cookies")?
                    .map(|s| s.validate())
            } else {
                None
            }
            .flatten()
            .unwrap_or_default(),
        ))
    }
}

impl From<Session> for async_session::Session {
    fn from(s: Session) -> Self {
        s.0
    }
}

impl Deref for Session {
    type Target = async_session::Session;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Session {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
