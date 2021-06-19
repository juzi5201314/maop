use crate::response::Response;
use serde::Serialize;

pub type RespType<'a> = &'a str;

pub fn format_api<S>(
    resp_type: Option<RespType<'_>>,
    data: S,
) -> crate::Result<'_>
where
    S: Serialize,
{
    Ok(match resp_type {
        None | Some("json") => Response::new().json(&data),
        Some(other) => match other {
            "msgpack" => Response::new().msgpack(&data),
            "bincode" => Response::new().bincode(&data),
            "xml" => Response::new().xml(&data),
            other => Err(anyhow::anyhow!(
                "{}: {}",
                utils::i18n!("errors.http.unsupported_resp_type"),
                other
            )),
        },
    }?)
}
