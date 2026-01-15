//! Utility functions for the application.
//!
//! Contains:
//! - `load_study_async` - Async study loading
//! - `extract_domain_code` - Domain code extraction from filenames
//! - `load_app_icon` - Application icon loading

use std::path::PathBuf;

use iced::window;

use crate::state::{DomainSource, DomainState, Study};
use tss_standards::TerminologyRegistry;

/// Load a study asynchronously, including CT loading.
///
/// # Arguments
/// * `folder` - Path to the study folder containing CSV files
/// * `header_rows` - Number of header rows in CSV files
/// * `confidence_threshold` - Minimum confidence (0.0-1.0) for mapping suggestions
pub async fn load_study_async(
    folder: PathBuf,
    header_rows: usize,
    confidence_threshold: f32,
) -> Result<(Study, TerminologyRegistry), String> {
    // Create study from folder
    let mut study = Study::from_folder(folder.clone());

    // Discover CSV files
    let csv_files: Vec<PathBuf> = std::fs::read_dir(&folder)
        .map_err(|e| format!("Failed to read folder: {}", e))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .map(|ext| ext.eq_ignore_ascii_case("csv"))
                .unwrap_or(false)
        })
        .collect();

    if csv_files.is_empty() {
        return Err("No CSV files found in the selected folder".to_string());
    }

    // Load metadata if available
    study.metadata = tss_ingest::load_study_metadata(&folder, header_rows).ok();

    // Load SDTM-IG
    let ig_domains =
        tss_standards::load_sdtm_ig().map_err(|e| format!("Failed to load SDTM-IG: {}", e))?;

    // Load Controlled Terminology
    let ct_version = tss_standards::ct::CtVersion::default();
    let terminology = tss_standards::ct::load(ct_version).map_err(|e| {
        format!(
            "Failed to load Controlled Terminology ({}): {}",
            ct_version, e
        )
    })?;
    tracing::info!(
        "Loaded CT {} with {} catalogs",
        ct_version,
        terminology.catalogs.len()
    );

    // Process each CSV file
    for csv_path in csv_files {
        let file_stem = csv_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        // Extract domain code from filename
        // Handles both simple names (DM.csv) and prefixed names (STUDY_DM.csv)
        let domain_code = extract_domain_code(file_stem);

        // Skip non-domain files
        if domain_code.is_empty()
            || domain_code.starts_with('_')
            || domain_code.eq_ignore_ascii_case("items")
            || domain_code.eq_ignore_ascii_case("codelists")
        {
            continue;
        }

        let domain_code = domain_code.to_uppercase();

        // Load CSV
        let (df, _headers) = tss_ingest::read_csv_table(&csv_path, header_rows)
            .map_err(|e| format!("Failed to load {}: {}", domain_code, e))?;

        // Find domain in SDTM-IG
        let ig_domain = ig_domains
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(&domain_code));

        let Some(ig_domain) = ig_domain else {
            tracing::warn!("Domain {} not found in SDTM-IG, skipping", domain_code);
            continue;
        };

        // Create source
        let source = DomainSource::new(csv_path, df.clone(), ig_domain.label.clone());

        // Create mapping state
        let hints = tss_ingest::build_column_hints(&df);
        let source_columns: Vec<String> = df
            .get_column_names()
            .into_iter()
            .map(|s| s.to_string())
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
        study.add_domain(domain_code, domain);
    }

    if study.domain_count() == 0 {
        return Err("No valid SDTM domains found in the selected folder".to_string());
    }

    Ok((study, terminology))
}

/// Extract domain code from a filename.
///
/// Handles various naming conventions:
/// - Simple: `DM.csv` → `DM`
/// - Prefixed: `STUDY_DM.csv` → `DM`
/// - Full path: `DEMO_GDISC_20240903_072908_DM.csv` → `DM`
///
/// Returns the last underscore-separated segment.
pub fn extract_domain_code(file_stem: &str) -> &str {
    // If there's no underscore, return the whole string
    if !file_stem.contains('_') {
        return file_stem;
    }

    // Return the last segment after underscore
    file_stem.rsplit('_').next().unwrap_or(file_stem)
}

/// Load the application icon from embedded PNG data.
pub fn load_app_icon() -> Option<window::Icon> {
    let icon_data = include_bytes!("../../assets/icon.png");
    window::icon::from_file_data(icon_data, Some(image::ImageFormat::Png)).ok()
}
