use utils::unit::time_unit::TimeUnit;
use utils::unit::byte_unit::ByteUnit;

crate::gen_config!(RuntimeConfig, {
    shutdown_timeout: TimeUnit,
    worker_threads: Option<usize>,
    thread_stack_size: Option<ByteUnit>,
    blocking_thread_keep_alive: Option<TimeUnit>,
    max_blocking_threads: Option<usize>
});
