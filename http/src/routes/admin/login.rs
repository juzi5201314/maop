use rocket::{post, get, State};
use crate::request::Request;

#[get("/login")]
pub async fn login_page(req: Request<'_>) -> crate::Result<'_> {
    todo!()
}

#[post("/login")]
pub async fn login(req: Request<'_>) -> crate::Result<'_> {
    todo!()
}
