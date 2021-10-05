use compact_str::CompactStr;
use utils::unit::time_unit::TimeUnit;

crate::gen_config!(DatabaseConfig, {
    uri: CompactStr,
    timeout: TimeUnit,
    max_conn: u32,
    min_conn: u32,
    max_lifetime: TimeUnit,
    idle_timeout: TimeUnit,
    warn_time: TimeUnit
});
