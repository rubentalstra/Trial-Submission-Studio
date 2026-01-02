//! GUI mapping service
//!
//! Provides GUI-specific mapping state with CT caching for display.

use polars::prelude::DataFrame;
use sdtm_ingest::{build_column_hints, get_sample_values};
use sdtm_map::ColumnHint;
use sdtm_model::{Domain, Variable};
use sdtm_standards::load_default_ct_registry;
use std::collections::{BTreeMap, BTreeSet};

pub use sdtm_map::VariableStatus;

/// Pre-fetched codelist display info for UI rendering.
#[derive(Debug, Clone)]
pub struct CodelistDisplayInfo {
    #[allow(dead_code)] // May be used for debugging/display
    pub code: String,
    pub name: String,
    pub extensible: bool,
    /// (submission_value, definition) - limited to 8 terms for display
    pub terms: Vec<(String, Option<String>)>,
    pub total_terms: usize,
    pub found: bool,
    /// Lookup map: uppercase(synonym or submission_value) → submission_value
    /// Used for normalization preview in the UI.
    pub lookup: BTreeMap<String, String>,
}

/// GUI mapping state with CT caching and UI state.
#[derive(Debug, Clone)]
pub struct MappingState {
    pub inner: sdtm_map::MappingState,
    pub ct_cache: BTreeMap<String, CodelistDisplayInfo>,
    /// UI state: currently selected variable index
    pub selected_variable_idx: Option<usize>,
    /// UI state: search filter text
    pub search_filter: String,
    /// UI state: reason text being edited for "not collected" (per variable)
    #[allow(dead_code)] // Planned feature for custom "not collected" reasons
    pub not_collected_reason_edit: BTreeMap<String, String>,
}

impl std::ops::Deref for MappingState {
    type Target = sdtm_map::MappingState;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for MappingState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl MappingState {
    pub fn new(
        sdtm_domain: Domain,
        study_id: &str,
        source_columns: &[String],
        column_hints: BTreeMap<String, ColumnHint>,
    ) -> Self {
        let inner =
            sdtm_map::MappingState::new(sdtm_domain, study_id, source_columns, column_hints, 0.6);
        let ct_cache = load_ct_cache(inner.domain());
        Self {
            inner,
            ct_cache,
            selected_variable_idx: None,
            search_filter: String::new(),
            not_collected_reason_edit: BTreeMap::new(),
        }
    }

    /// Get filtered variables based on search text.
    pub fn filtered_variables(&self) -> Vec<(usize, &Variable)> {
        let filter = self.search_filter.to_lowercase();
        self.inner
            .domain()
            .variables
            .iter()
            .enumerate()
            .filter(|(_, v)| {
                if filter.is_empty() {
                    true
                } else {
                    let matches_name = v.name.to_lowercase().contains(&filter);
                    let matches_label = v
                        .label
                        .as_ref()
                        .is_some_and(|l| l.to_lowercase().contains(&filter));
                    let matches_subjid = v.name.eq_ignore_ascii_case("USUBJID")
                        && (filter.contains("subjid")
                            || filter.contains("subject id")
                            || filter.contains("subject"));
                    matches_name || matches_label || matches_subjid
                }
            })
            .collect()
    }

    /// Get the currently selected variable.
    pub fn selected_variable(&self) -> Option<&Variable> {
        self.selected_variable_idx
            .and_then(|idx| self.inner.domain().variables.get(idx))
    }

    /// Get number of suggestions (for summary display).
    pub fn suggestions_count(&self) -> usize {
        self.inner.all_suggestions().len()
    }

    /// Get CT display info for a variable's codelist codes.
    pub fn get_ct_for_variable(&self, codelist_codes: &str) -> Vec<&CodelistDisplayInfo> {
        codelist_codes
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter_map(|code| self.ct_cache.get(code))
            .collect()
    }
}

/// Load CT data for all codelists referenced by domain variables.
fn load_ct_cache(domain: &Domain) -> BTreeMap<String, CodelistDisplayInfo> {
    let codelist_codes: BTreeSet<_> = domain
        .variables
        .iter()
        .filter_map(|v| v.codelist_code.as_ref())
        .flat_map(|codes| codes.split(';').map(str::trim))
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    if codelist_codes.is_empty() {
        return BTreeMap::new();
    }

    let Ok(registry) = load_default_ct_registry() else {
        tracing::warn!("Failed to load CT registry for domain {}", domain.name);
        return BTreeMap::new();
    };

    let cache: BTreeMap<_, _> = codelist_codes
        .into_iter()
        .map(|code| {
            let info = match registry.resolve(&code, None) {
                Some(resolved) => {
                    let cl = resolved.codelist;
                    let terms: Vec<_> = cl
                        .terms
                        .values()
                        .take(8)
                        .map(|t| (t.submission_value.clone(), t.definition.clone()))
                        .collect();

                    // Build lookup map: uppercase(synonym or submission_value) → submission_value
                    let mut lookup = BTreeMap::new();
                    for term in cl.terms.values() {
                        let sv = &term.submission_value;
                        // Map the submission value itself
                        lookup.insert(sv.to_uppercase(), sv.clone());
                        // Map all synonyms
                        for syn in &term.synonyms {
                            lookup.insert(syn.to_uppercase(), sv.clone());
                        }
                    }

                    CodelistDisplayInfo {
                        code: code.clone(),
                        name: cl.name.clone(),
                        extensible: cl.extensible,
                        terms,
                        total_terms: cl.terms.len(),
                        found: true,
                        lookup,
                    }
                }
                None => CodelistDisplayInfo {
                    code: code.clone(),
                    name: String::new(),
                    extensible: false,
                    terms: Vec::new(),
                    total_terms: 0,
                    found: false,
                    lookup: BTreeMap::new(),
                },
            };
            (code, info)
        })
        .collect();

    tracing::info!(
        "Pre-loaded {} codelists for domain {}",
        cache.len(),
        domain.name
    );
    cache
}

/// GUI extension trait for VariableStatus icons.
pub trait VariableStatusIcon {
    fn icon(&self) -> &'static str;
}

impl VariableStatusIcon for VariableStatus {
    fn icon(&self) -> &'static str {
        match self {
            Self::Accepted => egui_phosphor::regular::CHECK,
            Self::Suggested => egui_phosphor::regular::CIRCLE_DASHED,
            Self::NotCollected => egui_phosphor::regular::PROHIBIT,
            Self::Omitted => egui_phosphor::regular::MINUS_CIRCLE,
            Self::Unmapped => egui_phosphor::regular::MINUS,
        }
    }
}

/// Service for column mapping operations.
pub struct MappingService;

impl MappingService {
    /// Create a new mapping state.
    pub fn create_mapping_state(
        sdtm_domain: Domain,
        study_id: &str,
        source_columns: &[String],
        column_hints: BTreeMap<String, ColumnHint>,
    ) -> MappingState {
        MappingState::new(sdtm_domain, study_id, source_columns, column_hints)
    }

    /// Extract column hints from a DataFrame.
    pub fn extract_column_hints(df: &DataFrame) -> BTreeMap<String, ColumnHint> {
        build_column_hints(df)
    }

    /// Get sample values from a DataFrame column.
    pub fn get_sample_values(df: &DataFrame, column: &str, limit: usize) -> Vec<String> {
        get_sample_values(df, column, limit)
    }
}
