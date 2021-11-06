use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Extension, FromRequest, Query, RequestParts};
use axum::handler::{delete, get, post};
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::BoxRoute;
use axum::{extract, Json, Router};
use compact_str::CompactStr;
use sea_orm::prelude::DbConn;

use config::SiteConfig;
use database::models::comment::{Comment, CommentModel, NewComment};
use database::models::post::{NewPost, Post, PostModel};

use crate::error::HttpError;
use anyhow::Context;
use crate::login_status::Logged;
use utils::markdown::html_escape;

pub fn routes_post() -> Router<BoxRoute> {
    let router = Router::new()
        .route(
            "/:id",
            get(index_ssr_edit_post)
                .post(update_post)
                .delete(delete_post),
        )
        .route("/", get(index_ssr_new_post).post(new_post))
        .route("/:id/api", get(index_api_edit_post))
        .route("/api", get(index_api_new_post));

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
    title: String,
    content: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdatePostData {
    title: Option<String>,
    content: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PostRes {
    id: u32,
}

async fn update_post(
    _: Logged,
    extract::Path(post_id): extract::Path<u32>,
    Json(data): Json<UpdatePostData>,
    Extension(db): Extension<Arc<DbConn>>,
) -> Result<Json<PostRes>, HttpError> {
    Post::update(
        &*db,
        post_id,
        data.title,
        data.content,
    )
    .await?;
    Ok(Json(PostRes { id: post_id }))
}

async fn new_post(
    _: Logged,
    Json(data): Json<PostData>,
    Extension(db): Extension<Arc<DbConn>>,
) -> Result<Json<PostRes>, HttpError> {
    let post_id = Post::insert(
        &*db,
        NewPost {
            title: data.title,
            content: data.content,
        },
    )
    .await?;
    Ok(Json(PostRes { id: post_id }))
}

async fn delete_post(
    _: Logged,
    extract::Path(post_id): extract::Path<u32>,
    Extension(db): Extension<Arc<DbConn>>,
) -> Result<Json<PostRes>, HttpError> {
    Post::delete(&*db, post_id).await?;
    Ok(Json(PostRes { id: post_id }))
}

#[allow(clippy::needless_lifetimes)]
pub async fn index_ssr_new_post<'reg>(
    _: Logged,
    data: NewPostData,
    Extension(tm): Extension<Arc<template::TemplateManager<'reg>>>,
) -> Result<Html<String>, HttpError> {
    tm.render("edit", &data).map(Html).map_err(Into::into)
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
    tm.render("edit", &data).map(Html).map_err(Into::into)
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
    post: PostModel,
    comments: BTreeMap<u32, CommentModel>,
}

#[async_trait::async_trait]
impl FromRequest for EditPostData {
    type Rejection = HttpError;

    async fn from_request(
        req: &mut RequestParts<Body>,
    ) -> Result<Self, Self::Rejection> {
        Logged::from_request(req).await?;
        let extract::Path(post_id) =
            extract::Path::<u32>::from_request(req).await?;
        let Extension(db): Extension<Arc<DbConn>> =
            Extension::from_request(req)
                .await
                .context("`DbConn` extension missing")?;
        let site = config::get_config_temp().site().clone();

        let post_and_comments = Post::find_and_commit(&*db, post_id)
            .await?
            .ok_or_else(|| {
                HttpError::from_const(
                    StatusCode::NOT_FOUND,
                    "post not found",
                )
            })?;

        Ok(EditPostData {
            site,
            comments: post_and_comments
                .1
                .into_iter()
                .map(|comment| (comment.id, comment))
                .collect(),
            post: post_and_comments.0,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommentData {
    post_id: u32,
    reply_to: Option<u32>,
    nickname: CompactStr,
    email: CompactStr,
    content: CompactStr,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommentRes {
    id: u32,
}

async fn new_comment(
    Json(data): Json<CommentData>,
    Extension(db): Extension<Arc<DbConn>>,
) -> Result<Json<CommentRes>, HttpError> {
    let comment_id = Comment::insert(
        &*db,
        data.post_id,
        NewComment {
            content: html_escape(&data.content),
            nickname: html_escape(&data.nickname.to_string()),
            email: html_escape(&data.email.to_string()),
        },
        data.reply_to,
    )
    .await?;
    Ok(Json(CommentRes { id: comment_id }))
}

async fn delete_comment(
    _: Logged,
    extract::Path(comment_id): extract::Path<u32>,
    Query(params): Query<HashMap<CompactStr, CompactStr>>,
    Extension(db): Extension<Arc<DbConn>>,
) -> Result<Json<CommentRes>, HttpError> {
    if params.get("hard").is_some() {
        Comment::hard_delete(&*db, comment_id).await?;
    } else {
        Comment::soft_delete(&*db, comment_id).await?;
    }
    Ok(Json(CommentRes { id: comment_id }))
}
