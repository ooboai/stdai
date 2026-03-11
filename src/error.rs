use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Db(#[from] rusqlite::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

    #[error("artifact not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
