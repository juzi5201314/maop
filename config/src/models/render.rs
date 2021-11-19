use std::path::PathBuf;

crate::gen_config!(RenderConfig, { strict_mode: bool, dev_mode: bool, template: Option<PathBuf> });
