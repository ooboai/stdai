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

    #[error("identity required — all writes must be signed.\n\nTo create an identity:\n\n  stdai identity new --label \"my-agent-name\"\n\nThen set it for your session:\n\n  export STDAI_IDENTITY=<address>\n\nOr pass it per-command:\n\n  stdai write --identity <address> ...")]
    IdentityRequired,

    #[error("identity not found: {0}")]
    IdentityNotFound(String),

    #[error("signature verification failed: {0}")]
    SignatureInvalid(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
