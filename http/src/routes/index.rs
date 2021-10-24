use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Extension, FromRequest, RequestParts};
use axum::handler::get;
use axum::response::Html;
use axum::routing::BoxRoute;
use axum::{Json, Router};
use sea_orm::prelude::DbConn;

use config::SiteConfig;
use database::models::post::{Post, PostModel};

use crate::error::HttpError;
use crate::error::HttpServerError;
use crate::login_status::LoginStatus;

pub fn routes() -> Router<BoxRoute> {
    let index =
        match config::get_config_temp().render().default_render() {
            config::RenderStrategy::SSR => {
                Router::new().route("/", get(index_ssr)).boxed()
            }
            config::RenderStrategy::CSR => {
                Router::new().route("/", get(index_csr)).boxed()
            }
        };

    let router = index
        .route("/api", get(index_api))
        .route("/ssr", get(index_ssr))
        .route("/csr", get(index_csr));

    router.boxed()
}

#[allow(clippy::needless_lifetimes)]
pub async fn index_ssr<'a>(
    data: Data,
    Extension(tm): Extension<Arc<template::TemplateManager<'a>>>,
) -> Result<Html<String>, HttpError> {
    tm.render("index", &data).map(Html).map_err(Into::into)
}

pub async fn index_csr() -> &'static str {
    "Hello, World!"
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
        let Extension(db): Extension<Arc<DbConn>> =
            Extension::<Arc<DbConn>>::from_request(req)
                .await
                .server_error("`DbConn` extension missing")?;
        let site = config::get_config_temp().site().clone();
        Ok(Data {
            site,
            logged: matches!(login_status, LoginStatus::Logged),
            posts: Post::find_all(&db).await?,
        })
    }
}
