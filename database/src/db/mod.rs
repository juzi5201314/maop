use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Context;
use sqlx::sqlite::SqliteLockingMode;
use sqlx::AnyPool;
use sqlx::ConnectOptions;

#[derive(Clone, Copy)]
pub enum DBType {
    Sqlite,
    Pgsql,
    //Mysql,
}

utils::strum!(DBType, {
    Sqlite,
    Pgsql
    //Mysql
});

pub struct Database {
    ty: DBType,
    pool: AnyPool,
}

impl Database {
    pub async fn new() -> anyhow::Result<Self> {
        let conf = config::get_config();
        let db_conf = conf.database();
        let ty = DBType::from_str(
            db_conf
                .uri()
                .split_once(":")
                .ok_or_else(|| {
                    anyhow::anyhow!(utils::i18n!(
                        "errors.database.unsupported_database_type"
                    ))
                })?
                .0,
        )
        .map_err(|_| {
            anyhow::anyhow!(
                "{}",
                utils::i18n!(
                    "errors.database.unsupported_database_type"
                )
            )
        })?;
        Ok({
            let db = Database {
                ty,
                pool: sqlx::any::AnyPoolOptions::default()
                    .connect_timeout(db_conf.timeout().duration().clone())
                    .max_connections(*db_conf.max_conn())
                    .min_connections(*db_conf.min_conn())
                    .max_lifetime(Some(
                        db_conf.max_lifetime().duration().clone(),
                    ))
                    .idle_timeout(Some(
                        db_conf.idle_timeout().duration().clone(),
                    ))
                    .connect_with(match ty {
                        DBType::Sqlite => {
                            let mut opt = sqlx::sqlite::SqliteConnectOptions::from_str(db_conf.uri())?
                                .create_if_missing(true)
                                .locking_mode(SqliteLockingMode::Normal);
                            opt.log_statements(log::LevelFilter::Debug);
                            opt.log_slow_statements(log::LevelFilter::Warn, db_conf.warn_time().duration().clone());
                            opt.into()
                        },
                        DBType::Pgsql => {
                            let mut opt = sqlx::postgres::PgConnectOptions::from_str(db_conf.uri())?;
                            opt.log_statements(log::LevelFilter::Debug);
                            opt.log_slow_statements(log::LevelFilter::Warn, db_conf.warn_time().duration().clone());
                            opt.into()
                        },
                        //DBType::Mysql => sqlx::sqlite::SqliteConnectOptions::from_str(db.uri())?
                    })
                    .await?,
            };
            sqlx::migrate!("./migrations").run(db.deref()).await?;
            db
        })
    }

    pub fn _type(&self) -> DBType {
        self.ty
    }
}

impl Deref for Database {
    type Target = AnyPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
