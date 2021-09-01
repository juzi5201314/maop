use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("database error: {0} (rbatis)")]
    Rbatis(#[from] rbatis::Error),

    #[error("database error: {0} (sqlx)")]
    Sqlx(#[from] sqlx::Error)
}
