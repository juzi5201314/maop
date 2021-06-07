use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Local;
use fast_async_mutex::mutex::Mutex;
use flume::{unbounded, Receiver, Sender};
use futures::StreamExt;
use once_cell::sync::Lazy;
use tokio::task::JoinHandle;

pub use crate::level::Level;
pub use crate::macros::*;

pub mod default_processors;
mod level;
mod macros;

static LOGGER: Lazy<Logger> = Lazy::new(Default::default);

#[inline]
pub fn _log(
    lvl: Level,
    content: String,
    module_path: &'static str,
    file: Cow<'static, str>,
    line: u32,
) {
    if LOGGER.check_level(lvl) {
        LOGGER.log(Record {
            content,
            level: lvl,
            module_path,
            file,
            line,
            time: Local::now(),
        })
    }
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
        self.task_processors_handler =
            Some(Logger::spawn_task_processors(rx));
        self
    }

    #[inline]
    fn check_level(&self, lvl: Level) -> bool {
        self.level <= lvl
    }

    #[inline]
    fn log(&self, record: Record) {
        self.sender
            .send(Arc::new(record))
            .map_err(|_| {
                println!(
                    "{}",
                    utils::i18n!("errors.log.send_failure")
                )
            })
            .unwrap();
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
                    Some(Ok(_)) => {},
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
            #[cfg(debug_assertions)]
            Level::Debug,
            #[cfg(not(debug_assertions))]
            std::env::var("LOG")
                .map(|s| Level::from(s))
                .unwrap_or_else(|_| Level::Info),
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
}

#[tokio::test]
async fn log_test() {
    for i in 0..10 {
        debug!(i);
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
}
