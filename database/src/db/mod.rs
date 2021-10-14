use rbatis::core::db::{DBConnectOption, DBPoolOptions};
use rbatis::executor::Executor;
use rbatis::rbatis::Rbatis;
use rbatis::DriverType;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous,
};
use sqlx::ConnectOptions;

use config::MaopConfig;
use error::Error;

pub async fn new(m_config: &MaopConfig) -> Result<Rbatis, Error> {
    let config = m_config.database();
    let rb = Rbatis::new();
    rb.link_cfg(
        &DBConnectOption {
            driver_type: DriverType::Sqlite,
            sqlite: Some({
                let mut opt = SqliteConnectOptions::new()
                    .filename(m_config.data_path().join("main.db"))
                    .journal_mode(SqliteJournalMode::Wal)
                    .synchronous(SqliteSynchronous::Normal)
                    .create_if_missing(true)
                    .statement_cache_capacity(
                        *config.statement_cache_capacity(),
                    )
                    .page_size(*config.page_size())
                    .shared_cache(*config.shared_cache());

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
    )
    .await?;
    rb.exec(include_str!("../sqls/create_posts.sql"), vec![])
        .await?;
    rb.exec(include_str!("../sqls/create_comments.sql"), vec![])
        .await?;
    Ok(rb)
}
