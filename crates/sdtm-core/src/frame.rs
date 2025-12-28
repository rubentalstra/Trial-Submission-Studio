use std::path::PathBuf;

use polars::prelude::DataFrame;

/// Metadata about a domain frame's provenance and identity.
///
/// This struct tracks the source files that contributed to a domain frame
/// and the dataset naming for outputs.
///
/// # SDTMIG Reference
/// See Chapter 4.1.4 (Split Datasets) for rules on domain splitting.
/// See Chapter 8 for relationship datasets and SUPPQUAL naming.
#[derive(Debug, Clone, Default)]
pub struct DomainFrameMeta {
    /// The output dataset name (e.g., "AE", "SUPPDM", "FACM").
    /// If None, defaults to the domain_code.
    pub dataset_name: Option<String>,

    /// The source CSV files that contributed to this frame.
    /// Useful for traceability in validation reports.
    pub source_files: Vec<PathBuf>,

    /// The base SDTM domain code before splitting (e.g., "FA" for FACM).
    /// For non-split domains, this equals the domain_code.
    pub base_domain_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DomainFrame {
    pub domain_code: String,
    pub data: DataFrame,
    /// Optional metadata about provenance and naming.
    pub meta: Option<DomainFrameMeta>,
}

impl DomainFrame {
    /// Create a new domain frame with just domain code and data.
    pub fn new(domain_code: impl Into<String>, data: DataFrame) -> Self {
        Self {
            domain_code: domain_code.into(),
            data,
            meta: None,
        }
    }

    pub fn record_count(&self) -> usize {
        self.data.height()
    }

    /// Get the effective dataset name for output files.
    pub fn dataset_name(&self) -> String {
        self.meta
            .as_ref()
            .and_then(|m| m.dataset_name.clone())
            .unwrap_or_else(|| self.domain_code.to_uppercase())
    }

    /// Get the source files that contributed to this frame.
    pub fn source_files(&self) -> &[PathBuf] {
        self.meta
            .as_ref()
            .map(|m| m.source_files.as_slice())
            .unwrap_or(&[])
    }

    /// Get the base domain code (for split domains).
    pub fn base_domain_code(&self) -> &str {
        self.meta
            .as_ref()
            .and_then(|m| m.base_domain_code.as_deref())
            .unwrap_or(&self.domain_code)
    }

    /// Add a source file to the metadata.
    pub fn add_source_file(&mut self, path: PathBuf) {
        if let Some(ref mut meta) = self.meta {
            meta.source_files.push(path);
        } else {
            self.meta = Some(DomainFrameMeta {
                dataset_name: None,
                source_files: vec![path],
                base_domain_code: None,
            });
        }
    }
}
