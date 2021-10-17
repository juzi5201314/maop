#![feature(decl_macro)]
#![feature(never_type)]

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use axum::handler::get;
use axum::{AddExtensionLayer, Router};
use cfg_if::cfg_if;

use ::error::Error;
use template::TemplateManager;
use utils::SHUTDOWN_NOTIFY;

use crate::routes::auth::Password;
use crate::routes::{assets, auth, edit, index, post};
use crate::session_store::SessionStore;

mod cookies;
mod error;
mod login_status;
mod routes;
mod session;
mod session_store;

pub async fn run_http_server() -> Result<(), Error> {
    let full_config = config::get_config_full();
    let config = full_config.http();

    let axum_app = Router::new()
        .nest("/", index::routes())
        .nest("/post/:id", post::routes())
        .nest("/assets", get(assets::assets))
        .nest("/edit", edit::routes_post())
        .nest("/edit/comment", edit::routes_comment())
        .nest("/auth", auth::routes())
        .layer(AddExtensionLayer::new(Arc::new(
            config.password().clone() as Password,
        )))
        .layer(AddExtensionLayer::new(Arc::new(
            TemplateManager::new()?,
        )))
        .layer(AddExtensionLayer::new(Arc::new(
            database::new().await?,
        )))
        .layer(AddExtensionLayer::new(
            SessionStore::new(
                full_config.data_path().clone().join("session.data"),
            )
            .await?,
        ));

    match config.r#type() {
        config::ListenType::Uds => {cfg_if! {
            if #[cfg(target_os = "unix")] {
                use hyperlocal::UnixServerExt;
                let server = hyper::Server::bind_unix(config.bind())?
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

#[tokio::test]
async fn test_http_server() {
    config::init(vec![]).unwrap();
    logger::init();
    run_http_server().await.unwrap();
}
