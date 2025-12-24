#![deny(unsafe_code)]

#[derive(Debug, thiserror::Error)]
pub enum ValidateError {
    #[error("not implemented")]
    NotImplemented,
}
