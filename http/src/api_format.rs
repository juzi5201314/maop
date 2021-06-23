use crate::request::Request;
use crate::response::Response;
use rocket::request::{FromRequest, Outcome};
use serde::Serialize;
use std::fmt::Debug;

pub macro check_api($req:expr, $data:expr) {
    let base = $req.route.map(|uri| uri.uri.base());
    match base {
        Some("api") => {
            return $crate::api_format::format_api(
                $crate::api_format::RespType::from_request2($req),
                $data,
            )
        }
        _ => {}
    }
}

pub struct RespType<'a>(&'a str);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for RespType<'a> {
    type Error = ();

    async fn from_request(
        request: &'a rocket::Request<'_>,
    ) -> Outcome<Self, Self::Error> {
        Outcome::Success(RespType(
            request
                .uri()
                .query()
                .and_then(|q| {
                    q.segments().find_map(|(name, val)| {
                        if name == "resp_type" {
                            Some(val)
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or("json"),
        ))
    }
}

pub struct Api;

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Api {
    type Error = ();

    async fn from_request(
        request: &'a rocket::Request<'_>,
    ) -> Outcome<Self, Self::Error> {
        dbg!(request.route());
        match request.route().and_then(|route| {
            (route.uri.base() == "/api").then(|| ())
        }) {
            Some(()) => Outcome::Success(Api),
            None => Outcome::Forward(()),
        }
    }
}

pub fn format_api<'a, 'b, S>(
    resp_type: RespType<'b>,
    data: &'a S,
) -> crate::Result<'b>
where
    S: Serialize,
{
    Ok(match resp_type.0 {
        "json" => Response::new().json(data),
        "msgpack" => Response::new().msgpack(data),
        "bincode" => Response::new().bincode(data),
        "xml" => Response::new().xml(data),
        other => Err(anyhow::anyhow!(
            "{}: {}",
            utils::i18n!("errors.http.unsupported_resp_type"),
            other
        )),
    }?)
}
