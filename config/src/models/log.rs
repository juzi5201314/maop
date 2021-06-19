use std::collections::HashMap;

crate::gen_config!(LogConfig, {
    filter: HashMap<String, String>,
    flush_stdout_every_time: bool,
    full_module_path: bool
});
