use std::borrow::Cow;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use chrono::Local;
use fast_async_mutex::mutex::Mutex;
use flume::{unbounded, Receiver, Sender};
use futures::StreamExt;
use log::Metadata;
use once_cell::sync::Lazy;
use tokio::task::JoinHandle;

pub use crate::level::Level;
pub use crate::macros::*;

pub mod default_processors;
mod level;
mod macros;

static LOGGER: Lazy<Logger> = Lazy::new(Default::default);

pub fn init() {
    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(LOGGER.deref()).unwrap();
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        LOGGER.check_level(metadata.level().into())
    }

    fn log(&self, record: &log::Record) {
        let crate_name = record
            .module_path_static()
            .unwrap_or_default()
            .split_once(':')
            .map(|(name, _)| name)
            .unwrap_or_else(|| "Unknown");
        self.log(Record {
            content: record.args().to_string(),
            level: Level::from(record.metadata().level()),
            module_path: record
                .module_path_static()
                .unwrap_or_default(),
            file: Cow::Borrowed(
                record.file_static().unwrap_or_default(),
            ),
            line: record.line().unwrap_or_default(),
            time: chrono::Local::now(),
            crate_name,
        })
    }

    fn flush(&self) {}
}

#[inline]
pub fn _log(
    lvl: Level,
    content: String,
    module_path: &'static str,
    file: Cow<'static, str>,
    line: u32,
    crate_name: &'static str,
) {
    LOGGER.log(Record {
        content,
        level: lvl,
        module_path,
        file,
        line,
        time: Local::now(),
        crate_name,
    })
}

struct Logger {
    level: Level,
    processors: Mutex<
        Vec<
            Box<
                dyn Processor<Output = anyhow::Result<()>>
                    + Send
                    + Sync,
            >,
        >,
    >,

    task_processors_status: AtomicBool,
    task_processors_handler: Option<JoinHandle<()>>,
    sender: Sender<Arc<Record>>,
}

impl Logger {
    pub fn new(level: Level, sender: Sender<Arc<Record>>) -> Self {
        let logger = Logger {
            level,
            processors: Mutex::new(
                default_processors::default_processors(),
            ),
            task_processors_status: AtomicBool::new(false),
            task_processors_handler: None,
            sender,
        };
        logger
    }

    fn start(mut self, rx: Receiver<Arc<Record>>) -> Self {
        self.log(Record {
            content: utils::i18n!("start_msg.logger").to_owned(),
            level: Level::Info,
            ..Default::default()
        });
        self.task_processors_handler =
            Some(Logger::spawn_task_processors(rx));
        self
    }

    #[inline]
    fn check_level(&self, lvl: Level) -> bool {
        self.level <= lvl
    }

    #[inline]
    fn log(&self, mut record: Record) {
        let conf = config::get_config();
        if let Some(level) =
            conf.log().filter().get(record.crate_name)
        {
            if let Ok(lvl) = Level::from_str(level) {
                if record.level < lvl {
                    return;
                }
            }
        }
        if self.check_level(record.level) {
            self.sender
                .send(Arc::new(record))
                .map_err(|_| {
                    println!(
                        "{}",
                        utils::i18n!("errors.logger.send_failure")
                    )
                })
                .unwrap();
        }
    }

    fn spawn_task_processors(
        mut rx: Receiver<Arc<Record>>,
    ) -> JoinHandle<()> {
        tokio::task::spawn(async move {
            if !LOGGER.task_processors_status.load(Ordering::SeqCst) {
                LOGGER
                    .task_processors_status
                    .store(true, Ordering::SeqCst);
                while Self::process(&mut rx).await {}
                LOGGER
                    .task_processors_status
                    .store(false, Ordering::SeqCst);
            } else {
                panic!("Duplicate spawn_task_processors!")
            }
        })
    }

    async fn process(rx: &mut Receiver<Arc<Record>>) -> bool {
        if let Ok(record) = rx.recv_async().await {
            let mut lock = LOGGER.processors.lock().await;
            let mut futures = lock
                .iter_mut()
                .map(|ps| ps.process(Arc::clone(&record)))
                .collect::<futures::stream::FuturesUnordered<_>>();
            loop {
                match futures.next().await {
                    Some(Err(err)) => println!(
                        "{}: {}",
                        utils::i18n!("errors.logger.processor_error"),
                        err
                    ),
                    Some(Ok(_)) => {}
                    None => break,
                }
            }
            true
        } else {
            LOGGER
                .task_processors_status
                .store(false, Ordering::SeqCst);
            panic!("recv err")
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        Logger::new(
            std::env::var("LOG")
                .map(|s| {
                    Level::from_str(&s)
                        .map_err(|_| {
                            anyhow::anyhow!(
                                "{}: {}",
                                utils::i18n!(
                                    "errors.logger.nonexistent_level"
                                ),
                                s
                            )
                        })
                        .unwrap()
                })
                .unwrap_or_else(|_| {
                    if cfg!(debug_assertions) {
                        Level::Debug
                    } else {
                        Level::Info
                    }
                }),
            tx,
        )
        .start(rx)
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.task_processors_handler
            .iter_mut()
            .for_each(|h| h.abort());
    }
}

#[async_trait]
pub trait Processor {
    type Output;

    async fn process(&mut self, record: Arc<Record>) -> Self::Output;
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Record {
    content: String,
    level: Level,
    module_path: &'static str,
    file: Cow<'static, str>,
    line: u32,
    time: chrono::DateTime<Local>,
    crate_name: &'static str,
}

impl Default for Record {
    fn default() -> Self {
        Record {
            content: String::new(),
            level: Level::Debug,
            module_path: module_path!(),
            file: ::std::borrow::Cow::Borrowed(file!()),
            line: line!(),
            time: chrono::Local::now(),
            crate_name: crate___name::crate_name!(),
        }
    }
}

#[tokio::test]
async fn log_test() {
    for i in 0..10 {
        debug!(i);
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
