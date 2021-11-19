use std::collections::HashMap;
use compact_str::CompactStr;
use _log::Level;

crate::gen_config!(LogConfig, {
    filter: HashMap<CompactStr, Level>,
    level: Level
});
