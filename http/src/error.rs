use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::Debug;

use axum::body::{Bytes, Full};
use axum::extract::rejection::PathParamsRejection;
use axum::http::Response;
use axum::response::IntoResponse;
use hyper::StatusCode;

pub struct HttpError {
    code: StatusCode,
    msg: Cow<'static, str>,
}

impl HttpError {
    pub const fn from_const(
        code: StatusCode,
        msg: &'static str,
    ) -> Self {
        HttpError {
            code,
            msg: Cow::Borrowed(msg),
        }
    }
}

impl IntoResponse for HttpError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        log::error!("{}", self.msg);
        Response::builder()
            .status(self.code)
            .body(Full::from(format!(
                r###"{{"err":"{}"}}"###,
                self.msg
            )))
            .unwrap()
    }
}

impl From<anyhow::Error> for HttpError {
    fn from(err: anyhow::Error) -> Self {
        HttpError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("{:?}", err).into(),
        }
    }
}

impl From<PathParamsRejection> for HttpError {
    fn from(ppr: PathParamsRejection) -> Self {
        let resp = ppr.into_response();
        HttpError {
            code: resp.status(),
            msg: Cow::Borrowed("PathParamsRejection"),
        }
    }
}

impl From<axum::http::Error> for HttpError {
    fn from(err: axum::http::Error) -> Self {
        HttpError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: err.to_string().into(),
        }
    }
}

impl From<headers::Error> for HttpError {
    fn from(err: headers::Error) -> Self {
        HttpError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: err.to_string().into(),
        }
    }
}

pub trait HttpServerError<T> {
    fn server_error<S>(self, msg: S) -> Result<T, HttpError>
    where
        S: AsRef<str>;
}

impl<T, E> HttpServerError<T> for Result<T, E>
where
    E: Debug,
{
    fn server_error<S>(self, msg: S) -> Result<T, HttpError>
    where
        S: AsRef<str>,
    {
        match self {
            Ok(t) => Ok(t),
            Err(err) => Err(HttpError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: Cow::Owned(format!(
                    "{}: {:?}",
                    msg.as_ref(),
                    err
                )),
            }),
        }
    }
}
