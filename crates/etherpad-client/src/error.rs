use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("changeset parse error: {0}")]
    ParseChangeset(String),

    #[error("changeset apply error: {0}")]
    ApplyChangeset(String),

    #[error("socket error: {0}")]
    Socket(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ClientError>;
