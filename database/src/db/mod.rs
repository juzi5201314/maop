use rbatis::core::db::{DBConnectOption, DBPoolOptions};
use rbatis::rbatis::Rbatis;
use rbatis::DriverType;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::ConnectOptions;

use config::MaopConfig;
use std::str::FromStr;
use error::Error;
use rbatis::executor::Executor;

pub async fn new(m_config: &MaopConfig) -> Result<Rbatis, Error> {
    let config = m_config.database();
    let rb = Rbatis::new();
    rb.link_cfg(
        &DBConnectOption {
            driver_type: DriverType::Sqlite,
            sqlite: Some({
                let mut opt =
                    SqliteConnectOptions::from_str(&config.uri().replace("{{data_path}}", &*m_config.data_path().to_string_lossy()))?
                        .create_if_missing(true);
                opt.log_statements(log::LevelFilter::Debug);
                opt.log_slow_statements(
                    log::LevelFilter::Warn,
                    *config.warn_time().duration(),
                );
                opt
            }),
        },
        &DBPoolOptions {
            max_connections: *config.max_conn(),
            min_connections: *config.min_conn(),
            connect_timeout: *config.timeout().duration(),
            max_lifetime: Some(*config.max_lifetime().duration()),
            idle_timeout: Some(*config.idle_timeout().duration()),
            test_before_acquire: true,
        },
    ).await?;
    rb.exec(include_str!("../sqls/create_posts.sql"), &vec![]).await?;
    rb.exec(include_str!("../sqls/create_commits.sql"), &vec![]).await?;
    Ok(rb)
}
