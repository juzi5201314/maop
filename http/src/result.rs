use rocket::http::Status;
use rocket::response::Responder;

use crate::response::Response;

#[derive(Debug)]
pub struct Error(anyhow::Error);

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error(e)
    }
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error(anyhow::Error::msg(s))
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error(anyhow::Error::msg(s))
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(
        self,
        req: &'r rocket::request::Request<'_>,
    ) -> rocket::response::Result<'o> {
        Response::new()
            .status(Status::InternalServerError)
            .body(self.0.to_string().as_bytes().to_vec())
            .respond_to(req)
    }
}
pub type Result<'a> = std::result::Result<Response<'a, 'a>, Error>;
