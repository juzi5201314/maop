use rocket::http::Status;
use rocket::response::Responder;
use rocket::Request;
use std::borrow::Cow;
use serde::Serialize;

utils::builder!(Response<'a, 'b>, {
    status: Status,
    body: Vec<u8>,
    headers: Vec<Header<'a, 'b>>
});

impl Response<'_, '_> {
    pub fn new() -> Self {
        Default::default()
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
