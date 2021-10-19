use tokio::runtime::Builder;

use utils::SHUTDOWN_NOTIFY;

#[cfg(feature = "prof")]
mod prof;

#[cfg(feature = "snmalloc")]
#[global_allocator]
static GLOBAL: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

pub fn run(configs: Vec<String>, no_password: bool) {
    #[cfg(feature = "prof")]
    let guard = prof::start();
    config::init(configs.into_iter().map(|s| s.into()).collect())
        .expect("config error");

    let config = config::get_config_temp().runtime().clone();

    let mut builder = Builder::new_multi_thread();

    builder.enable_all();

    if let Some(num) = config.worker_threads() {
        builder.worker_threads(*num);
    }

    if let Some(bytes) = config.thread_stack_size() {
        builder.thread_stack_size(bytes.get_bytes() as usize);
    }

    if let Some(time) = config.blocking_thread_keep_alive() {
        builder.thread_keep_alive(*time.duration());
    }

    if let Some(num) = config.max_blocking_threads() {
        builder.max_blocking_threads(*num);
    }

    let rt = builder.build().unwrap();

    let shutdown_timeout = *config.shutdown_timeout().duration();

    rt.block_on(async move {
        logger::init();

        tokio::spawn(async move {
            if no_password {
                log::warn!("you are running in no-password mode");
            }
            http::run_http_server(no_password)
                .await
                .expect("http server error");
        });

        tokio::signal::ctrl_c().await.unwrap();

        log::info!("Shutting down...");

        if tokio::time::timeout(
            shutdown_timeout,
            SHUTDOWN_NOTIFY.notify(),
        )
        .await
        .is_err()
        {
            eprintln!("shutdown timeout");
        }
    });

    rt.shutdown_timeout(shutdown_timeout);

    #[cfg(feature = "prof")]
    prof::report(&guard);
}
