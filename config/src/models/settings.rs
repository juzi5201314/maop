use utils::unit::byte_unit::ByteUnit;
use utils::unit::time_unit::TimeUnit;
crate::gen_config!(SettingsConfig, {
    sled_cache_capacity: ByteUnit,
    sled_use_compression: bool,
    sled_compression_factor: u8,
    sled_flush_every_ms: Option<TimeUnit>
});
