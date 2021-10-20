use std::borrow::Borrow;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::ops::Deref;

use rocket::http::uri::Origin;
use rocket::http::{
    Accept, ContentType, CookieJar, HeaderMap, Method,
};
use rocket::request::{FromRequest, Outcome};
use rocket::{Orbit, Request as RRequest, Rocket, Route};

pub struct Request<'a> {
    pub method: Method,
    pub uri: &'a Origin<'a>,
    pub headers: &'a HeaderMap<'a>,
    pub remote: Option<SocketAddr>,
    pub rocket: &'a Rocket<Orbit>,
    pub route: Option<&'a Route>,
    pub cookies: &'a CookieJar<'a>,
    pub accept: Option<&'a Accept>,
    pub content_type: Option<&'a ContentType>,
}

#[rocket::async_trait]
impl<'r: 'a, 'a> FromRequest<'r> for Request<'a> {
    type Error = ();

    async fn from_request(
        request: &'r RRequest<'_>,
    ) -> Outcome<Self, Self::Error> {
        Outcome::Success(Request {
            method: request.method(),
            uri: request.uri(),
            headers: request.headers(),
            remote: request.remote(),
            rocket: request.rocket(),
            route: request.route(),
            cookies: request.cookies(),
            accept: request.accept(),
            content_type: request.content_type(),
        })
    }
}
