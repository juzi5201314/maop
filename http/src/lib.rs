#![feature(decl_macro)]

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use axum::handler::get;
use axum::Router;
use tokio::net::TcpListener;

use error::Error;

use crate::routes::index::index;

mod routes;

pub async fn run_http_server() -> Result<(), Error> {
    let conf_guard = config::get_config();
    let config = conf_guard.http();

    let axum_app = Router::new().route("/", get(index));

    if config.r#type() == "unix" {
        #[cfg(target_os = "unix")]
        {
            use hyperlocal::UnixServerExt;
            hyper::Server::bind_unix(config.bind())?
                .serve(axum_app.into_make_service())
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
        hyper::Server::bind(&addr)
            .serve(axum_app.into_make_service())
    };

    Ok(())
}

#[tokio::test]
async fn test_http_server() {
    run_http_server().await;
}
