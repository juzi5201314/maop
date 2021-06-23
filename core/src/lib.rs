#[cfg(feature = "prof")]
mod prof;

use once_cell::sync::Lazy;

use config::CONFIG;
use database::Database;
use std::sync::Arc;

#[cfg(any(
    all(feature = "use_jemalloc", feature = "use_mimalloc",),
    all(feature = "use_jemalloc", feature = "use_snmalloc",),
    all(feature = "use_mimalloc", feature = "use_snmalloc",),
))]
compile_error!(
    "Only one memory allocator can be used at the same time!"
);

#[cfg(all(target_env = "msvc", feature = "use_jemalloc"))]
compile_error!(
    "jemalloc does not support msvc targets. Please refer to https://github.com/gnzlbg/jemallocator#platform-support for available platforms."
);

#[cfg(all(not(target_env = "msvc"), feature = "use_jemalloc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(feature = "use_mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "use_snmalloc")]
#[global_allocator]
static GLOBAL: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

pub async fn init() {
    Lazy::force(&CONFIG);
}

pub async fn run() -> anyhow::Result<()> {
    #[cfg(feature = "prof")]
    let guard = prof::start();

    logger::init();
    let db = Arc::new(Database::new().await?);

    http::generate_password_if_no_exists()?;
    http::run_http_server(Arc::clone(&db)).await?;

    #[cfg(feature = "prof")]
    prof::report(&guard);
    Ok(())
}
