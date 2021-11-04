#![feature(decl_macro)]
#![feature(never_type)]
#![feature(try_blocks)]

use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;

use axum::handler::get;
use axum::{AddExtensionLayer, Router};
use cfg_if::cfg_if;
use inquire::error::InquireError;
use inquire::PasswordDisplayMode;

use global_resource::SHUTDOWN_NOTIFY;
use template::TemplateManager;

use crate::routes::auth::Password;
use crate::routes::{assets, auth, edit, index, post};
use crate::session_store::SessionStore;
use sea_orm::prelude::DbConn;

mod cookies;
mod error;
mod login_status;
mod routes;
mod session;
mod session_store;

pub async fn run_http_server(
    no_password: bool,
) -> anyhow::Result<()> {
    let full_config = config::get_config_full();
    let config = full_config.http();

    let password = if !no_password {
        require_password(full_config.data_path())?
    } else {
        None
    };

    let axum_app = Router::new()
        .nest("/", index::routes())
        .nest("/post/:id", post::routes())
        .nest("/assets", get(assets::assets))
        .nest("/edit", edit::routes_post())
        .nest("/edit/comment", edit::routes_comment())
        .nest("/auth", auth::routes())
        .layer(AddExtensionLayer::new(Arc::new(password)))
        .layer(AddExtensionLayer::new(Arc::new(
            TemplateManager::new()?,
        )))
        .layer(AddExtensionLayer::new(Arc::new(
            database::new().await? as DbConn,
        )))
        .layer(AddExtensionLayer::new(
            SessionStore::new(
                full_config.data_path().clone().join("sessions"),
            )
            .await?,
        ));

    match config.r#type() {
        config::ListenType::Uds => {cfg_if! {
            if #[cfg(target_family = "unix")] {
                use hyperlocal::UnixServerExt;
                let server = hyper::Server::bind_unix(Path::new(config.bind().as_str()))?
                    .serve(axum_app.into_make_service());
                log::info!("listen on unix://{}", config.bind());
                let graceful = server.with_graceful_shutdown(async {
                    let resp = SHUTDOWN_NOTIFY.register(5).await.wait().await;
                    log::debug!("http server shutdown");
                    resp.ready()
                });
                graceful.await
            } else {
                panic!("Unsupported. You cannot use unix sockets on non-Unix systems.")
            }
        }},
        config::ListenType::Http => {
            let addr = SocketAddr::new(
                IpAddr::from_str(config.bind())?,
                *config.port(),
            );
            let server = hyper::Server::bind(&addr)
                .serve(axum_app.into_make_service());

            log::info!("listen on http://{}", server.local_addr());

            let graceful = server.with_graceful_shutdown(async {
                let resp = SHUTDOWN_NOTIFY.register(5).await.wait().await;
                log::debug!("http server shutdown");
                resp.ready()
            });
            graceful.await
        }
    }.unwrap();

    Ok(())
}

pub const PASSWORD_FILE_NAME: &str = ".password";

fn require_password(data_path: &Path) -> anyhow::Result<Password> {
    let pwd_path = data_path.join(PASSWORD_FILE_NAME);
    Ok(if pwd_path.exists() {
        Some(read_to_string(pwd_path)?)
    } else {
        request_password_from_input(&pwd_path)?
    })
}

pub fn set_password(
    pwd_path: &Path,
    password: String,
) -> anyhow::Result<Password> {
    let pwd = utils::password_hash::password_hash(password)?;
    let mut file = OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .open(pwd_path)?;
    file.write_all(pwd.as_bytes())?;
    file.sync_all()?;
    Ok(Some(pwd))
}

fn request_password_from_input(
    pwd_path: &Path,
) -> anyhow::Result<Password> {
    loop {
        let res = inquire::Password::new("enter your password:")
            .with_display_mode(PasswordDisplayMode::Masked)
            .prompt();

        match res {
            Ok(pwd) => {
                if pwd.is_empty() {
                    println!("the password cannot be empty");
                    continue;
                }

                break Ok(set_password(pwd_path, pwd)?);
            }
            Err(err) => match err {
                InquireError::OperationCanceled => continue,
                InquireError::OperationInterrupted => exit(0),
                other => break Err(other.into()),
            },
        }
    }
}

#[ignore]
#[tokio::test]
async fn test_http_server() {
    config::init(vec![]).unwrap();
    logger::init();
    run_http_server(false).await.unwrap();
}
