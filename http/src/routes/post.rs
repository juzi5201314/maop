use std::collections::BTreeMap;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Extension, FromRequest, RequestParts};
use axum::handler::get;
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::BoxRoute;
use axum::{extract, Json, Router};
use sea_orm::prelude::DbConn;

use config::SiteConfig;
use database::models::comment::CommentModel;
use database::models::post::{PostModel, Post};

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
pub async fn index_ssr<'reg>(
    data: Data,
    Extension(tm): Extension<Arc<template::TemplateManager<'reg>>>,
) -> Result<Html<String>, HttpError> {
    tm.render("post", &data).map(Html).map_err(Into::into)
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
    post: PostModel,
    comments: BTreeMap<u32, CommentModel>,
}

#[async_trait::async_trait]
impl FromRequest for Data {
    type Rejection = HttpError;

    async fn from_request(
        req: &mut RequestParts<Body>,
    ) -> Result<Self, Self::Rejection> {
        let login_status = LoginStatus::from_request(req).await?;
        let extract::Path(post_id) =
            extract::Path::<u32>::from_request(req).await?;
        let Extension(db): Extension<Arc<DbConn>> =
            Extension::<Arc<DbConn>>::from_request(req)
                .await
                .server_error("`DbConn` extension missing")?;
        let site = config::get_config_temp().site().clone();

        let post_and_comments = Post::find_and_commit(&*db, post_id)
            .await?
            .ok_or_else(|| {
                HttpError::from_const(
                    StatusCode::NOT_FOUND,
                    "post not found",
                )
            })?;

        Ok(Data {
            site,
            logged: matches!(login_status, LoginStatus::Logged),
            comments: post_and_comments
                .1
                .into_iter()
                .map(|comment| (comment.id, comment))
                .collect(),
            post: post_and_comments.0,
        })
    }
}
