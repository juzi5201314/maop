#![feature(box_syntax)]

use std::borrow::Cow;
use std::fs::create_dir_all;
use std::ops::Deref;

use chrono::{DateTime, Local, NaiveDate, SecondsFormat};
use colored::{ColoredString, Colorize};
use compact_str::CompactStr;
use crossfire::mpsc;
use crossfire::mpsc::SharedSenderBRecvF;
use futures::FutureExt;
use log::Level;
use once_cell::sync::Lazy;
use tokio::fs::{File, OpenOptions};
use tokio::io::{stdout, AsyncWriteExt, Stdout};

use config::get_config_temp;
use utils::SHUTDOWN_NOTIFY;

static LOGGER: Lazy<Logger> = Lazy::new(Default::default);

/// 在这之前必须初始化config!!!
pub fn init() {
    set_max_level();
    log::set_logger(LOGGER.deref()).unwrap();

    config::hook(box set_max_level);
}

#[inline]
fn set_max_level() {
    log::set_max_level(
        config::get_config_temp().log().level().to_level_filter(),
    )
}

impl log::Log for Logger {
    #[inline]
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let config_guard = config::get_config_temp();
        let config = config_guard.log();

        config
            .filter()
            .get(metadata.target())
            .map(|lvl| *lvl >= metadata.level())
            .unwrap_or(true)
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let record = Record {
                metadata: Metadata {
                    level: record.metadata().level(),
                    target: CompactStr::from(
                        record.metadata().target(),
                    ),
                },
                content: {
                    if let Some(s) = record.args().as_str() {
                        Cow::Borrowed(s)
                    } else {
                        Cow::Owned(record.args().to_string())
                    }
                },
                file: CompactStr::from(
                    record.file().unwrap_or_default(),
                ),
                line: record.line().unwrap_or_default(),
                time: Local::now(),
            };
            self.sender.send(record).expect("failed to send log.");
        }
    }

    #[inline]
    fn flush(&self) {}
}

struct Logger {
    sender: mpsc::TxBlocking<Record, SharedSenderBRecvF>,
}

impl Logger {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::bounded_tx_blocking_rx_future(1024);
        Logger::start(rx);
        Logger { sender: tx }
    }

    fn start(rx: mpsc::RxFuture<Record, SharedSenderBRecvF>) {
        tokio::spawn(async move {
            let mut context = Context {
                stdout: stdout(),
                file: None,
            };
            let wait_handle = SHUTDOWN_NOTIFY.register(1).await;

            loop {
                futures::select_biased! {
                    _ = Logger::process(&mut context, &rx).fuse() => {},
                    resp = wait_handle.wait().fuse() => {
                        while rx.len() > 0 {
                            Logger::process(&mut context, &rx).await;
                        }

                        resp.ready();
                        break
                    },
                };
            }
        });
    }

    async fn process(
        ctx: &mut Context,
        rx: &mpsc::RxFuture<Record, SharedSenderBRecvF>,
    ) {
        let res = rx.recv().await.unwrap().record(ctx).await;
        match res {
            Err(err) => eprintln!("record error: {:?}", err),
            _ => {}
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Logger::new()
    }
}

struct Context {
    stdout: Stdout,
    file: Option<(NaiveDate, File)>,
}

#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    serde::Serialize,
)]
pub struct Metadata {
    level: Level,
    target: CompactStr,
}

impl Metadata {
    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    #[inline]
    pub fn target(&self) -> &CompactStr {
        &self.target
    }
}

#[derive(Clone, Debug, serde::Serialize)]
struct Record {
    metadata: Metadata,
    content: Cow<'static, str>,
    file: CompactStr,
    line: u32,
    time: DateTime<Local>,
}

impl Record {
    async fn record(self, cxt: &mut Context) -> std::io::Result<()> {
        let (r1, r2) = tokio::join!(
            self.record_to_stdout(&mut cxt.stdout),
            self.record_to_file(&mut cxt.file)
        );

        r1.or(r2)
    }

    async fn record_to_stdout(
        &self,
        stdout: &mut Stdout,
    ) -> std::io::Result<()> {
        #[inline]
        fn level_color(lvl: Level) -> ColoredString {
            (match lvl {
                Level::Trace => Colorize::purple,
                Level::Debug => Colorize::green,
                Level::Info => Colorize::blue,
                Level::Warn => Colorize::yellow,
                Level::Error => Colorize::red,
            })(lvl.as_str())
        }

        let format = format!(
            "{time} [{lvl}]({target}) {content}\n",
            time = self
                .time
                .to_rfc3339_opts(SecondsFormat::Secs, true)
                .black()
                .on_bright_white(),
            lvl = level_color(self.metadata.level),
            target = self.metadata.target.bold(),
            content = self.content
        );
        stdout.write_all(format.as_bytes()).await?;
        stdout.flush().await?;
        Ok(())
    }

    async fn record_to_file(
        &self,
        data: &mut Option<(NaiveDate, File)>,
    ) -> std::io::Result<()> {
        #[inline]
        async fn new_file(date: NaiveDate) -> std::io::Result<File> {
            let path = get_config_temp().data_path().join("log");
            let filename = format!("{}.log", date.to_string());

            create_dir_all(&path)?;

            OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(path.join(filename))
                .await
        }

        let now = Local::now().date().naive_local();

        if data.as_ref().map(|(date, _)| now > *date).unwrap_or(true)
        {
            *data = Some((now, new_file(now).await?))
        }

        let file = &mut data.as_mut().unwrap().1;

        let mut data = Vec::with_capacity(100);
        serde_json::to_writer(&mut data, self).unwrap();
        data.push(b'\n');

        file.write_all(&data).await?;
        // todo: 定时保存
        //file.sync_data().await?;

        Ok(())
    }
}

#[tokio::test]
async fn log_test() {
    config::init(vec![]).unwrap();
    init();
    log::trace!("hello");
    log::debug!("world");
    log::info!("{}", 1 + 1);
    log::warn!("fbi");
    log::error!("!!!!");
    tracing::info!("tracing~");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
}
