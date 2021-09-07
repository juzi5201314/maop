use crate::auth::LoginStatus;
use crate::error::HttpServerError;
use async_session::{MemoryStore, Session, SessionStore};
use axum::extract::Extension;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Duration;
use config::HttpConfig;
use hyper::{Response, StatusCode};
use serde::Deserialize;
use std::ops::Add;
use utils::password_hash::password_verify;

pub async fn login(
    login_status: LoginStatus,
    Json(data): Json<LoginData>,
    Extension(config): Extension<HttpConfig>,
    Extension(store): Extension<MemoryStore>,
) -> Result<(StatusCode, &'static str), (StatusCode, String)> {
    Ok(if let Some(hash) = config.password() {
        if password_verify(data.password.as_bytes(), hash)
            .server_error("failed to verify password")?
        {
            let mut session = Session::new();
            session
                .insert("login_status", LoginStatus::Logged)
                .unwrap();
            session.set_expiry(
                chrono::Utc::now().add(Duration::hours(24)),
            );
            store
                .store_session(session)
                .await
                .server_error("failed to store session")?;
            (StatusCode::OK, "Ok")
        } else {
            (StatusCode::UNAUTHORIZED, "wrong password")
        }
    } else {
        (StatusCode::UNAUTHORIZED, "no password is set")
    })
}

#[derive(Deserialize)]
pub struct LoginData {
    password: String,
}
