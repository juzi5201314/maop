use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("database error: {0} (rbatis)")]
    Rbatis(#[from] rbatis::Error),

    #[error("database error: {0} (sqlx)")]
    Sqlx(#[from] sqlx::Error),

    #[error("addr parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("template error: {0}")]
    Template(#[from] handlebars::TemplateError),

    #[error("render error: {0}")]
    Render(#[from] handlebars::RenderError),
}
