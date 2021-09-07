#![feature(decl_macro)]

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use async_session::MemoryStore;
use axum::handler::{get, post};
use axum::{AddExtensionLayer, Router};
use tokio::net::TcpListener;

use ::error::Error;

use crate::routes::index::index;
use crate::routes::login::{login, logout};

mod auth;
mod error;
mod routes;

pub async fn run_http_server() -> Result<(), Error> {
    let conf_guard = config::get_config();
    let config = conf_guard.http();

    let axum_app = Router::new()
        .route("/", get(index))
        .route("/auth", post(login).delete(logout))
        .layer(AddExtensionLayer::new(config.clone()))
        // todo: 持久化
        .layer(AddExtensionLayer::new(MemoryStore::new()));

    if config.r#type() == "unix" {
        #[cfg(target_os = "unix")]
        {
            use hyperlocal::UnixServerExt;
            let server = hyper::Server::bind_unix(config.bind())?
                .serve(axum_app.into_make_service());
            log::info!("listen on unix://{}", config.bind());

            let graceful = server.with_graceful_shutdown(async {
                tokio::signal::ctrl_c().await.unwrap();
            });
            graceful.await
        }
        #[cfg(not(target_os = "unix"))]
        {
            panic!("Unsupported. You cannot use unix sockets on non-Unix systems.")
        }
    } else {
        let addr = SocketAddr::new(
            IpAddr::from_str(config.bind())?,
            *config.port(),
        );
        let server = hyper::Server::bind(&addr)
            .serve(axum_app.into_make_service());

        log::info!("listen on http://{}", server.local_addr());

        let graceful = server.with_graceful_shutdown(async {
                tokio::signal::ctrl_c().await.unwrap();
            });
        graceful.await
    }.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_http_server() {
    run_http_server().await.unwrap();
}
