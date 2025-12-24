use thiserror::Error;

#[derive(Debug, Error)]
pub enum SdtmError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, SdtmError>;
