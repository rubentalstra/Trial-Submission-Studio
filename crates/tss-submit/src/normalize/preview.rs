//! Preview DataFrame builder for validation integration.
//!
//! Builds transformed DataFrames for the GUI validation tab
//! using column mappings and domain metadata.

use polars::prelude::DataFrame;
use std::collections::{BTreeMap, BTreeSet};

use tss_standards::{SdtmDomain, TerminologyRegistry};

use super::error::NormalizationError;
use super::executor::execute_normalization;
use super::inference::infer_normalization_rules;
use super::types::{NormalizationContext, NormalizationPipeline};

/// Build preview DataFrame for validation tab.
///
/// Creates a transformed DataFrame by:
/// 1. Building a normalization pipeline from domain metadata
/// 2. Applying column mappings
/// 3. Executing normalizations on source data
///
/// # Arguments
/// * `source_df` - Source DataFrame with raw data
/// * `mappings` - Column mappings (target SDTM variable -> source column)
/// * `domain` - SDTM domain definition
/// * `study_id` - Study identifier
/// * `ct_registry` - Optional CT registry for normalization
///
/// # Returns
/// Transformed DataFrame with SDTM-compliant columns
pub fn build_preview_dataframe(
    source_df: &DataFrame,
    mappings: &BTreeMap<String, String>,
    domain: &SdtmDomain,
    study_id: &str,
    ct_registry: Option<&TerminologyRegistry>,
) -> Result<DataFrame, NormalizationError> {
    // Call the extended version with empty omitted set
    build_preview_dataframe_with_omitted(
        source_df,
        mappings,
        &BTreeSet::new(),
        domain,
        study_id,
        ct_registry,
    )
}

/// Build preview DataFrame with support for omitted variables.
///
/// Creates a transformed DataFrame by:
/// 1. Building a normalization pipeline from domain metadata
/// 2. Applying column mappings
/// 3. Excluding omitted variables from output
/// 4. Executing normalizations on source data
///
/// # Arguments
/// * `source_df` - Source DataFrame with raw data
/// * `mappings` - Column mappings (target SDTM variable -> source column)
/// * `omitted` - Variables to exclude from output (Permissible only)
/// * `domain` - SDTM domain definition
/// * `study_id` - Study identifier
/// * `ct_registry` - Optional CT registry for normalization
///
/// # Returns
/// Transformed DataFrame with SDTM-compliant columns (excluding omitted variables)
pub fn build_preview_dataframe_with_omitted(
    source_df: &DataFrame,
    mappings: &BTreeMap<String, String>,
    omitted: &BTreeSet<String>,
    domain: &SdtmDomain,
    study_id: &str,
    ct_registry: Option<&TerminologyRegistry>,
) -> Result<DataFrame, NormalizationError> {
    // Build pipeline from domain metadata
    let pipeline = infer_normalization_rules(domain);

    // Apply mappings to pipeline
    let pipeline_with_mappings = apply_mappings_to_pipeline(pipeline, mappings);

    // Create execution context
    let context = NormalizationContext::new(study_id, &domain.name)
        .with_ct_registry(ct_registry.cloned())
        .with_mappings(mappings.clone())
        .with_omitted(omitted.clone());

    // Execute pipeline
    execute_normalization(source_df, &pipeline_with_mappings, &context)
}

/// Apply column mappings to pipeline rules.
fn apply_mappings_to_pipeline(
    mut pipeline: NormalizationPipeline,
    mappings: &BTreeMap<String, String>,
) -> NormalizationPipeline {
    for rule in &mut pipeline.rules {
        rule.source_column = mappings.get(&rule.target_variable).cloned();
    }
    pipeline
}

/// Build preview DataFrame with reference date from DM domain.
///
/// This variant loads RFSTDTC from the DM domain for study day calculations.
///
/// # Arguments
/// * `source_df` - Source DataFrame with raw data
/// * `mappings` - Column mappings (target SDTM variable -> source column)
/// * `domain` - SDTM domain definition
/// * `study_id` - Study identifier
/// * `dm_df` - Optional DM domain DataFrame for RFSTDTC extraction
/// * `ct_registry` - Optional CT registry for normalization
pub fn build_preview_dataframe_with_dm(
    source_df: &DataFrame,
    mappings: &BTreeMap<String, String>,
    domain: &SdtmDomain,
    study_id: &str,
    dm_df: Option<&DataFrame>,
    ct_registry: Option<&TerminologyRegistry>,
) -> Result<DataFrame, NormalizationError> {
    // Call the extended version with empty omitted set
    build_preview_dataframe_with_dm_and_omitted(
        source_df,
        mappings,
        &BTreeSet::new(),
        domain,
        study_id,
        dm_df,
        ct_registry,
    )
}

/// Build preview DataFrame with reference date and omitted variable support.
///
/// # Arguments
/// * `source_df` - Source DataFrame with raw data
/// * `mappings` - Column mappings (target SDTM variable -> source column)
/// * `omitted` - Variables to exclude from output
/// * `domain` - SDTM domain definition
/// * `study_id` - Study identifier
/// * `dm_df` - Optional DM domain DataFrame for RFSTDTC extraction
/// * `ct_registry` - Optional CT registry for normalization
pub fn build_preview_dataframe_with_dm_and_omitted(
    source_df: &DataFrame,
    mappings: &BTreeMap<String, String>,
    omitted: &BTreeSet<String>,
    domain: &SdtmDomain,
    study_id: &str,
    dm_df: Option<&DataFrame>,
    ct_registry: Option<&TerminologyRegistry>,
) -> Result<DataFrame, NormalizationError> {
    // Build pipeline from domain metadata
    let pipeline = infer_normalization_rules(domain);

    // Apply mappings to pipeline
    let pipeline_with_mappings = apply_mappings_to_pipeline(pipeline, mappings);

    // Extract reference date from DM if available
    let reference_date = dm_df.and_then(extract_reference_date);

    // Create execution context
    let context = NormalizationContext::new(study_id, &domain.name)
        .with_reference_date(reference_date)
        .with_ct_registry(ct_registry.cloned())
        .with_mappings(mappings.clone())
        .with_omitted(omitted.clone());

    // Execute pipeline
    execute_normalization(source_df, &pipeline_with_mappings, &context)
}

/// Extract RFSTDTC from DM domain DataFrame.
fn extract_reference_date(dm_df: &DataFrame) -> Option<chrono::NaiveDate> {
    // Look for RFSTDTC column
    let rfstdtc = dm_df.column("RFSTDTC").ok()?;

    // Get first non-null value
    for idx in 0..rfstdtc.len() {
        if let Ok(polars::prelude::AnyValue::String(s)) = rfstdtc.get(idx)
            && !s.is_empty()
            && s.len() >= 10
            && let Ok(date) = chrono::NaiveDate::parse_from_str(&s[..10], "%Y-%m-%d")
        {
            return Some(date);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use tss_standards::{
        CoreDesignation, SdtmDatasetClass, SdtmVariable, VariableRole, VariableType,
    };

    fn create_test_domain() -> SdtmDomain {
        SdtmDomain {
            name: "AE".to_string(),
            label: Some("Adverse Events".to_string()),
            class: Some(SdtmDatasetClass::Events),
            structure: None,
            dataset_name: None,
            variables: vec![
                SdtmVariable {
                    name: "STUDYID".to_string(),
                    label: Some("Study Identifier".to_string()),
                    data_type: VariableType::Char,
                    length: None,
                    role: Some(VariableRole::Identifier),
                    core: Some(CoreDesignation::Required),
                    codelist_code: None,
                    described_value_domain: None,
                    order: Some(1),
                },
                SdtmVariable {
                    name: "USUBJID".to_string(),
                    label: Some("Unique Subject Identifier".to_string()),
                    data_type: VariableType::Char,
                    length: None,
                    role: Some(VariableRole::Identifier),
                    core: Some(CoreDesignation::Required),
                    codelist_code: None,
                    described_value_domain: None,
                    order: Some(2),
                },
                SdtmVariable {
                    name: "AETERM".to_string(),
                    label: Some("Reported Term".to_string()),
                    data_type: VariableType::Char,
                    length: None,
                    role: Some(VariableRole::Topic),
                    core: Some(CoreDesignation::Required),
                    codelist_code: None,
                    described_value_domain: None,
                    order: Some(3),
                },
            ],
        }
    }

    #[test]
    fn test_build_preview_dataframe() {
        let domain = create_test_domain();

        let source_df = df! {
            "SUBJECT" => &["001", "002"],
            "AE_TERM" => &["Headache", "Nausea"],
        }
        .unwrap();

        let mut mappings = BTreeMap::new();
        mappings.insert("SUBJID".to_string(), "SUBJECT".to_string());
        mappings.insert("AETERM".to_string(), "AE_TERM".to_string());

        let result = build_preview_dataframe(&source_df, &mappings, &domain, "CDISC01", None);

        assert!(result.is_ok());
        let df = result.unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 3); // STUDYID, USUBJID, AETERM
    }
}
