use std::collections::BTreeMap;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Extension, FromRequest, RequestParts};
use axum::handler::get;
use axum::response::Html;
use axum::routing::BoxRoute;
use axum::{extract, Json, Router};
use rbatis::rbatis::Rbatis;

use config::SiteConfig;
use database::models::comment::Comments;
use database::models::post::Posts;

use crate::login_status::LoginStatus;
use crate::error::HttpError;
use crate::error::HttpServerError;

pub fn routes() -> Router<BoxRoute> {
    let index = match config::get_config().render().default_render() {
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

pub async fn index_ssr<'reg>(
    data: Data,
    Extension(tm): Extension<Arc<template::TemplateManager<'reg>>>,
) -> Result<Html<String>, HttpError> {
    tm.render("post", &data)
        .map(|s| Html(s))
        .map_err(Into::into)
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
    post: Posts,
    comments: BTreeMap<u64, Comments>
}

#[async_trait::async_trait]
impl FromRequest for Data {
    type Rejection = HttpError;

    async fn from_request(
        req: &mut RequestParts<Body>,
    ) -> Result<Self, Self::Rejection> {
        let login_status = LoginStatus::from_request(req).await?;
        let extract::Path(post_id) = extract::Path::<u64>::from_request(req).await?;
        let Extension(rb): Extension<Arc<Rbatis>> =
            Extension::<Arc<Rbatis>>::from_request(req)
                .await
                .server_error("`Rbatis` extension missing")?;
        let config_guard = config::get_config();
        let site = config_guard.site().clone();

        let post = Posts::select(&rb, post_id).await?;

        Ok(Data {
            site,
            logged: matches!(login_status, LoginStatus::Logged),
            comments: post.query_comments(&rb).await?.into_iter().map(|comment| (comment.id, comment)).collect(),
            post
        })
    }
}
