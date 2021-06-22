use rocket::{get, post, State};
use rocket::http::CookieJar;

use crate::api_format::{Api, RespType};
use crate::request::Request;
use crate::response::Response;

#[get("/login")]
pub async fn login_page(req: Request<'_>) -> crate::Result<'_> {
    todo!()
}

#[post("/login")]
pub async fn login<'a>(
    _api: Api,
    resp_t: RespType<'_>,
    cookies: &CookieJar<'_>,
) -> crate::Result<'a> {
    let pwd = cookies
        .get_private("password")
        .map(|c| c.value().to_owned());
    if let Some(pwd) = pwd {
        Ok(Response::new()
            .format(&LoginRespData { password: pwd }, resp_t)?)
    } else {
        todo!()
    }
}

#[derive(serde::Serialize)]
struct LoginReqData {}

#[derive(serde::Serialize)]
struct LoginRespData {
    password: String,
}
