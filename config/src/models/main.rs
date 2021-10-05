use std::path::PathBuf;

use crate::models::*;

#[rustfmt::skip]
crate::gen_config!(MaopConfig, {
    #[serde(default = "default_data_path")]
    data_path: PathBuf,
    database: DatabaseConfig,
    log: LogConfig,
    http: HttpConfig,
    render: RenderConfig,
    site: SiteConfig
});

#[inline]
fn default_data_path() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".maop")
}
