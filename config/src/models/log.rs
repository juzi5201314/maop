use std::collections::HashMap;
use compact_str::CompactStr;

crate::gen_config!(LogConfig, {
    filter: HashMap<CompactStr, CompactStr>,
    flush_stdout_every_time: bool,
    full_module_path: bool
});
