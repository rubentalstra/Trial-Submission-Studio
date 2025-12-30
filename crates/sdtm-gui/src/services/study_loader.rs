//! Study loading service
//!
//! Loads a study folder and discovers domains.

use crate::state::{DomainState, StudyState};
use anyhow::{Context, Result};
use sdtm_ingest::{discover_domain_files, list_csv_files, load_study_metadata, read_csv_table};
use sdtm_standards::load_default_sdtm_ig_domains;
use std::path::Path;

/// Service for loading studies from disk
pub struct StudyLoader;

impl StudyLoader {
    /// Load a study from a folder
    ///
    /// Discovers CSV files and matches them to SDTM domains.
    pub fn load_study(study_folder: &Path) -> Result<StudyState> {
        // Create the study state
        let mut study = StudyState::new(study_folder.to_path_buf());

        // Get supported domain codes from SDTM-IG
        let domains =
            load_default_sdtm_ig_domains().context("Failed to load SDTM-IG domain definitions")?;
        let domain_codes: Vec<String> = domains.iter().map(|d| d.code.clone()).collect();

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
        match load_study_metadata(study_folder) {
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
                match read_csv_table(file_path) {
                    Ok(df) => {
                        tracing::info!(
                            "Loaded domain {} from {} ({} rows, {} columns)",
                            domain_code,
                            file_path.display(),
                            df.height(),
                            df.width()
                        );

                        let domain_state = DomainState::new(file_path.clone(), df);
                        study.domains.insert(domain_code, domain_state);
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
