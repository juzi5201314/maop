use std::ops::Deref;
use std::sync::Arc;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{get, State, Route};

use data_structure::website::WebsiteInfo;
use data_structure::wraps::Posts;
use database::Database;

use crate::api_format::check_api;
use crate::request::Request;
use crate::response::Response;

#[get("/")]
pub async fn index_page(
    req: Request<'_>,
    data: IndexPageData,
) -> crate::Result<'_> {
    check_api!(req, &data);
    Ok(Response::new().text("hello world"))
}

#[derive(serde::Serialize)]
pub struct IndexPageData {
    website_info: WebsiteInfo,
    posts: Posts,
}

impl IndexPageData {
    pub async fn new(db: &Database) -> anyhow::Result<Self> {
        Ok(IndexPageData {
            website_info: WebsiteInfo::new()?,
            posts: Posts::get(db).await?,
        })
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IndexPageData {
    type Error = crate::result::Error;

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> Outcome<Self, Self::Error> {
        let db = crate::try_outcome!(request.guard::<&'r State<Arc<Database>>>().await, "");
        crate::try_outcome!(IndexPageData::new(db).await)
    }
}
