
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
    logger::init();
    let db = Arc::new(Database::new().await?);

    http::run_http_server(Arc::clone(&db)).await?;
    Ok(())
}
