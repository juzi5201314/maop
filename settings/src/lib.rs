use std::ops::Deref;

use anyhow::Context;
use once_cell::sync::Lazy;

use data_structure::{De, DeFromIVec, Se};
use utils::Consume;

static SLED: Lazy<sled::Db> = Lazy::new(|| {
    let conf = config::get_config();
    let conf = conf.settings();
    sled::Config::new()
        .path(config::DATA_PATH.deref().join("settings"))
        .cache_capacity(conf.sled_cache_capacity().get_bytes() as u64)
        .mode(sled::Mode::LowSpace)
        .use_compression(*conf.sled_use_compression())
        .compression_factor(*conf.sled_compression_factor() as i32)
        // 不自动保存, 因为写入操作非常少, 所以可以在写入后手动flush
        .flush_every_ms(
            conf.sled_flush_every_ms()
                .as_ref()
                .map(|time| time.duration().as_millis() as u64),
        )
        .open()
        .with_context(|| {
            utils::i18n!("errors.settings.open_sled_error")
        })
        .unwrap()
});

pub fn get<'a, D, S>(key: S) -> anyhow::Result<Option<D>>
where
    S: AsRef<[u8]>,
    D: De<'a>,
{
    Ok(match SLED.get(key.as_ref())? {
        Some(ivec) => Some(D::deser(ivec)?),
        None => None,
    })
}

pub fn get_default<'a, D, S>(
    key: S,
    default: D,
) -> anyhow::Result<Option<D>>
where
    S: AsRef<[u8]>,
    D: Se + De<'a>,
{
    match get(&key)? {
        Some(v) => Ok(Some(v)),
        None => {
            set(key, &default)?;
            Ok(Some(default))
        }
    }
}

pub fn set<S, T>(
    key: S,
    val: T,
) -> anyhow::Result<Option<impl DeFromIVec>>
where
    S: AsRef<[u8]>,
    T: Se,
{
    utils::defer!(|| SLED.flush().consume());
    Ok(SLED.insert(key.as_ref(), val.ser()?)?.map(|v| v))
}

#[test]
fn settings_test() {
    dbg!(get::<String, _>("abc"));
    dbg!(set("abc", "123").consume());
    dbg!(get::<String, _>("abc"));
}
