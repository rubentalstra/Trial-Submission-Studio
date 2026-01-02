//! Preview generation service
//!
//! Shared logic for generating preview DataFrames used by both
//! the Preview and Transform tabs.

use crate::state::{AppState, Versioned};
use polars::prelude::DataFrame;
use sdtm_standards::load_default_ct_registry;
use sdtm_transform::build_preview_dataframe_with_dm_and_omitted;
use std::collections::{BTreeMap, BTreeSet};

/// Ensure the preview is up-to-date for the given domain.
/// Returns true if the preview is ready, false if still building.
pub fn ensure_preview(state: &mut AppState, domain_code: &str) -> bool {
    let Some(domain) = state.domain(domain_code) else {
        return false;
    };

    let domain_version = domain.version;

    // Check if preview is stale and needs rebuilding
    let needs_rebuild = domain
        .derived
        .preview
        .as_ref()
        .map(|v| v.is_stale(domain_version))
        .unwrap_or(true);

    if needs_rebuild {
        rebuild_preview(state, domain_code);
        false
    } else {
        true
    }
}

/// Get the preview DataFrame if available and up-to-date.
pub fn get_preview(state: &AppState, domain_code: &str) -> Option<DataFrame> {
    let domain = state.domain(domain_code)?;
    let preview = domain.derived.preview.as_ref()?;

    if preview.is_stale(domain.version) {
        None
    } else {
        Some(preview.data.clone())
    }
}

/// Rebuild preview data for a domain.
pub fn rebuild_preview(state: &mut AppState, domain_code: &str) {
    // Extract the data we need for preview generation
    let preview_result = {
        let Some(study) = state.study() else {
            return;
        };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };

        let ms = &domain.mapping;

        // Build mappings BTreeMap from accepted mappings
        let mappings: BTreeMap<String, String> = ms
            .all_accepted()
            .iter()
            .map(|(var, (col, _))| (var.clone(), col.clone()))
            .collect();

        // Get omitted variables
        let omitted: BTreeSet<String> = ms.all_omitted().clone();

        // Get the SDTM domain definition and source data
        let sdtm_domain = ms.domain().clone();
        let source_df = domain.source.data.clone();
        let study_id = study.study_id.clone();

        // Get DM preview DataFrame if this is not DM
        let dm_df = if !domain_code.eq_ignore_ascii_case("DM") {
            study.dm_preview_data().cloned()
        } else {
            None
        };

        // Load CT registry
        let ct = load_default_ct_registry().ok();

        // Build the preview DataFrame
        build_preview_dataframe_with_dm_and_omitted(
            &source_df,
            &mappings,
            &omitted,
            &sdtm_domain,
            &study_id,
            dm_df.as_ref(),
            ct.as_ref(),
        )
    };

    // Store the result in state
    match preview_result {
        Ok(df) => {
            // Get current version
            let version = state
                .study
                .as_ref()
                .and_then(|s| s.get_domain(domain_code))
                .map(|d| d.version)
                .unwrap_or(0);

            // Store preview in derived state
            if let Some(domain) = state
                .study_mut()
                .and_then(|s| s.get_domain_mut(domain_code))
            {
                domain.derived.preview = Some(Versioned {
                    data: df,
                    source_version: version,
                });
            }

            // Clear error
            state.ui.domain_editor(domain_code).preview.error = None;
            state.ui.domain_editor(domain_code).preview.reset();

            // If this is DM, mark DM as ready
            if domain_code.eq_ignore_ascii_case("DM") {
                if let Some(study) = state.study_mut() {
                    study.set_dm_ready(version);
                }
            }
        }
        Err(e) => {
            // Store error
            state.ui.domain_editor(domain_code).preview.error = Some(format!("{}", e));
        }
    }
}
