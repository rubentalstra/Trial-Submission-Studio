//! Study loading service.
//!
//! Loads a study folder, discovers domains, and initializes mapping state.

use crate::state::{DomainSource, DomainState, StudyState};
use anyhow::{Context, Result};
use sdtm_ingest::{discover_domain_files, list_csv_files, load_study_metadata, read_csv_table};
use sdtm_map::{ColumnHint, MappingState as CoreMappingState};
use sdtm_standards::load_default_sdtm_ig_domains;
use std::collections::BTreeMap;
use std::path::Path;

/// Service for loading studies from disk.
pub struct StudyLoader;

impl StudyLoader {
    /// Load a study from a folder.
    ///
    /// Discovers CSV files, matches them to SDTM domains, and initializes
    /// mapping state with auto-generated suggestions.
    ///
    /// # Arguments
    /// - `study_folder`: Path to the study folder
    /// - `header_rows`: Number of header rows in CSV files (1 = single, 2 = double with labels)
    pub fn load_study(study_folder: &Path, header_rows: usize) -> Result<StudyState> {
        use std::collections::HashMap;

        // Create the study state (derives study_id from folder name)
        let mut study = StudyState::from_folder(study_folder.to_path_buf());

        // Get supported domain codes from SDTM-IG
        let sdtm_domains =
            load_default_sdtm_ig_domains().context("Failed to load SDTM-IG domain definitions")?;

        // Build lookup: domain code -> (label, domain definition)
        let domain_map: HashMap<String, (Option<String>, sdtm_model::Domain)> = sdtm_domains
            .into_iter()
            .map(|d| (d.name.clone(), (d.label.clone(), d)))
            .collect();

        let domain_codes: Vec<String> = domain_map.keys().cloned().collect();

        // Find all CSV files
        let csv_files =
            list_csv_files(study_folder).context("Failed to list CSV files in study folder")?;

        if csv_files.is_empty() {
            tracing::warn!(
                "No CSV files found in study folder: {}",
                study_folder.display()
            );
            return Ok(study);
        }

        tracing::info!("Found {} CSV files in study folder", csv_files.len());

        // Match files to domains
        let domain_files = discover_domain_files(&csv_files, &domain_codes);

        tracing::info!("Discovered {} domains", domain_files.len());

        // Load study metadata (Items.csv, CodeLists.csv)
        match load_study_metadata(study_folder, header_rows) {
            Ok(metadata) => {
                if !metadata.is_empty() {
                    tracing::info!(
                        "Loaded study metadata: {} items, {} codelists",
                        metadata.items.len(),
                        metadata.codelists.len()
                    );
                    study.metadata = Some(metadata);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to load study metadata: {}", e);
            }
        }

        // Load each domain
        for (domain_code, files) in domain_files {
            // For now, just use the first file for each domain
            // TODO: Handle multiple files per domain (e.g., split datasets)
            if let Some((file_path, _variant)) = files.first() {
                // Get the SDTM domain definition
                let Some((label, sdtm_domain)) = domain_map.get(&domain_code) else {
                    tracing::warn!("No SDTM-IG definition for domain: {}", domain_code);
                    continue;
                };

                match read_csv_table(file_path, header_rows) {
                    Ok((df, _headers)) => {
                        tracing::info!(
                            "Loaded domain {} from {} ({} rows, {} columns)",
                            domain_code,
                            file_path.display(),
                            df.height(),
                            df.width()
                        );

                        // Create DomainSource (immutable)
                        let source = DomainSource::new(file_path.clone(), df, label.clone());

                        // Extract column names for mapping
                        let source_columns = source.columns();

                        // Build column hints from DataFrame
                        let hints = build_column_hints(&source);

                        // Create mapping state with auto-suggestions
                        let mut mapping = CoreMappingState::new(
                            sdtm_domain.clone(),
                            &study.study_id,
                            &source_columns,
                            hints,
                            0.6, // min confidence for suggestions
                        );

                        // Auto-accept generated variables (STUDYID, DOMAIN, --SEQ, USUBJID)
                        auto_accept_generated_variables(&mut mapping);

                        // Create domain state
                        let domain_state = DomainState::new(source, mapping);
                        study.add_domain(domain_code, domain_state);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to load {} from {}: {}",
                            domain_code,
                            file_path.display(),
                            e
                        );
                    }
                }
            }
        }

        Ok(study)
    }
}

/// Build column hints from a DataFrame.
///
/// Extracts metadata about columns for the scoring engine.
fn build_column_hints(source: &DomainSource) -> BTreeMap<String, ColumnHint> {
    use polars::prelude::*;

    let mut hints = BTreeMap::new();

    for col_name in source.data.get_column_names() {
        if let Ok(series) = source.data.column(col_name) {
            let len = series.len() as f64;

            // Calculate unique ratio
            let unique_ratio = if len > 0.0 {
                series.n_unique().unwrap_or(0) as f64 / len
            } else {
                0.0
            };

            // Calculate null ratio
            let null_ratio = if len > 0.0 {
                series.null_count() as f64 / len
            } else {
                0.0
            };

            // Determine if numeric
            let is_numeric = matches!(
                series.dtype(),
                DataType::Int8
                    | DataType::Int16
                    | DataType::Int32
                    | DataType::Int64
                    | DataType::UInt8
                    | DataType::UInt16
                    | DataType::UInt32
                    | DataType::UInt64
                    | DataType::Float32
                    | DataType::Float64
            );

            hints.insert(
                col_name.to_string(),
                ColumnHint {
                    is_numeric,
                    unique_ratio,
                    null_ratio,
                    label: None, // Could be set from study metadata
                },
            );
        }
    }

    hints
}

/// Auto-accept generated variables.
///
/// Variables like STUDYID, DOMAIN, --SEQ, and USUBJID are auto-generated
/// by the transform system and should be marked as such.
fn auto_accept_generated_variables(mapping: &mut CoreMappingState) {
    use sdtm_map::VariableStatus;

    let domain_code = mapping.domain().name.clone();

    // Auto-generated variables
    let generated_vars = vec![
        "STUDYID".to_string(),
        "DOMAIN".to_string(),
        "USUBJID".to_string(),
        format!("{}SEQ", domain_code), // e.g., AESEQ, DMSEQ
    ];

    for var_name in generated_vars {
        // Check if this variable exists in the domain
        if mapping.status(&var_name) != VariableStatus::Unmapped {
            continue;
        }

        // Mark as auto-generated
        mapping.mark_auto_generated(&var_name);
    }
}
