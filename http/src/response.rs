use rocket::http::Status;
use rocket::response::Responder;
use rocket::Request;
use std::borrow::Cow;
use serde::Serialize;
use crate::api_format::{format_api, RespType};

utils::builder!(Response<'a, 'b>, {
    status: Status,
    body: Vec<u8>,
    headers: Vec<Header<'a, 'b>>
});

impl Response<'_, '_> {
    pub fn new() -> Self {
        Default::default()
    }

    /// utf-8 string
    pub fn text<S>(mut self, text: S) -> Self where S: AsRef<str> {
        self.body(text.as_ref().as_bytes().to_owned())
    }

    pub fn format<T>(mut self, data: &T, resp_type: RespType<'_>) -> anyhow::Result<Self> where T: Serialize {
        Ok(self.body(format_api(resp_type, data)?.body))
    }

    pub fn bincode<T>(mut self, data: &T) -> anyhow::Result<Self> where T: Serialize {
        bincode::serialize_into(&mut self.body, data)?;
        Ok(self)
    }

    pub fn urlencoded<T>(mut self, data: &T) -> anyhow::Result<Self> where T: Serialize {
        Ok(self.text(serde_urlencoded::to_string(data)?))
    }

    pub fn xml<T>(mut self, data: &T) -> anyhow::Result<Self> where T: Serialize {
        quick_xml::se::to_writer(&mut self.body, data)?;
        Ok(self)
    }

    pub fn msgpack<T>(mut self, data: &T) -> anyhow::Result<Self> where T: Serialize {
        rmp_serde::encode::write(&mut self.body, data)?;
        Ok(self)
    }

    pub fn json<T>(mut self, data: &T) -> anyhow::Result<Self> where T: Serialize {
        serde_json::to_writer(&mut self.body, data)?;
        Ok(self)
    }
}

impl<'r, 'o: 'r, 'h: 'o> Responder<'r, 'o> for Response<'h, 'h> {
    fn respond_to(
        self,
        _: &'r Request<'_>,
    ) -> rocket::response::Result<'o> {
        Ok({
            let mut builder = rocket::Response::build();
            for header in self.headers {
                builder.header(rocket::http::Header::new(
                    header.0, header.1,
                ));
            }

            builder
                .status(self.status)
                .sized_body(
                    self.body.len(),
                    std::io::Cursor::new(self.body),
                )
                .finalize()
        })
    }
}

#[derive(Debug)]
pub struct Header<'a, 'b>(Cow<'a, str>, Cow<'b, str>);
