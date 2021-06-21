use std::ops::Deref;
use std::sync::Arc;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{get, State, Route};

use data_structure::website::WebsiteInfo;
use data_structure::wraps::Posts;
use database::Database;

use crate::api_format::{format_api, RespType};
use crate::request::Request;
use crate::response::Response;

#[get("/?<resp_type>")]
pub async fn index<'a>(
    resp_type: Option<RespType<'a>>,
    route: &'a Route,
    data: IndexData,
) -> crate::Result<'a> {
    let base = route.uri.base();
    match base {
        "/api" => index_api(resp_type, data).await,
        _ => Ok(Response::new().text("hello world")),
    }
}

pub async fn index_api(
    resp_type: Option<RespType<'_>>,
    data: IndexData,
) -> crate::Result<'_> {
    format_api(resp_type, &data)
}

#[derive(serde::Serialize)]
pub struct IndexData {
    website_info: WebsiteInfo,
    posts: Posts,
}

impl IndexData {
    pub async fn new(db: &Database) -> anyhow::Result<Self> {
        Ok(IndexData {
            website_info: WebsiteInfo::new()?,
            posts: Posts::get(db).await?,
        })
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IndexData {
    type Error = crate::result::Error;

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> Outcome<Self, Self::Error> {
        let db = crate::try_outcome!(request.guard::<&'r State<Arc<Database>>>().await, "");
        crate::try_outcome!(IndexData::new(db).await)
    }
}
