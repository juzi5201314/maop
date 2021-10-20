use crate::models::*;

#[rustfmt::skip]
crate::gen_config!(MaopConfig, {
    rocket: RocketConfig,
    database: DatabaseConfig,
    settings: SettingsConfig,
    log: LogConfig
});
