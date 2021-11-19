use axum::extract::{FromRequest, RequestParts};
use axum::http::StatusCode;

use crate::error::HttpError;
use crate::session::Session;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum LoginStatus {
    Guest,
    Logged,
}

#[async_trait::async_trait]
impl<B> FromRequest<B> for LoginStatus
where
    B: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request(
        req: &mut RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request(req).await?;

        Ok(session
            .get::<LoginStatus>("login_status")
            .unwrap_or(LoginStatus::Guest))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Logged;

#[async_trait::async_trait]
impl<B> FromRequest<B> for Logged
where
    B: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request(
        req: &mut RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let login_status = LoginStatus::from_request(req).await?;
        if matches!(login_status, LoginStatus::Logged) {
            Ok(Logged)
        } else {
            Err(HttpError::from_const(
                StatusCode::UNAUTHORIZED,
                "Not logged in",
            ))
        }
    }
}
