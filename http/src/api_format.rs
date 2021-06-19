use serde::Serialize;

pub fn format_api<S>(content_type: &str, data: &S) -> anyhow::Result<Vec<u8>> where S: Serialize {
    Ok(match content_type {
        "application/json" => serde_json::to_vec(data)?,
        "application/msgpack"
        | "application/x-msgpack"
        | "application/x.msgpack" => rmp_serde::to_vec(data)?,
        // bincode does not have a certain MIME type
        "application/bincode" => bincode::serialize(data)?,
        "application/x-www-form-urlencoded" => serde_urlencoded::to_string(data)?.as_bytes().to_vec(),
        "application/xml"
        | "text/xml" => quick_xml::se::to_string(data)?.as_bytes().to_vec(),
        other => return Err(anyhow::anyhow!("{}: {}", utils::i18n!("errors.http.unsupported_mime_type"), other))
    })
}
