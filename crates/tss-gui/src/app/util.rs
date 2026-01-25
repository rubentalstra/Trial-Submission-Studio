//! Utility functions for the application.
//!
//! Contains:
//! - `create_study_from_assignments` - Create study from manual assignments
//! - `load_app_icon` - Application icon loading

use std::collections::BTreeMap;
use std::path::PathBuf;

use iced::window;

use crate::state::{DomainSource, DomainState, Study, WorkflowMode};
use tss_standards::TerminologyRegistry;

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
) -> Result<(Study, TerminologyRegistry), String> {
    if assignments.is_empty() {
        return Err("No domains assigned".to_string());
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
            let domains = tss_standards::load_sdtm_ig()
                .map_err(|e| format!("Failed to load SDTM-IG: {}", e))?;
            let ct_version = tss_standards::ct::CtVersion::default();
            let ct = tss_standards::ct::load(ct_version, Some("SDTM")).map_err(|e| {
                format!(
                    "Failed to load Controlled Terminology ({}): {}",
                    ct_version, e
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
            return Err(format!(
                "{} workflow not yet fully supported",
                workflow_mode.display_name()
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

        let (df, _headers) = tss_ingest::read_csv_table(&file_path, header_rows)
            .map_err(|e| format!("Failed to load {}: {}", file_stem, e))?;

        // Find domain in IG
        let ig_domain = ig_domains
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(&domain_code));

        let Some(ig_domain) = ig_domain else {
            tracing::warn!("Domain {} not found in IG, skipping", domain_code);
            continue;
        };

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
        return Err("No valid domains created from assignments".to_string());
    }

    Ok((study, terminology))
}

/// Load the application icon from embedded PNG data.
pub fn load_app_icon() -> Option<window::Icon> {
    let icon_data = include_bytes!("../../assets/icon.png");
    window::icon::from_file_data(icon_data, Some(image::ImageFormat::Png)).ok()
}
