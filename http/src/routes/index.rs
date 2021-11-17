use std::sync::Arc;

use anyhow::Context;
use axum::body::Body;
use axum::extract::{Extension, FromRequest, RequestParts};
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};
use sea_orm::prelude::DbConn;

use config::{MaopConfig, SiteConfig};
use database::models::post::{Post, PostModel};

use crate::error::HttpError;
use crate::login_status::LoginStatus;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(index_ssr))
        .route("/api", get(index_api))
}

#[allow(clippy::needless_lifetimes)]
pub async fn index_ssr<'a>(
    data: Data,
    Extension(tm): Extension<Arc<template::TemplateManager<'a>>>,
) -> Result<Html<String>, HttpError> {
    tm.render("index", &data).map(Html).map_err(Into::into)
}

pub async fn index_api(data: Data) -> Result<Json<Data>, HttpError> {
    Ok(Json(data))
}

///todo: 分页
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Data {
    site: SiteConfig,
    logged: bool,
    posts: Vec<PostModel>,
}

#[async_trait::async_trait]
impl FromRequest for Data {
    type Rejection = HttpError;

    async fn from_request(
        req: &mut RequestParts<Body>,
    ) -> Result<Self, Self::Rejection> {
        let login_status = LoginStatus::from_request(req).await?;
        let Extension(db) =
            Extension::<Arc<DbConn>>::from_request(req)
                .await
                .context("`DbConn` extension missing")?;

        let Extension(config) =
            Extension::<Arc<MaopConfig>>::from_request(req)
                .await
                .context("`config` extension missing")?;

        let site = config.site().clone();
        Ok(Data {
            site,
            logged: matches!(login_status, LoginStatus::Logged),
            posts: Post::find_all(&db).await?,
        })
    }
}
