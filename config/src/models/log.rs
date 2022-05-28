use std::collections::HashMap;
use compact_str::CompactString;
use _log::Level;

crate::gen_config!(LogConfig, {
    filter: HashMap<CompactString, Level>,
    level: Level
});
