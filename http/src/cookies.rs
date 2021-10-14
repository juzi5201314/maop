use axum::extract::{FromRequest, RequestParts};
use headers::{Cookie, HeaderMapExt};

use crate::error::HttpError;

pub struct Cookies(pub Option<Cookie>);

#[async_trait::async_trait]
impl<B> FromRequest<B> for Cookies
where
    B: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request(
        req: &mut RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        Ok(Cookies(req.headers().unwrap().typed_try_get::<Cookie>()?))
    }
}
