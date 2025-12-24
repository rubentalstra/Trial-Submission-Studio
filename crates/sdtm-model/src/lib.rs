#![deny(unsafe_code)]

pub mod ids;
pub mod provenance;
pub mod table;

pub use crate::ids::{DomainCode, RowId, VarName};
pub use crate::provenance::{DerivationStep, SourceRef};
pub use crate::table::{CellValue, Row, Table};

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("invalid domain code: {0}")]
    InvalidDomainCode(String),
    #[error("invalid variable name: {0}")]
    InvalidVarName(String),
}
