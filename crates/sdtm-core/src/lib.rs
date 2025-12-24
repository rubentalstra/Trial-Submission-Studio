#![deny(unsafe_code)]

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("stage failed: {0}")]
    StageFailed(String),
}

pub mod pipeline {
    use sdtm_model::Table;

    #[derive(Debug, Clone)]
    pub struct IngestInput {
        pub domain: String,
        pub source_id: String,
        pub path: std::path::PathBuf,
    }

    #[derive(Debug, Clone)]
    pub struct IngestOutput {
        pub table: Table,
    }

    #[derive(Debug, Clone)]
    pub struct ValidateOutput {
        pub errors: usize,
        pub warnings: usize,
    }

    /// Reads raw source data into a canonical in-memory table.
    pub trait Ingestor {
        fn ingest(&self, input: IngestInput) -> anyhow::Result<IngestOutput>;
    }

    /// Maps canonical input into SDTM domain tables.
    pub trait Mapper {
        fn map(&self, input: Table) -> anyhow::Result<Vec<Table>>;
    }

    /// Validates SDTM domain tables and returns counts.
    pub trait Validator {
        fn validate(&self, tables: &[Table]) -> anyhow::Result<ValidateOutput>;
    }
}
