//! Data processing service
//!
//! Applies SDTM transformations to domain data using standalone functions
//! from sdtm-core::transforms.
//!
//! NOTE: This service will be used when the Transform tab is implemented.

#![allow(dead_code)]

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_core::pipeline_context::CtMatchingMode;
use sdtm_core::transforms::{
    apply_usubjid_prefix, assign_sequence_numbers, get_ct_columns, normalize_ct_column,
};
use sdtm_model::ct::TerminologyRegistry;
use sdtm_model::Domain;

/// Result of applying a transformation
#[derive(Debug, Clone)]
pub struct TransformResult {
    /// Number of values that were modified
    pub modified_count: usize,
    /// Description of what was transformed
    pub description: String,
}

/// Service for applying SDTM transformations to data
pub struct ProcessingService;

impl ProcessingService {
    /// Apply STUDYID prefix to USUBJID column.
    ///
    /// Per SDTMIG 4.1.2, USUBJID should be formatted as "STUDYID-SUBJID".
    pub fn apply_usubjid_prefix(
        df: &mut DataFrame,
        study_id: &str,
    ) -> Result<TransformResult> {
        let modified = apply_usubjid_prefix(df, study_id, "USUBJID", Some("STUDYID"))?;

        Ok(TransformResult {
            modified_count: modified,
            description: format!(
                "Added STUDYID prefix '{}' to {} USUBJID values",
                study_id, modified
            ),
        })
    }

    /// Assign sequence numbers for a domain.
    ///
    /// Per SDTMIG 4.1.5, --SEQ is a unique number for each record within a domain
    /// for a subject. The column name follows the pattern {domain_code}SEQ.
    pub fn assign_sequence_numbers(
        df: &mut DataFrame,
        domain_code: &str,
    ) -> Result<TransformResult> {
        let seq_column = format!("{}SEQ", domain_code);
        let modified = assign_sequence_numbers(df, &seq_column, "USUBJID")?;

        Ok(TransformResult {
            modified_count: modified,
            description: format!(
                "Assigned {} values to {} column",
                modified, seq_column
            ),
        })
    }

    /// Normalize all CT-bound columns in a domain.
    ///
    /// Returns a list of results for each column that was processed.
    pub fn normalize_all_ct_columns(
        df: &mut DataFrame,
        domain: &Domain,
        ct_registry: &TerminologyRegistry,
        matching_mode: CtMatchingMode,
    ) -> Result<Vec<TransformResult>> {
        let ct_columns = get_ct_columns(df, domain);
        let mut results = Vec::new();

        for (column_name, codelist_code) in ct_columns {
            if let Some(resolved) = ct_registry.resolve(&codelist_code, None) {
                let modified = normalize_ct_column(df, &column_name, resolved.codelist, matching_mode)?;

                if modified > 0 {
                    results.push(TransformResult {
                        modified_count: modified,
                        description: format!(
                            "Normalized {} values in {} using codelist {}",
                            modified, column_name, codelist_code
                        ),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Apply all standard SDTM transformations to a domain.
    ///
    /// This applies:
    /// 1. USUBJID prefix (if STUDYID is available)
    /// 2. Sequence numbers (--SEQ)
    /// 3. CT normalization for all applicable columns
    pub fn apply_all_transforms(
        df: &mut DataFrame,
        domain_code: &str,
        study_id: &str,
        domain: Option<&Domain>,
        ct_registry: Option<&TerminologyRegistry>,
        matching_mode: CtMatchingMode,
    ) -> Result<Vec<TransformResult>> {
        let mut results = Vec::new();

        // 1. USUBJID prefix
        let usubjid_result = Self::apply_usubjid_prefix(df, study_id)?;
        if usubjid_result.modified_count > 0 {
            results.push(usubjid_result);
        }

        // 2. Sequence numbers
        let seq_result = Self::assign_sequence_numbers(df, domain_code)?;
        if seq_result.modified_count > 0 {
            results.push(seq_result);
        }

        // 3. CT normalization
        if let (Some(domain), Some(registry)) = (domain, ct_registry) {
            let ct_results = Self::normalize_all_ct_columns(df, domain, registry, matching_mode)?;
            results.extend(ct_results);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[test]
    fn test_apply_usubjid_prefix() {
        let mut df = DataFrame::new(vec![
            Series::new("USUBJID".into(), vec!["001", "002"]).into(),
        ])
        .unwrap();

        let result = ProcessingService::apply_usubjid_prefix(&mut df, "STUDY01").unwrap();

        assert_eq!(result.modified_count, 2);
        let col = df.column("USUBJID").unwrap().str().unwrap();
        assert_eq!(col.get(0), Some("STUDY01-001"));
    }

    #[test]
    fn test_assign_sequence_numbers() {
        let mut df = DataFrame::new(vec![
            Series::new("USUBJID".into(), vec!["A", "A", "B"]).into(),
        ])
        .unwrap();

        let result = ProcessingService::assign_sequence_numbers(&mut df, "AE").unwrap();

        assert_eq!(result.modified_count, 3);
        assert!(df.column("AESEQ").is_ok());
    }
}
