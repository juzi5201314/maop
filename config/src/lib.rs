use std::path::Path;
use std::sync::Arc;

use arc_swap::{ArcSwap, ArcSwapAny, Cache, Guard};
use compact_str::CompactStr;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;

pub use models::*;

mod models;

#[macro_export]
macro_rules! gen_config {
    ($name:ident, { $($(#[$attr:meta])* $field:ident: $r#type:ty),* }) => {
        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
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

static CONFIG: OnceCell<Config> = OnceCell::new();

/// 快速的获取, 但不能长时间保存, 也不要跨越yield点
#[inline(always)]
pub fn get_config_temp() -> Guard<Arc<MaopConfig>> {
    CONFIG.get().unwrap().inner.load()
}

#[inline(always)]
pub fn get_config_full() -> Arc<MaopConfig> {
    CONFIG.get().unwrap().inner.load_full()
}

#[inline(always)]
pub fn get_config_cache(
) -> Cache<Arc<ArcSwapAny<Arc<MaopConfig>>>, Arc<MaopConfig>> {
    Cache::new(Arc::clone(&CONFIG.get().unwrap().inner))
}

pub fn init(paths: Vec<CompactStr>) -> anyhow::Result<()> {
    CONFIG.set(Config::new(paths)?).ok();
    Ok(())
}

struct Config {
    raw: Mutex<config_rs::Config>,
    inner: Arc<ArcSwap<MaopConfig>>,
    _watcher: OnceCell<RecommendedWatcher>,
}

impl Config {
    fn new(paths: Vec<CompactStr>) -> anyhow::Result<Config> {
        let mut c = config_rs::Config::new();
        c.merge(
            config_rs::File::from_str(
                include_str!("default.toml"),
                config_rs::FileFormat::Toml,
            )
            .required(true),
        )?;

        paths
            .iter()
            .try_for_each::<_, Result<(), config_rs::ConfigError>>(
                |file_name| {
                    c.merge(
                        config_rs::File::with_name(&*file_name)
                            .required(false),
                    )?;
                    Ok(())
                },
            )?;

        c.merge(
            config_rs::Environment::with_prefix("MAOP")
                .separator("__"),
        )?;

        let maop_config = c.clone().try_into::<MaopConfig>()?;

        let config = Config {
            inner: Arc::new(ArcSwap::from_pointee(maop_config)),
            raw: Mutex::new(c),
            _watcher: OnceCell::new(),
        };

        config.create_data_dir()?;
        config.watch(paths)?;

        Ok(config)
    }

    fn refresh(&self) -> anyhow::Result<()> {
        let mut config = self.raw.lock();
        config.refresh()?;
        let maop_config = config.clone().try_into::<MaopConfig>()?;
        self.inner.store(Arc::new(maop_config));
        self.create_data_dir()?;
        Ok(())
    }

    fn watch(&self, paths: Vec<CompactStr>) -> notify::Result<()> {
        let mut watcher = notify::recommended_watcher(
            |res: notify::Result<Event>| match res {
                Ok(event) => {
                    if event.kind.is_create()
                        || event.kind.is_modify()
                    {
                        if let Err(err) =
                            CONFIG.get().unwrap().refresh()
                        {
                            eprintln!("watch error: {:?}", err)
                        }
                    }
                }
                Err(err) => eprintln!("watch error: {:?}", err),
            },
        )?;
        paths.into_iter().try_for_each(|path| {
            let path = Path::new(&*path);
            if path.exists() {
                watcher.watch(path, RecursiveMode::NonRecursive)
            } else {
                Ok(())
            }
        })?;
        self._watcher.set(watcher).ok();
        Ok(())
    }

    fn create_data_dir(&self) -> std::io::Result<()> {
        let config = self.inner.load();
        let path = config.data_path();
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        } else if !path.is_dir() {
            panic!("data path: `{:?}` no a dir", path);
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new(Vec::new()).unwrap()
    }
}

#[test]
fn config_test() {
    init(vec![]).unwrap();
    let guard = get_config_temp();
    let a = guard.site().name();
    assert_eq!(a.as_str(), "Maop");

    let conf = get_config_full();
    let a = conf.site().name();
    assert_eq!(a.as_str(), "Maop");

    let mut cache = get_config_cache();
    let a = cache.load().site().name();
    assert_eq!(a.as_str(), "Maop");
}
