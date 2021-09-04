use hyper::StatusCode;

pub trait HttpServerError {
    fn server_error<S>(self, msg: S) -> (StatusCode, String) where S: AsRef<str>;
}

impl<T, E> HttpServerError for Result<T, E> {
    fn server_error<S>(self, msg: S) -> (StatusCode, String) where S: AsRef<str> {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {:?}", msg, self))
    }
}
