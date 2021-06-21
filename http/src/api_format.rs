use crate::response::Response;
use serde::Serialize;

pub type RespType<'a> = &'a str;

pub macro check_api {
    ($req:expr, $data:expr) => {
        let base = $req.route.map(|uri| uri.uri.base());
        let resp_type = $req.uri.query().unwrap().segments().find_map(|(name, val)| if name == "resp_type" {
            Some(val)
        } else {
            None
        });
        match base {
            Some("api") => return $crate::api_format::format_api(resp_type, $data),
            _ => {}
        }
    }
}

pub fn format_api<'a, 'b, S>(
    resp_type: Option<RespType<'b>>,
    data: &'a S,
) -> crate::Result<'b>
where
    S: Serialize,
{
    Ok(match resp_type {
        None | Some("json") => Response::new().json(data),
        Some(other) => match other {
            "msgpack" => Response::new().msgpack(data),
            "bincode" => Response::new().bincode(data),
            "xml" => Response::new().xml(data),
            other => Err(anyhow::anyhow!(
                "{}: {}",
                utils::i18n!("errors.http.unsupported_resp_type"),
                other
            )),
        },
    }?)
}
