use utils::unit::time_unit::TimeUnit;

crate::gen_config!(DatabaseConfig, {
    timeout: TimeUnit,
    max_conn: u32,
    min_conn: u32,
    max_lifetime: TimeUnit,
    idle_timeout: TimeUnit,
    warn_time: TimeUnit,

    shared_cache: bool,
    statement_cache_capacity: usize,

    page_size: u32
});
