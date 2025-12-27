use std::path::PathBuf;

use polars::prelude::DataFrame;

/// Metadata about a domain frame's provenance and identity.
///
/// This struct tracks the source files that contributed to a domain frame,
/// the dataset naming for outputs, and any split domain information.
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

    /// For split domains (e.g., FACM, FAAE), the variant identifier.
    /// This is the suffix or distinguishing part of the split.
    pub split_variant: Option<String>,

    /// The base SDTM domain code before splitting (e.g., "FA" for FACM).
    /// For non-split domains, this equals the domain_code.
    pub base_domain_code: Option<String>,
}

impl DomainFrameMeta {
    /// Create a new empty metadata instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the dataset name.
    pub fn with_dataset_name(mut self, name: impl Into<String>) -> Self {
        self.dataset_name = Some(name.into());
        self
    }

    /// Add a source file path.
    pub fn with_source_file(mut self, path: PathBuf) -> Self {
        self.source_files.push(path);
        self
    }

    /// Add multiple source file paths.
    pub fn with_source_files(mut self, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        self.source_files.extend(paths);
        self
    }

    /// Set the split variant.
    pub fn with_split_variant(mut self, variant: impl Into<String>) -> Self {
        self.split_variant = Some(variant.into());
        self
    }

    /// Set the base domain code.
    pub fn with_base_domain_code(mut self, code: impl Into<String>) -> Self {
        self.base_domain_code = Some(code.into());
        self
    }

    /// Get the effective dataset name, falling back to the provided domain_code.
    pub fn effective_dataset_name(&self, domain_code: &str) -> String {
        self.dataset_name
            .clone()
            .unwrap_or_else(|| domain_code.to_uppercase())
    }

    /// Check if this is a split domain.
    pub fn is_split_domain(&self) -> bool {
        self.split_variant.is_some()
    }
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

    /// Create a new domain frame with metadata.
    pub fn with_meta(
        domain_code: impl Into<String>,
        data: DataFrame,
        meta: DomainFrameMeta,
    ) -> Self {
        Self {
            domain_code: domain_code.into(),
            data,
            meta: Some(meta),
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

    /// Check if this is a split domain.
    pub fn is_split_domain(&self) -> bool {
        self.meta
            .as_ref()
            .map(|m| m.is_split_domain())
            .unwrap_or(false)
    }

    /// Get the base domain code (for split domains).
    pub fn base_domain_code(&self) -> &str {
        self.meta
            .as_ref()
            .and_then(|m| m.base_domain_code.as_deref())
            .unwrap_or(&self.domain_code)
    }

    /// Set metadata on this frame.
    pub fn set_meta(&mut self, meta: DomainFrameMeta) {
        self.meta = Some(meta);
    }

    /// Add a source file to the metadata.
    pub fn add_source_file(&mut self, path: PathBuf) {
        if let Some(ref mut meta) = self.meta {
            meta.source_files.push(path);
        } else {
            self.meta = Some(DomainFrameMeta::new().with_source_file(path));
        }
    }
}
