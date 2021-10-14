use std::borrow::Cow;
use std::sync::Arc;
use axum::body::{Bytes, Full};
use axum::extract::Extension;
use axum::http::{Response, StatusCode, Uri};
use crate::error::HttpError;

pub async fn assets<'reg>(uri: Uri, Extension(tm): Extension<Arc<template::TemplateManager<'reg>>>) -> Result<Response<Full<Bytes>>, HttpError> {
    let resp = Response::builder();
    Ok(if let Some(data) = tm.provider().0.get(uri.path().trim_start_matches("/")).await? {
        resp.status(StatusCode::OK).body(Full::from(data))?
    } else {
        resp.status(StatusCode::NOT_FOUND).body(Full::from(Cow::Borrowed(&[] as &'static [u8])))?
    })
}
