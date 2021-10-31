#![feature(result_flattening)]

use std::process::exit;

use futures::FutureExt;
use futures::TryFutureExt;
use tokio::runtime::Builder;

use global_resource::SHUTDOWN_NOTIFY;

#[cfg(feature = "prof")]
mod prof;

#[cfg(feature = "snmalloc")]
#[global_allocator]
static GLOBAL: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[cfg(all(feature = "tokio-console", not(tokio_unstable)))]
compile_error!("the `tokio_unstable` cfg must be enabled");

pub fn run(configs: Vec<String>, no_password: bool) {
    #[cfg(feature = "prof")]
    let guard = prof::start();

    config::init(configs.into_iter().map(|s| s.into()).collect())
        .expect("config error");

    let rt_config = config::get_config_temp().runtime().clone();

    let mut builder = Builder::new_multi_thread();

    builder.enable_all();

    if let Some(num) = rt_config.worker_threads() {
        builder.worker_threads(*num);
    }

    if let Some(bytes) = rt_config.thread_stack_size() {
        builder.thread_stack_size(bytes.get_bytes() as usize);
    }

    if let Some(time) = rt_config.blocking_thread_keep_alive() {
        builder.thread_keep_alive(*time.duration());
    }

    if let Some(num) = rt_config.max_blocking_threads() {
        builder.max_blocking_threads(*num);
    }

    let rt = builder.build().unwrap();

    let shutdown_timeout = *rt_config.shutdown_timeout().duration();

    rt.block_on(async move {
        logger::init();

        #[cfg(feature = "tokio-console")]
            {
                use tracing_subscriber::util::SubscriberInitExt;
                if *config::get_config_temp().log().level() == log::Level::Trace {
                    println!("tokio-console does not allow `trace` level");
                    exit(1);
                } else {
                    console_subscriber::build().try_init().ok();
                }
            }

        let join_handle = utils::task::spawn(async move {
            if no_password {
                log::warn!("you are running in no-password mode");
            }
            http::run_http_server(no_password).await
        }, "http server");

        tokio::select! (
            res = join_handle.map(|x| x.map_err(Into::into).flatten()) => res,
            res = tokio::signal::ctrl_c().map_err(Into::into) => res
        ).unwrap();

        log::info!("Shutting down...");

        if tokio::time::timeout(
            shutdown_timeout,
            SHUTDOWN_NOTIFY.notify(),
        )
        .await
        .is_err()
        {
            println!("shutdown timeout");
            exit(1);
        }
    });

    rt.shutdown_timeout(shutdown_timeout);

    #[cfg(feature = "prof")]
    prof::report(&guard);
}
