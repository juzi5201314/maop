use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Extension, FromRequest, Query, RequestParts};
use axum::handler::{delete, get, post};
use axum::response::Html;
use axum::routing::BoxRoute;
use axum::{extract, Json, Router};
use compact_str::CompactStr;
use rbatis::rbatis::Rbatis;

use config::SiteConfig;
use database::models::comment::{Comments, NewComment};
use database::models::post::{NewPost, Posts};

use crate::error::HttpError;
use crate::error::HttpServerError;
use crate::login_status::Logged;

pub fn routes_post() -> Router<BoxRoute> {
    let index = match config::get_config_temp().render().default_render() {
        config::RenderStrategy::SSR => Router::new()
            .route(
                "/:id",
                get(index_ssr_edit_post)
                    .post(update_post)
                    .delete(delete_post),
            )
            .route("/", get(index_ssr_new_post).post(new_post))
            .boxed(),
        config::RenderStrategy::CSR => Router::new()
            .route(
                "/:id",
                get(index_csr_edit_post)
                    .post(update_post)
                    .delete(delete_post),
            )
            .route("/", get(index_csr_new_post).post(new_post))
            .boxed(),
    };

    let router = index
        .route("/:id/api", get(index_api_edit_post))
        .route("/:id/ssr", get(index_ssr_edit_post))
        .route("/:id/csr", get(index_csr_edit_post))
        .route("/api", get(index_api_new_post))
        .route("/ssr", get(index_ssr_new_post))
        .route("/csr", get(index_csr_new_post));

    router.boxed()
}

pub fn routes_comment() -> Router<BoxRoute> {
    let router = Router::new()
        .route("/", post(new_comment))
        .route("/:id", delete(delete_comment));

    router.boxed()
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PostData {
    title: CompactStr,
    content: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PostRes {
    id: u64,
}

async fn update_post(
    _: Logged,
    extract::Path(post_id): extract::Path<u64>,
    Json(data): Json<PostData>,
    Extension(rb): Extension<Arc<Rbatis>>,
) -> Result<Json<PostRes>, HttpError> {
    Posts::update_by_id(
        &*rb,
        post_id,
        NewPost {
            title: data.title,
            content: utils::markdown::render(&data.content).map_err(error::Error::Io)?,
        },
    )
    .await?;
    Ok(Json(PostRes { id: post_id }))
}

async fn new_post(
    _: Logged,
    Json(data): Json<PostData>,
    Extension(rb): Extension<Arc<Rbatis>>,
) -> Result<Json<PostRes>, HttpError> {
    let post = Posts::insert(
        &*rb,
        NewPost {
            title: data.title,
            content: utils::markdown::render(&data.content).map_err(error::Error::Io)?,
        },
    )
    .await?;
    Ok(Json(PostRes { id: post.id }))
}

async fn delete_post(
    _: Logged,
    extract::Path(post_id): extract::Path<u64>,
    Extension(rb): Extension<Arc<Rbatis>>,
) -> Result<Json<PostRes>, HttpError> {
    Posts::remove_by_id(&*rb, post_id).await?;
    Ok(Json(PostRes { id: post_id }))
}

#[allow(clippy::needless_lifetimes)]
pub async fn index_ssr_new_post<'reg>(
    _: Logged,
    data: NewPostData,
    Extension(tm): Extension<Arc<template::TemplateManager<'reg>>>,
) -> Result<Html<String>, HttpError> {
    tm.render("edit", &data)
        .map(Html)
        .map_err(Into::into)
}

pub async fn index_csr_new_post() -> &'static str {
    "Hello, World!"
}

pub async fn index_api_new_post(
    _: Logged,
    data: NewPostData,
) -> Result<Json<NewPostData>, HttpError> {
    Ok(Json(data))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NewPostData {
    site: SiteConfig,
}

#[async_trait::async_trait]
impl FromRequest for NewPostData {
    type Rejection = HttpError;

    async fn from_request(
        req: &mut RequestParts<Body>,
    ) -> Result<Self, Self::Rejection> {
        Logged::from_request(req).await?;
        let site = config::get_config_temp().site().clone();

        Ok(NewPostData { site })
    }
}

#[allow(clippy::needless_lifetimes)]
pub async fn index_ssr_edit_post<'reg>(
    _: Logged,
    data: EditPostData,
    Extension(tm): Extension<Arc<template::TemplateManager<'reg>>>,
) -> Result<Html<String>, HttpError> {
    tm.render("edit", &data)
        .map(Html)
        .map_err(Into::into)
}

pub async fn index_csr_edit_post() -> &'static str {
    "Hello, World!"
}

pub async fn index_api_edit_post(
    _: Logged,
    data: EditPostData,
) -> Result<Json<EditPostData>, HttpError> {
    Ok(Json(data))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct EditPostData {
    site: SiteConfig,
    post: Posts,
    comments: BTreeMap<u64, Comments>,
}

#[async_trait::async_trait]
impl FromRequest for EditPostData {
    type Rejection = HttpError;

    async fn from_request(
        req: &mut RequestParts<Body>,
    ) -> Result<Self, Self::Rejection> {
        Logged::from_request(req).await?;
        let extract::Path(post_id) =
            extract::Path::<u64>::from_request(req).await?;
        let Extension(rb): Extension<Arc<Rbatis>> =
            Extension::<Arc<Rbatis>>::from_request(req)
                .await
                .server_error("`Rbatis` extension missing")?;
        let site = config::get_config_temp().site().clone();

        let post = Posts::select(&rb, post_id).await?;

        Ok(EditPostData {
            site,
            comments: post
                .query_comments(&rb)
                .await?
                .into_iter()
                .map(|comment| (comment.id, comment))
                .collect(),
            post,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommentData {
    post_id: u64,
    reply_to: Option<u64>,
    nickname: CompactStr,
    email: CompactStr,
    content: CompactStr,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommentRes {
    id: u64,
}

async fn new_comment(
    Json(data): Json<CommentData>,
    Extension(rb): Extension<Arc<Rbatis>>,
) -> Result<Json<CommentRes>, HttpError> {
    let comment = Comments::insert(
        &*rb,
        data.post_id,
        NewComment {
            content: utils::markdown::render_safe(&data.content).map_err(error::Error::Io)?,
            nickname: data.nickname,
            email: data.email,
        },
        data.reply_to,
    )
    .await?;
    Ok(Json(CommentRes { id: comment.id }))
}

async fn delete_comment(
    _: Logged,
    extract::Path(comment_id): extract::Path<u64>,
    Query(params): Query<HashMap<CompactStr, CompactStr>>,
    Extension(rb): Extension<Arc<Rbatis>>,
) -> Result<Json<CommentRes>, HttpError> {
    if params.get("hard").is_some() {
        Comments::hard_delete(&*rb, comment_id).await?;
    } else {
        Comments::soft_delete(&*rb, comment_id).await?;
    }
    Ok(Json(CommentRes { id: comment_id }))
}
