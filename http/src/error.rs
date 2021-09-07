use hyper::StatusCode;
use std::fmt::Debug;

pub trait HttpServerError<T> {
    fn server_error<S>(self, msg: S) -> Result<T, (StatusCode, String)> where S: AsRef<str>;
}

impl<T, E> HttpServerError<T> for Result<T, E> where E: Debug {
    fn server_error<S>(self, msg: S) -> Result<T, (StatusCode, String)> where S: AsRef<str> {
        match self {
            Ok(t) => Ok(t),
            Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {:?}", msg.as_ref(), err)))
        }
    }
}
