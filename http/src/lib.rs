use std::net::ToSocketAddrs;
use std::ops::Deref;
use std::sync::Arc;

use rocket::config::SecretKey;
use rocket::data::ByteUnit;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use database::Database;

use crate::routes::index;

mod api_format;
mod request;
mod response;
mod result;
mod routes;
mod session;

use result::Result;

#[macro_export]
macro_rules! try_outcome {
    ($result:expr) => {
        match $result {
            Ok(o) => rocket::request::Outcome::Success(o),
            Err(e) => rocket::request::Outcome::Failure((
                rocket::http::Status::InternalServerError,
                crate::result::Error::from(e.to_string()),
            )),
        }
    };

    ($outcome:expr, $msg:expr) => {
        match $outcome {
            rocket::request::Outcome::Success(s) => s,
            rocket::request::Outcome::Forward(f) => {
                return rocket::request::Outcome::Forward(f)
            }
            rocket::request::Outcome::Failure(_) => {
                return rocket::request::Outcome::Failure((
                    rocket::http::Status::InternalServerError,
                    crate::result::Error::from($msg),
                ))
            }
        }
    };
}

pub async fn run_http_server(
    db: Arc<Database>,
) -> anyhow::Result<()> {
    let conf = config::get_config();
    let conf = conf.rocket();
    let mut rocket_config = rocket::Config::release_default();

    let secret_key_file =
        config::DATA_PATH.deref().join(".secret_key");
    let secret_key = if secret_key_file.exists() {
        let mut vec = Vec::with_capacity(64);
        OpenOptions::new()
            .read(true)
            .open(secret_key_file)
            .await?
            .read_to_end(&mut vec)
            .await?;
        vec
    } else {
        let key = cookie::Key::generate();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(secret_key_file)
            .await?;
        file.write_all(key.master()).await?;
        file.sync_all().await?;
        key.master().to_vec()
    };

    rocket_config.secret_key = SecretKey::from(&secret_key);
    rocket_config.address = conf.addr().clone();
    rocket_config.port = *conf.port();
    if let Some(num) = conf.workers() {
        rocket_config.workers = *num;
    }
    if let Some(u) = conf.keep_alive() {
        rocket_config.keep_alive = *u;
    }
    if let Some(limits) = conf.limits() {
        for (name, limit) in limits.iter() {
            rocket_config.limits = rocket_config.limits.limit(
                name.clone(),
                ByteUnit::from(limit.get_bytes()),
            );
        }
    }
    rocket_config.ident =
        rocket::config::Ident::try_new("Maop").unwrap();

    rocket::Rocket::build()
        .manage(db)
        .mount("/", routes::index::routes())
        .mount("/api", routes::index::routes())
        .mount("/admin", routes::admin::routes())
        .configure(rocket_config)
        .ignite()
        .await?
        .launch()
        .await?;
    Ok(())
}
