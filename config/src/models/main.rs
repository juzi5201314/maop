use crate::models::*;

#[rustfmt::skip]
crate::gen_config!(MaopConfig, {
    database: DatabaseConfig,
    log: LogConfig,
    http: HttpConfig,
    render: RenderConfig
});
