use std::fs::read_dir;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use notify::{RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::task::JoinHandle;

use utils::*;

static CONFIG: Lazy<Config> = Lazy::new(|| {
    #[allow(clippy::expect_fun_call)]
    Config::new().expect(&i18n!("errors.config.init_failed"))
});

pub fn get_config() -> Arc<MaopConfig> {
    Arc::clone(&CONFIG.main.load())
}

macro_rules! gen_config {
    ($name:ident, { $($(#[$attr:meta])* $field:ident: $r#type:ty),* }) => {
        #[derive(Debug, Deserialize, Clone)]
        pub struct $name {
            $($(#[$attr])* $field: $r#type),*
        }
        impl $name {
            $(#[allow(dead_code)] pub fn $field(&self) -> &$r#type {
                &self.$field
            })*
        }
    };
}
gen_config!(RocketConfig, { port: u16 });
gen_config!(MaopConfig, { rocket: RocketConfig });
pub struct Config {
    inner: config_rs::Config,
    main: ArcSwap<MaopConfig>,
    #[allow(dead_code)]
    watch_handle: JoinHandle<()>,
}

impl Config {
    pub fn new() -> anyhow::Result<Config> {
        // 在当前目录找到所有名字叫maop的配置文件
        let config_files: Vec<String> =
            Config::find_config_files(".", "maop")?;
        let mut c = config_rs::Config::new();

        dotenv::dotenv().ok();
        c.merge(config_rs::Environment::with_prefix("MAOP"))?;
        config_files.iter().try_for_each(|file_name| {
            c.merge(
                config_rs::File::with_name(file_name).required(false),
            )
            .erase()
        })?;

        let watch_handle = Config::start_watch(&config_files)?;

        let maop_config: MaopConfig =
            c.clone().try_into().map_err(|err| {
                anyhow::anyhow!(
                    "{}: {}",
                    i18n!("errors.config.parsing_failed"),
                    err
                )
            })?;

        Ok(Config {
            main: ArcSwap::from_pointee(maop_config),
            watch_handle,
            inner: c,
        })
    }

    pub fn refresh(&self) -> anyhow::Result<()> {
        let mut config = self.inner.clone();
        config.refresh()?;
        let config: MaopConfig = config.try_into()?;
        self.main.store(Arc::new(config));
        Ok(())
    }

    fn start_watch(
        watch_files: &[String],
    ) -> anyhow::Result<JoinHandle<()>> {
        let (tx, rx) = channel();
        let mut watcher =
            notify::watcher(tx, Duration::from_secs(2))?;
        watch_files.iter().try_for_each(|watch_file| {
            watcher.watch(watch_file, RecursiveMode::NonRecursive)
        })?;
        Ok(tokio::task::spawn_blocking(move || {
            // 即使不用,也要把watcher move到task里
            // 否则watcher会在函数结束后drop,导致无法继续watch文件
            let _ = watcher;
            loop {
                match rx.recv() {
                    Ok(event) => {
                        if let notify::DebouncedEvent::Write(_) =
                            event
                        {
                            CONFIG.refresh().unwrap()
                        }
                    }
                    Err(_) => {
                        panic!("watch error")
                    }
                }
            }
        }))
    }

    /// 在path找到全部config文件
    /// config文件以固定的后缀结尾
    fn find_config_files(
        path: impl AsRef<Path>,
        name: impl AsRef<str>,
    ) -> anyhow::Result<Vec<String>> {
        Ok(any_try!(
            read_dir(path.as_ref()),
            "{}: {:?}",
            i18n!("errors.io.read_dir_error"),
            path.as_ref()
        )?
        .filter_map(|res| res.ok())
        .map(|de| de.path())
        .filter(|pb| pb.is_file())
        .filter(|pb| {
            pb.extension()
                .map(|oss| {
                    oss.to_str().map(|s| {
                        ["toml", "json", "yml", "yaml", "ini"]
                            .contains(&&*s.trim().to_lowercase())
                    })
                })
                .flatten()
                .unwrap_or(false)
        })
        .filter(|pb| {
            pb.file_stem()
                .map(|oss| {
                    oss.to_str().map(|s| {
                        s.trim().to_lowercase() == name.as_ref()
                    })
                })
                .flatten()
                .unwrap_or(false)
        })
        .filter_map(|pb| {
            let file_name = pb.file_name();
            file_name
                .map(|oss| oss.to_str().map(|s| s.to_owned()))
                .flatten()
        })
        .collect::<Vec<String>>())
    }
}
