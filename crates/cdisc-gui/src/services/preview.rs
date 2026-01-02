//! Preview generation service
//!
//! Shared logic for generating preview DataFrames.

use crate::state::AppState;
use cdisc_standards::{CtVersion, load_ct};
use cdisc_transform::build_preview_dataframe_with_dm_and_omitted;
use std::collections::{BTreeMap, BTreeSet};

/// Rebuild preview data for a domain.
///
/// This should be called after mapping changes to keep preview up-to-date.
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

        // Get DM preview DataFrame if this is not DM (for RFSTDTC)
        let dm_df = if !domain_code.eq_ignore_ascii_case("DM") {
            study.dm_preview_data().cloned()
        } else {
            None
        };

        // Load CT registry
        let ct = load_ct(CtVersion::default()).ok();

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
            // Store preview directly (no versioning)
            if let Some(domain) = state
                .study_mut()
                .and_then(|s| s.get_domain_mut(domain_code))
            {
                domain.derived.preview = Some(df);
            }

            // Clear error
            state.ui.domain_editor(domain_code).preview.error = None;
            state.ui.domain_editor(domain_code).preview.reset();
        }
        Err(e) => {
            // Store error
            state.ui.domain_editor(domain_code).preview.error = Some(format!("{}", e));
        }
    }
}
