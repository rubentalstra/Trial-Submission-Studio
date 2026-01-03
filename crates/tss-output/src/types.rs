//! Output-related types for SDTM data export.

use std::collections::HashMap;

use polars::prelude::DataFrame;
use tss_model::Domain;

/// A transformed domain with its data.
///
/// Used for output generation (XPT, SAS, XML).
#[derive(Debug, Clone)]
pub struct DomainFrame {
    /// Domain code (e.g., "AE", "DM").
    pub domain_code: String,

    /// Transformed DataFrame.
    pub data: DataFrame,

    /// Optional dataset name (for split domains like LBCH, FAAE).
    /// If None, uses domain_code.
    pub dataset_name_override: Option<String>,
}

impl DomainFrame {
    /// Create a new domain frame.
    pub fn new(domain_code: impl Into<String>, data: DataFrame) -> Self {
        Self {
            domain_code: domain_code.into(),
            data,
            dataset_name_override: None,
        }
    }

    /// Create a domain frame with a custom dataset name.
    pub fn with_dataset_name(
        domain_code: impl Into<String>,
        data: DataFrame,
        dataset_name: impl Into<String>,
    ) -> Self {
        Self {
            domain_code: domain_code.into(),
            data,
            dataset_name_override: Some(dataset_name.into()),
        }
    }

    /// Get the dataset name for output.
    /// Uses `dataset_name_override` if set, otherwise `domain_code`.
    pub fn dataset_name(&self) -> String {
        self.dataset_name_override
            .clone()
            .unwrap_or_else(|| self.domain_code.clone())
    }

    /// Get the base domain code.
    /// For split domains (e.g., "LBCH"), returns the parent domain ("LB").
    /// For regular domains, returns the domain_code.
    pub fn base_domain_code(&self) -> &str {
        // Split domains follow pattern: 2-letter base + suffix (e.g., LBCH, FAAE)
        // Check if this looks like a split domain
        if self.domain_code.len() > 2 {
            let base = &self.domain_code[..2];
            // Common SDTM domain prefixes that can be split
            if matches!(base, "LB" | "FA" | "QS" | "VS" | "EG" | "PC" | "PP") {
                return base;
            }
        }
        &self.domain_code
    }
}

/// Create a lookup map from domain code to Domain reference.
pub fn domain_map_by_code(domains: &[Domain]) -> HashMap<String, &Domain> {
    domains.iter().map(|d| (d.name.to_uppercase(), d)).collect()
}
