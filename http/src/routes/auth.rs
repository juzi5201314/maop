use std::sync::Arc;

use async_session::SessionStore as _;
use axum::body::{Body, Bytes, Full};
use axum::extract::{Extension, FromRequest, RequestParts};
use axum::handler::{get, post};
use axum::http::header::SET_COOKIE;
use axum::http::Response;
use axum::response::Html;
use axum::routing::BoxRoute;
use axum::{Json, Router};
use compact_str::CompactStr;
use hyper::StatusCode;
use anyhow::Context;

use config::SiteConfig;
use utils::password_hash::password_verify;

use crate::error::HttpError;
use crate::login_status::LoginStatus;
use crate::session::Session;
use crate::session_store::SessionStore;

pub type Password = Option<String>;

pub fn routes() -> Router<BoxRoute> {
    let router = Router::new()
        .route("/", post(login).get(index_ssr))
        .route("/logout", post(logout))
        .route("/api", get(index_api));

    router.boxed()
}

#[allow(clippy::needless_lifetimes)]
async fn index_ssr<'reg>(
    data: Data,
    Extension(tm): Extension<Arc<template::TemplateManager<'reg>>>,
) -> Result<Html<String>, HttpError> {
    tm.render("auth", &data)
        .map(Html)
        .map_err(Into::into)
}

async fn index_api(data: Data) -> Result<Json<Data>, HttpError> {
    Ok(Json(data))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Data {
    site: SiteConfig,
    logged: bool,
}

#[async_trait::async_trait]
impl FromRequest for Data {
    type Rejection = HttpError;

    async fn from_request(
        req: &mut RequestParts<Body>,
    ) -> Result<Self, Self::Rejection> {
        let login_status = LoginStatus::from_request(req).await?;
        let site = config::get_config_temp().site().clone();

        Ok(Data {
            site,
            logged: matches!(login_status, LoginStatus::Logged),
        })
    }
}

/// post
pub async fn login(
    login_status: LoginStatus,
    Json(data): Json<LoginData>,
    Extension(password): Extension<Arc<Password>>,
    Extension(store): Extension<SessionStore>,
    mut session: Session,
) -> Result<Response<Full<Bytes>>, HttpError> {
    let mut resp = Response::builder();
    if password.is_none() {
        resp = resp.status(StatusCode::FORBIDDEN)
    } else if !matches!(login_status, LoginStatus::Logged) {
        let password = (&*password).as_ref().unwrap();
        resp = if password_verify(data.password.as_bytes(), password)
            .context("failed to verify password")?
        {
            session
                .insert("login_status", LoginStatus::Logged)
                .unwrap();
            session.expire_in({
                *config::get_config_temp().http().session_expiry().duration()
            });
            let cookie = store
                .store_session(session.into())
                .await
                .context("failed to store session")?;

            resp.header(
                SET_COOKIE,
                format!("session={}", cookie.unwrap_or_default())
                    .as_bytes(),
            )
        } else {
            resp.status(StatusCode::UNAUTHORIZED)
        }
    };
    Ok(resp.body(Full::from("{}"))?)
}

#[derive(serde::Deserialize)]
pub struct LoginData {
    password: CompactStr,
}

pub async fn logout(
    login_status: LoginStatus,
    Extension(store): Extension<SessionStore>,
    session: Session,
) -> Result<Response<Full<Bytes>>, HttpError> {
    let resp = Response::builder();

    Ok(match login_status {
        LoginStatus::Guest => resp.status(StatusCode::FORBIDDEN),
        LoginStatus::Logged => {
            store
                .destroy_session(session.into())
                .await
                .context("destroy session failed")?;
            resp.status(StatusCode::OK)
        }
    }
    .body(Full::from("{}"))
    .unwrap())
}
