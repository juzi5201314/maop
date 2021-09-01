#[cfg(feature = "prof")]
mod prof;

use once_cell::sync::Lazy;

use config::CONFIG;
use std::sync::Arc;

pub async fn init() {
    Lazy::force(&CONFIG);
}

pub async fn run() -> anyhow::Result<()> {
    #[cfg(feature = "prof")]
    let guard = prof::start();

    logger::init();

    let conf = config::get_config();
    let db = Arc::new(database::new(conf.database()).await?);

    http::generate_password_if_no_exists()?;
    http::run_http_server(Arc::clone(&db)).await?;

    #[cfg(feature = "prof")]
    prof::report(&guard);
    Ok(())
}
