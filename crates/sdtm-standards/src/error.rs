#![deny(unsafe_code)]

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum StandardsError {
    #[error("failed to read file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse TOML manifest {path}: {source}")]
    Toml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("invalid manifest: {message}")]
    InvalidManifest { message: String },

    #[error("missing required role in manifest: {role}")]
    MissingRole { role: String },

    #[error("missing one of the required CT roles in manifest: {roles}")]
    MissingCtRole { roles: String },

    #[error("duplicate role in manifest: {role}")]
    DuplicateRole { role: String },

    #[error("invalid sha256 for {path}: {message}")]
    InvalidSha256 { path: PathBuf, message: String },

    #[error("invalid manifest path {path}: {message}")]
    InvalidPath { path: PathBuf, message: String },

    #[error("missing file listed in manifest: {path}")]
    MissingFile { path: PathBuf },

    #[error("unexpected file present under standards/: {path}")]
    UnexpectedFile { path: PathBuf },

    #[error("sha256 mismatch for {path} (expected {expected}, got {actual})")]
    Sha256Mismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },

    #[error("failed to parse CSV {path}: {message}")]
    Csv { path: PathBuf, message: String },
}

impl StandardsError {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
