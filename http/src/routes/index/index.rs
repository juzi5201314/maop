use std::ops::Deref;
use std::sync::Arc;

use rocket::{get, State};
use rocket::http::Status;

use data_structure::http::IndexData;
use database::Database;

use crate::api_format::{format_api, RespType};
use crate::request::Request;
use crate::response::Response;

#[get("/?<resp_type>")]
pub async fn index<'a>(resp_type: Option<RespType<'a>>, req: Request<'a>, db: &'a State<Arc<Database>>) -> crate::Result<'a> {
    let data = IndexData::new(db).await?;

    let base = req.route.map(|r| r.uri.base());
    match base {
        Some("/api") => index_api(resp_type, data).await,
        _ => {
            Ok(Response::new()
                .text("hello world"))
        }
    }
}

pub async fn index_api(resp_type: Option<RespType<'_>>, data: IndexData) -> crate::Result<'_> {
    format_api(resp_type, data)
}
