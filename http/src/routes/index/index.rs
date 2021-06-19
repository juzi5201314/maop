use std::ops::Deref;
use std::sync::Arc;

use data_structure::http::IndexData;
use database::Database;

use rocket::{get, State};

use crate::api_format::format_api;
use crate::response::Response;
use crate::request::Request;
use rocket::http::Status;

#[get("/")]
pub async fn index<'a>(req: Request<'a>, db: &'a State<Arc<Database>>) -> crate::Result<'a> {
    let data = IndexData::new(db).await?;

    Ok(Response::new()
        .json(&data)?)
}

/*#[get("/data")]
pub async fn index_data<'a>(
    accept: AcceptHeaderOnce<'_>,
    db: &'a State<Arc<Database>>,
) -> crate::error::Result<Response<'a>> {
    let ct = accept.unwrap_or("application/json").split(',').next().unwrap();
    let data = IndexData::new(db).await?;
    let data = format_api(ct, &data)?;
    Ok(Response::new(rocket::Response::build()
        .header(Header::new("Content-Type", ct.to_owned()))
        .sized_body(data.len(), std::io::Cursor::new(data))
        .finalize()))
}*/
