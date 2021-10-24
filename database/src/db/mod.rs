use anyhow::Context;
use sea_orm::prelude::DbConn;
use sea_orm::{ConnectionTrait, Schema, SqlxSqliteConnector};
use sqlx_core::connection::ConnectOptions;
use sqlx_core::pool::PoolOptions;
use sqlx_core::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode,
    SqliteSynchronous,
};

pub async fn new() -> anyhow::Result<DbConn> {
    let config_full = config::get_config_full();
    let config = config_full.database();

    let mut opt = SqliteConnectOptions::new()
        .filename(config_full.data_path().join("main.db"))
        .journal_mode(SqliteJournalMode::Wal)
        .locking_mode(SqliteLockingMode::Normal)
        .synchronous(SqliteSynchronous::Normal)
        .create_if_missing(true)
        .statement_cache_capacity(*config.statement_cache_capacity())
        .page_size(*config.page_size())
        .shared_cache(*config.shared_cache());

    opt.log_statements(log::LevelFilter::Debug);
    opt.log_slow_statements(
        log::LevelFilter::Warn,
        *config.warn_time().duration(),
    );

    let db = SqlxSqliteConnector::from_sqlx_sqlite_pool(
        PoolOptions::new()
            .max_connections(*config.max_conn())
            .min_connections(*config.min_conn())
            .connect_timeout(*config.timeout().duration())
            .max_lifetime(*config.max_lifetime().duration())
            .idle_timeout(*config.idle_timeout().duration())
            .test_before_acquire(true)
            .connect_with(opt)
            .await
            .context("connect to database")?,
    );

    setup_schema(&db).await.context("setup schema")?;

    Ok(db)
}

async fn setup_schema(db: &DbConn) -> anyhow::Result<()> {
    db.execute(
        db.get_database_backend().build(
            Schema::create_table_from_entity(
                crate::models::post::Entity,
            )
            .if_not_exists(),
        ),
    )
    .await
    .context("create posts")?;

    db.execute(
        db.get_database_backend().build(
            Schema::create_table_from_entity(
                crate::models::comment::Entity,
            )
            .if_not_exists(),
        ),
    )
    .await
    .context("create comments")?;
    Ok(())
}
