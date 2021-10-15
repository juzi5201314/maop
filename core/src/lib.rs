use std::time::Duration;
use tokio::runtime::Builder;

#[cfg(feature = "prof")]
mod prof;

#[cfg(feature = "snmalloc")]
#[global_allocator]
static GLOBAL: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

pub fn run(configs: Vec<String>) {
    #[cfg(feature = "prof")] let guard = prof::start();

    let rt = Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async move {
        config::init(configs.into_iter().map(|s| s.into()).collect()).expect("config error");
        logger::init();

        http::run_http_server().await.expect("http server error");
    });

    rt.shutdown_timeout(Duration::from_secs(10));

    #[cfg(feature = "prof")] prof::report(&guard);
}
