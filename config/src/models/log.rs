use std::collections::HashMap;

crate::gen_config!(LogConfig, {
    filter: HashMap<String, String>
});
