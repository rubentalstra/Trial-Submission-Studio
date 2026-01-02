//! Preview generation service
//!
//! This module provides lazy preview computation for SDTM data transformation.
//!
//! ## Architecture
//!
//! Preview computation follows a lazy async pattern:
//!
//! 1. When mappings change → `invalidate_preview()` clears cached data (instant)
//! 2. Consumer tabs call `ensure_preview()` to get current state
//! 3. If preview is missing, background thread spawns to rebuild
//! 4. Results flow back via channel to `CdiscApp::handle_preview_results()`
//!
//! ## Key Types
//!
//! - [`PreviewState`] - Current state for consumer tabs to handle
//! - [`PreviewResult`] - Message from background thread with computed data
//! - [`ensure_preview`] - Main entry point for consumer tabs

use crate::state::AppState;
use cdisc_standards::{CtVersion, load_ct};
use cdisc_transform::build_preview_dataframe_with_dm_and_omitted;
use polars::prelude::DataFrame;
use std::collections::{BTreeMap, BTreeSet};

// ============================================================================
// Preview Result (sent from background thread)
// ============================================================================

/// Result from background preview computation.
///
/// Sent via channel from background thread to main UI thread.
pub struct PreviewResult {
    /// Domain this preview is for
    pub domain_code: String,
    /// The computed preview DataFrame or error message
    pub result: Result<DataFrame, String>,
}

// ============================================================================
// Preview State (returned to consumer tabs)
// ============================================================================

/// Current preview state for consumer tabs.
///
/// Consumer tabs (Preview, Transform, Validation) call `ensure_preview()`
/// and then match on this enum to decide what to render.
#[derive(Debug)]
pub enum PreviewState {
    /// Preview is being rebuilt in background thread.
    /// Tab should show spinner and call `ctx.request_repaint()`.
    Rebuilding,
    /// Preview is ready to use.
    /// Tab can access it via `state.domain(code).derived.preview`.
    Ready,
    /// No preview available because no mappings are configured.
    /// Tab should show helpful message about configuring mappings.
    NotConfigured,
    /// Preview failed to build.
    /// Tab should show error message with the contained string.
    Error(String),
}

// ============================================================================
// Main API
// ============================================================================

/// Ensure preview is available, spawning rebuild if needed.
///
/// This is the main entry point for consumer tabs (Preview, Transform, Validation).
/// Call this at the start of your `show()` function and match on the result.
///
/// # Example
///
/// ```ignore
/// match ensure_preview(state, domain_code) {
///     PreviewState::Rebuilding => {
///         ui.spinner();
///         ui.ctx().request_repaint();
///         return;
///     }
///     PreviewState::NotConfigured => {
///         ui.label("Configure mappings to see preview");
///         return;
///     }
///     PreviewState::Error(e) => {
///         ui.label(format!("Error: {}", e));
///         return;
///     }
///     PreviewState::Ready => {
///         // Continue to render content
///     }
/// }
/// // Now safe to access: state.domain(code).derived.preview.as_ref().unwrap()
/// ```
pub fn ensure_preview(state: &mut AppState, domain_code: &str) -> PreviewState {
    // Check if already rebuilding
    let is_rebuilding = state
        .ui
        .get_domain_editor(domain_code)
        .map(|ui| ui.preview.is_rebuilding)
        .unwrap_or(false);

    if is_rebuilding {
        return PreviewState::Rebuilding;
    }

    // Check for cached error
    if let Some(error) = state
        .ui
        .get_domain_editor(domain_code)
        .and_then(|ui| ui.preview.error.clone())
    {
        return PreviewState::Error(error);
    }

    // Check if preview exists
    let has_preview = state
        .domain(domain_code)
        .map(|d| d.derived.preview.is_some())
        .unwrap_or(false);

    if has_preview {
        return PreviewState::Ready;
    }

    // Preview doesn't exist - check if we can build it
    let has_mappings = state
        .domain(domain_code)
        .map(|d| !d.mapping.all_accepted().is_empty())
        .unwrap_or(false);

    if !has_mappings {
        return PreviewState::NotConfigured;
    }

    // Start async rebuild
    spawn_preview_rebuild(state, domain_code);
    PreviewState::Rebuilding
}

// ============================================================================
// Internal Implementation
// ============================================================================

/// Spawn background thread to rebuild preview.
///
/// Extracts all necessary data from state, spawns a thread, and sends
/// result via channel when complete.
fn spawn_preview_rebuild(state: &mut AppState, domain_code: &str) {
    // Mark as rebuilding
    state.ui.domain_editor(domain_code).preview.is_rebuilding = true;
    state.ui.domain_editor(domain_code).preview.error = None;

    // Extract all data needed for computation (clone to move into thread)
    let task_data = {
        let Some(study) = state.study() else { return };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };

        let ms = &domain.mapping;
        let mappings: BTreeMap<String, String> = ms
            .all_accepted()
            .iter()
            .map(|(var, (col, _))| (var.clone(), col.clone()))
            .collect();
        let omitted: BTreeSet<String> = ms.all_omitted().clone();
        let sdtm_domain = ms.domain().clone();
        let source_df = domain.source.data.clone();
        let study_id = study.study_id.clone();

        // Get DM preview for RFSTDTC derivation (if not DM domain)
        let dm_df = if !domain_code.eq_ignore_ascii_case("DM") {
            study.dm_preview_data().cloned()
        } else {
            None
        };

        let ct = load_ct(CtVersion::default()).ok();

        (
            mappings,
            omitted,
            sdtm_domain,
            source_df,
            study_id,
            dm_df,
            ct,
        )
    };

    let sender = state.preview_sender.clone();
    let domain_code_owned = domain_code.to_string();

    // Spawn background thread
    std::thread::spawn(move || {
        let (mappings, omitted, sdtm_domain, source_df, study_id, dm_df, ct) = task_data;

        let result = build_preview_dataframe_with_dm_and_omitted(
            &source_df,
            &mappings,
            &omitted,
            &sdtm_domain,
            &study_id,
            dm_df.as_ref(),
            ct.as_ref(),
        );

        let _ = sender.send(PreviewResult {
            domain_code: domain_code_owned,
            result: result.map_err(|e| e.to_string()),
        });
    });
}
