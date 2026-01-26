//! Study loading and creation services.
//!
//! Background tasks for creating studies from user assignments.

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use tss_standards::TerminologyRegistry;
use tss_standards::sdtm::get_reciprocal_srel;

use crate::error::GuiError;
use crate::state::{DomainSource, DomainState, Study, WorkflowMode};

/// Create a study from manual source-to-domain assignments.
///
/// This is the primary flow where users explicitly assign CSV files to domains
/// via the source assignment screen.
///
/// # Arguments
/// * `folder` - Study folder path
/// * `assignments` - Map of domain_code to file_path (user assignments)
/// * `metadata_files` - Files marked as metadata (e.g., Items.csv for column labels)
/// * `header_rows` - Number of header rows in CSV files
/// * `confidence_threshold` - Minimum confidence (0.0-1.0) for mapping suggestions
/// * `workflow_mode` - SDTM, ADaM, or SEND
pub async fn create_study_from_assignments(
    folder: PathBuf,
    assignments: BTreeMap<String, PathBuf>,
    metadata_files: Vec<PathBuf>,
    header_rows: usize,
    confidence_threshold: f32,
    workflow_mode: WorkflowMode,
) -> Result<(Study, TerminologyRegistry), GuiError> {
    if assignments.is_empty() {
        return Err(GuiError::operation("Create study", "No domains assigned"));
    }

    // Create study from folder
    let mut study = Study::from_folder(folder);

    // Load metadata from explicitly marked Items.csv files (for column labels)
    for metadata_path in &metadata_files {
        if let Ok(items_metadata) = tss_ingest::load_items_metadata(metadata_path, header_rows) {
            study.metadata = Some(items_metadata);
            tracing::info!(path = %metadata_path.display(), "Loaded column labels from metadata file");
            break; // Use first valid Items.csv
        }
    }

    // Load standards based on workflow mode
    let (ig_domains, terminology) = match workflow_mode {
        WorkflowMode::Sdtm => {
            let domains = tss_standards::load_sdtm_ig().map_err(|e| {
                GuiError::operation("Load standards", format!("Failed to load SDTM-IG: {}", e))
            })?;
            let ct_version = tss_standards::ct::CtVersion::default();
            let ct = tss_standards::ct::load(ct_version, Some("SDTM")).map_err(|e| {
                GuiError::operation(
                    "Load standards",
                    format!(
                        "Failed to load Controlled Terminology ({}): {}",
                        ct_version, e
                    ),
                )
            })?;
            tracing::info!(
                "Loaded CT {} with {} catalogs",
                ct_version,
                ct.catalogs.len()
            );
            (domains, ct)
        }
        WorkflowMode::Adam | WorkflowMode::Send => {
            return Err(GuiError::operation(
                "Create study",
                format!(
                    "{} workflow not yet fully supported",
                    workflow_mode.display_name()
                ),
            ));
        }
    };

    // Process each assignment
    for (domain_code, file_path) in assignments {
        // Load CSV file
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        let (mut df, _headers) =
            tss_ingest::read_csv_table(&file_path, header_rows).map_err(|e| {
                GuiError::domain_load(&domain_code, format!("Failed to load {}: {}", file_stem, e))
            })?;

        // Find domain in IG
        let ig_domain = ig_domains
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(&domain_code));

        let Some(ig_domain) = ig_domain else {
            tracing::warn!("Domain {} not found in IG, skipping", domain_code);
            continue;
        };

        // RELSUB: Auto-generate missing reciprocal relationships per SDTM-IG Section 8.7
        if domain_code.eq_ignore_ascii_case("RELSUB") {
            let (augmented_df, added_count) = ensure_relsub_bidirectional(df);
            df = augmented_df;
            if added_count > 0 {
                tracing::info!(
                    "Auto-generated {} reciprocal relationship(s) for RELSUB domain",
                    added_count
                );
            }
        }

        // Create source
        let source = DomainSource::new(file_path, df.clone(), ig_domain.label.clone());

        // Create mapping state
        let hints = tss_ingest::build_column_hints(&df);
        let source_columns: Vec<String> = df
            .get_column_names()
            .into_iter()
            .map(ToString::to_string)
            .collect();

        let mapping = tss_submit::MappingState::new(
            ig_domain.clone(),
            &study.study_id,
            &source_columns,
            hints,
            confidence_threshold,
        );

        // Create domain and add to study
        let domain = DomainState::new(source, mapping);
        study.add_domain(domain_code.to_uppercase(), domain);
    }

    if study.domain_count() == 0 {
        return Err(GuiError::operation(
            "Create study",
            "No valid domains created from assignments",
        ));
    }

    Ok((study, terminology))
}

// =============================================================================
// RELSUB BIDIRECTIONAL AUTO-GENERATION
// =============================================================================

/// Post-process RELSUB domain after import: auto-generate missing reciprocal relationships.
///
/// Per SDTM-IG v3.4 Section 8.7 Assumption 7:
/// "Every relationship between 2 study subjects is represented in RELSUB
/// as 2 directional relationships: (1) with the first subject's identifier
/// in USUBJID and the second subject's identifier in RSUBJID, and (2) with
/// the second subject's identifier in USUBJID and the first subject's
/// identifier in RSUBJID."
///
/// # Example
///
/// Input CSV with only:
/// ```text
/// STUDYID,USUBJID,RSUBJID,SREL
/// STUDY1,SUBJ-001,SUBJ-002,MOTHER, BIOLOGICAL
/// ```
///
/// Output DataFrame will have 2 rows:
/// ```text
/// STUDY1,SUBJ-001,SUBJ-002,MOTHER, BIOLOGICAL
/// STUDY1,SUBJ-002,SUBJ-001,CHILD, BIOLOGICAL
/// ```
pub fn ensure_relsub_bidirectional(df: DataFrame) -> (DataFrame, usize) {
    // Check if this is a RELSUB-like DataFrame with required columns
    let has_usubjid = df.column("USUBJID").is_ok();
    let has_rsubjid = df.column("RSUBJID").is_ok();
    let has_srel = df.column("SREL").is_ok();

    if !has_usubjid || !has_rsubjid || !has_srel {
        return (df, 0);
    }

    // Get STUDYID and DOMAIN if present for the new rows
    let studyid_col = df.column("STUDYID").ok();
    let domain_col = df.column("DOMAIN").ok();

    let usubjid_col = df.column("USUBJID").expect("USUBJID column exists");
    let rsubjid_col = df.column("RSUBJID").expect("RSUBJID column exists");
    let srel_col = df.column("SREL").expect("SREL column exists");

    // Build set of existing relationships
    let mut existing: HashSet<(String, String)> = HashSet::new();

    for i in 0..df.height() {
        let usubjid = get_string_value(usubjid_col, i);
        let rsubjid = get_string_value(rsubjid_col, i);
        existing.insert((usubjid, rsubjid));
    }

    // Find missing reciprocals
    let mut new_rows: Vec<(String, String, String, String, String)> = Vec::new(); // (studyid, domain, usubjid, rsubjid, srel)

    for i in 0..df.height() {
        let usubjid = get_string_value(usubjid_col, i);
        let rsubjid = get_string_value(rsubjid_col, i);
        let srel = get_string_value(srel_col, i);

        let reverse_key = (rsubjid.clone(), usubjid.clone());

        // Only add reciprocal if:
        // 1. The reverse relationship doesn't exist
        // 2. We can find a reciprocal SREL term
        if !existing.contains(&reverse_key)
            && let Some(reciprocal_srel) = get_reciprocal_srel(&srel)
        {
            let studyid = studyid_col
                .map(|c| get_string_value(c, i))
                .unwrap_or_default();
            let domain = domain_col
                .map(|c| get_string_value(c, i))
                .unwrap_or_else(|| "RELSUB".to_string());

            new_rows.push((
                studyid,
                domain,
                rsubjid.clone(),
                usubjid.clone(),
                reciprocal_srel.to_string(),
            ));
            existing.insert(reverse_key);
        }
    }

    let added_count = new_rows.len();

    if new_rows.is_empty() {
        return (df, 0);
    }

    // Build new DataFrame with original + new rows
    let original_height = df.height();
    let new_height = original_height + new_rows.len();

    // Extract original columns as vectors and append new values
    let mut studyid_vec: Vec<String> = (0..original_height)
        .map(|i| {
            studyid_col
                .map(|c| get_string_value(c, i))
                .unwrap_or_default()
        })
        .collect();
    let mut domain_vec: Vec<String> = (0..original_height)
        .map(|i| {
            domain_col
                .map(|c| get_string_value(c, i))
                .unwrap_or_else(|| "RELSUB".to_string())
        })
        .collect();
    let mut usubjid_vec: Vec<String> = (0..original_height)
        .map(|i| get_string_value(usubjid_col, i))
        .collect();
    let mut rsubjid_vec: Vec<String> = (0..original_height)
        .map(|i| get_string_value(rsubjid_col, i))
        .collect();
    let mut srel_vec: Vec<String> = (0..original_height)
        .map(|i| get_string_value(srel_col, i))
        .collect();

    // Append new rows
    for (studyid, domain, usubjid, rsubjid, srel) in new_rows {
        studyid_vec.push(studyid);
        domain_vec.push(domain);
        usubjid_vec.push(usubjid);
        rsubjid_vec.push(rsubjid);
        srel_vec.push(srel);
    }

    // Build new DataFrame
    let new_df = DataFrame::new(vec![
        Series::new("STUDYID".into(), studyid_vec).into(),
        Series::new("DOMAIN".into(), domain_vec).into(),
        Series::new("USUBJID".into(), usubjid_vec).into(),
        Series::new("RSUBJID".into(), rsubjid_vec).into(),
        Series::new("SREL".into(), srel_vec).into(),
    ]);

    match new_df {
        Ok(df) => {
            tracing::info!(
                "Added {} reciprocal relationship(s) to RELSUB (total: {} rows)",
                added_count,
                new_height
            );
            (df, added_count)
        }
        Err(e) => {
            tracing::warn!("Failed to create augmented RELSUB DataFrame: {}", e);
            (df, 0)
        }
    }
}

/// Helper to extract string value from a column at a given row index.
fn get_string_value(col: &Column, idx: usize) -> String {
    match col.get(idx) {
        Ok(AnyValue::String(s)) => s.to_string(),
        Ok(AnyValue::StringOwned(s)) => s.to_string(),
        Ok(v) => v.to_string(),
        Err(_) => String::new(),
    }
}
