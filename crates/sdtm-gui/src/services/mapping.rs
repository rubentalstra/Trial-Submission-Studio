//! GUI mapping service
//!
//! Provides GUI-specific mapping state with CT caching for display.

use polars::prelude::DataFrame;
use sdtm_ingest::{build_column_hints, get_sample_values};
use sdtm_model::{ColumnHint, Domain};
use sdtm_standards::load_default_ct_registry;
use std::collections::{BTreeMap, BTreeSet};

pub use sdtm_map::VariableMappingStatus;

/// Pre-fetched codelist display info for UI rendering.
#[derive(Debug, Clone)]
pub struct CodelistDisplayInfo {
    pub code: String,
    pub name: String,
    pub extensible: bool,
    /// (submission_value, definition) - limited to 8 terms
    pub terms: Vec<(String, Option<String>)>,
    pub total_terms: usize,
    pub found: bool,
}

/// GUI mapping state with CT caching.
#[derive(Debug, Clone)]
pub struct MappingState {
    pub inner: sdtm_map::MappingState,
    pub ct_cache: BTreeMap<String, CodelistDisplayInfo>,
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
        let inner = sdtm_map::MappingState::from_domain(sdtm_domain, study_id, source_columns, column_hints, 0.6);
        let ct_cache = load_ct_cache(&inner.sdtm_domain);
        Self { inner, ct_cache }
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
        tracing::warn!("Failed to load CT registry for domain {}", domain.code);
        return BTreeMap::new();
    };

    let cache: BTreeMap<_, _> = codelist_codes
        .into_iter()
        .map(|code| {
            let info = match registry.resolve(&code, None) {
                Some(resolved) => {
                    let cl = resolved.codelist;
                    let terms = cl
                        .terms
                        .values()
                        .take(8)
                        .map(|t| (t.submission_value.clone(), t.definition.clone()))
                        .collect();

                    CodelistDisplayInfo {
                        code: code.clone(),
                        name: cl.name.clone(),
                        extensible: cl.extensible,
                        terms,
                        total_terms: cl.terms.len(),
                        found: true,
                    }
                }
                None => CodelistDisplayInfo {
                    code: code.clone(),
                    name: String::new(),
                    extensible: false,
                    terms: Vec::new(),
                    total_terms: 0,
                    found: false,
                },
            };
            (code, info)
        })
        .collect();

    tracing::info!("Pre-loaded {} codelists for domain {}", cache.len(), domain.code);
    cache
}

/// GUI extension trait for VariableMappingStatus icons.
pub trait VariableMappingStatusIcon {
    fn icon(&self) -> &'static str;
}

impl VariableMappingStatusIcon for VariableMappingStatus {
    fn icon(&self) -> &'static str {
        match self {
            Self::Accepted => egui_phosphor::regular::CHECK,
            Self::Suggested => egui_phosphor::regular::CIRCLE_DASHED,
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
