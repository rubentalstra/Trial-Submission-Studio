//! Mapping service for the GUI
//!
//! Provides column-to-variable mapping functionality using sdtm-map,
//! with support for variable-centric mapping workflow.

// Fields/methods used for state serialization and future features
#![allow(dead_code)]

use polars::prelude::DataFrame;
use sdtm_map::{MappingEngine, MappingResult};
use sdtm_model::{ColumnHint, Domain, MappingConfig, MappingSuggestion, Variable};
use sdtm_standards::load_default_ct_registry;
use std::collections::BTreeMap;

/// Pre-fetched codelist display info (loaded once when domain opens)
#[derive(Debug, Clone)]
pub struct CodelistDisplayInfo {
    pub code: String,
    pub name: String,
    pub extensible: bool,
    /// (submission_value, truncated_definition) - limited to 8 terms
    pub terms: Vec<(String, Option<String>)>,
    pub total_terms: usize,
    pub found: bool,
}

/// State of a mapping operation for a single domain
#[derive(Debug, Clone)]
pub struct MappingState {
    /// Domain code
    pub domain_code: String,
    /// Study ID
    pub study_id: String,
    /// SDTM domain definition (for variable metadata)
    pub sdtm_domain: Domain,
    /// All mapping suggestions from the engine
    pub suggestions: Vec<MappingSuggestion>,
    /// Accepted mappings (variable_name -> source_column, confidence)
    pub accepted: BTreeMap<String, (String, f32)>,
    /// Source columns that couldn't be mapped
    pub unmapped_columns: Vec<String>,
    /// UI state: currently selected variable index
    pub selected_variable_idx: Option<usize>,
    /// UI state: search filter text
    pub search_filter: String,
    /// Column hints from source data
    pub column_hints: BTreeMap<String, ColumnHint>,
    /// Pre-loaded CT data (codelist_code -> display info)
    pub ct_cache: BTreeMap<String, CodelistDisplayInfo>,
}

impl MappingState {
    /// Create a new mapping state and pre-load CT data
    pub fn new(
        domain_code: &str,
        study_id: &str,
        sdtm_domain: Domain,
        result: MappingResult,
        column_hints: BTreeMap<String, ColumnHint>,
    ) -> Self {
        // Pre-load CT data for all codelists referenced by variables
        let ct_cache = Self::load_ct_cache(&sdtm_domain);

        Self {
            domain_code: domain_code.to_string(),
            study_id: study_id.to_string(),
            sdtm_domain,
            suggestions: result.mappings,
            accepted: BTreeMap::new(),
            unmapped_columns: result.unmapped_columns,
            selected_variable_idx: None,
            search_filter: String::new(),
            column_hints,
            ct_cache,
        }
    }

    /// Pre-load CT data for all codelists referenced by domain variables
    fn load_ct_cache(domain: &Domain) -> BTreeMap<String, CodelistDisplayInfo> {
        let mut cache = BTreeMap::new();

        // Collect all unique codelist codes from variables
        let mut codelist_codes: Vec<String> = Vec::new();
        for var in &domain.variables {
            if let Some(ref codes_str) = var.codelist_code {
                for code in codes_str.split(';').map(|s| s.trim()) {
                    if !code.is_empty() && !codelist_codes.contains(&code.to_string()) {
                        codelist_codes.push(code.to_string());
                    }
                }
            }
        }

        if codelist_codes.is_empty() {
            return cache;
        }

        // Load registry once and resolve all codelists
        let Ok(registry) = load_default_ct_registry() else {
            tracing::warn!("Failed to load CT registry for domain {}", domain.code);
            return cache;
        };

        for code in codelist_codes {
            let info = if let Some(resolved) = registry.resolve(&code, None) {
                let cl = resolved.codelist;
                // Pre-extract only what we need (limit to 8 terms)
                let terms: Vec<(String, Option<String>)> = cl
                    .terms
                    .values()
                    .take(8)
                    .map(|t| {
                        let def = t.definition.as_ref().map(|d| {
                            if d.len() > 40 {
                                format!("{}...", &d[..37])
                            } else {
                                d.clone()
                            }
                        });
                        (t.submission_value.clone(), def)
                    })
                    .collect();

                CodelistDisplayInfo {
                    code: code.clone(),
                    name: cl.name.clone(),
                    extensible: cl.extensible,
                    terms,
                    total_terms: cl.terms.len(),
                    found: true,
                }
            } else {
                CodelistDisplayInfo {
                    code: code.clone(),
                    name: String::new(),
                    extensible: false,
                    terms: Vec::new(),
                    total_terms: 0,
                    found: false,
                }
            };

            cache.insert(code, info);
        }

        tracing::info!(
            "Pre-loaded {} codelists for domain {}",
            cache.len(),
            domain.code
        );
        cache
    }

    /// Get CT display info for a variable's codelist codes
    pub fn get_ct_for_variable(&self, codelist_codes: &str) -> Vec<&CodelistDisplayInfo> {
        codelist_codes
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter_map(|code| self.ct_cache.get(code))
            .collect()
    }

    /// Get filtered variables based on search text
    pub fn filtered_variables(&self) -> Vec<(usize, &Variable)> {
        let filter = self.search_filter.to_lowercase();
        self.sdtm_domain
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

    /// Get the currently selected variable
    pub fn selected_variable(&self) -> Option<&Variable> {
        self.selected_variable_idx
            .and_then(|idx| self.sdtm_domain.variables.get(idx))
    }

    /// Get suggestion for a specific variable
    pub fn get_suggestion_for(&self, variable_name: &str) -> Option<&MappingSuggestion> {
        self.suggestions
            .iter()
            .find(|s| s.target_variable.eq_ignore_ascii_case(variable_name))
    }

    /// Get accepted mapping for a variable
    pub fn get_accepted_for(&self, variable_name: &str) -> Option<(&str, f32)> {
        self.accepted
            .get(variable_name)
            .map(|(col, conf)| (col.as_str(), *conf))
    }

    /// Get mapping status for a variable
    pub fn variable_status(&self, variable_name: &str) -> VariableMappingStatus {
        if self.accepted.contains_key(variable_name) {
            VariableMappingStatus::Accepted
        } else if self.get_suggestion_for(variable_name).is_some() {
            VariableMappingStatus::Suggested
        } else {
            VariableMappingStatus::Unmapped
        }
    }

    /// Accept the suggestion for a variable
    pub fn accept_suggestion(&mut self, variable_name: &str) -> bool {
        if let Some(suggestion) = self.get_suggestion_for(variable_name).cloned() {
            self.accepted.insert(
                variable_name.to_string(),
                (suggestion.source_column.clone(), suggestion.confidence),
            );
            true
        } else {
            false
        }
    }

    /// Accept a manual mapping for a variable
    pub fn accept_manual(&mut self, variable_name: &str, source_column: &str) {
        self.accepted.insert(
            variable_name.to_string(),
            (source_column.to_string(), 1.0),
        );
        // Remove from unmapped if present
        self.unmapped_columns.retain(|c| c != source_column);
    }

    /// Clear the mapping for a variable
    pub fn clear_mapping(&mut self, variable_name: &str) -> bool {
        if let Some((source_col, _)) = self.accepted.remove(variable_name) {
            // Add back to unmapped if not suggested elsewhere
            if !self
                .suggestions
                .iter()
                .any(|s| s.source_column == source_col)
            {
                self.unmapped_columns.push(source_col);
            }
            true
        } else {
            false
        }
    }

    /// Get all source columns (mapped and unmapped)
    pub fn all_source_columns(&self) -> Vec<&str> {
        let mut cols: Vec<&str> = self.column_hints.keys().map(String::as_str).collect();
        cols.sort();
        cols
    }

    /// Check if a source column is already used
    pub fn is_column_used(&self, column: &str) -> bool {
        self.accepted.values().any(|(c, _)| c == column)
    }

    /// Get available (unused) source columns
    pub fn available_columns(&self) -> Vec<&str> {
        self.all_source_columns()
            .into_iter()
            .filter(|c| !self.is_column_used(c))
            .collect()
    }

    /// Get summary counts
    pub fn summary(&self) -> MappingSummary {
        let required_count = self
            .sdtm_domain
            .variables
            .iter()
            .filter(|v| v.core.as_deref() == Some("Req"))
            .count();
        let required_mapped = self
            .sdtm_domain
            .variables
            .iter()
            .filter(|v| v.core.as_deref() == Some("Req") && self.accepted.contains_key(&v.name))
            .count();

        MappingSummary {
            total_variables: self.sdtm_domain.variables.len(),
            mapped: self.accepted.len(),
            suggested: self
                .suggestions
                .iter()
                .filter(|s| !self.accepted.contains_key(&s.target_variable))
                .count(),
            required_total: required_count,
            required_mapped,
        }
    }

    /// Convert to final MappingConfig
    pub fn to_config(&self) -> MappingConfig {
        let mappings: Vec<MappingSuggestion> = self
            .accepted
            .iter()
            .map(|(var, (col, conf))| MappingSuggestion {
                source_column: col.clone(),
                target_variable: var.clone(),
                confidence: *conf,
                transformation: None,
            })
            .collect();

        MappingConfig {
            domain_code: self.domain_code.clone(),
            study_id: self.study_id.clone(),
            mappings,
            unmapped_columns: self.unmapped_columns.clone(),
        }
    }
}

/// Status of a variable's mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableMappingStatus {
    /// Has an accepted mapping
    Accepted,
    /// Has a suggestion but not yet accepted
    Suggested,
    /// No mapping or suggestion
    Unmapped,
}

impl VariableMappingStatus {
    /// Get status icon (phosphor icon)
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Accepted => egui_phosphor::regular::CHECK,
            Self::Suggested => egui_phosphor::regular::CIRCLE_DASHED,
            Self::Unmapped => egui_phosphor::regular::MINUS,
        }
    }
}

/// Summary of mapping counts
#[derive(Debug, Clone, Copy)]
pub struct MappingSummary {
    pub total_variables: usize,
    pub mapped: usize,
    pub suggested: usize,
    pub required_total: usize,
    pub required_mapped: usize,
}

/// Service for generating and managing column mappings
pub struct MappingService;

impl MappingService {
    /// Generate mapping suggestions for a domain
    pub fn generate_suggestions(
        domain: &Domain,
        source_columns: &[String],
        column_hints: &BTreeMap<String, ColumnHint>,
        min_confidence: f32,
    ) -> MappingResult {
        let engine = MappingEngine::new(domain.clone(), min_confidence, column_hints.clone());
        engine.suggest(source_columns)
    }

    /// Create a new mapping state
    pub fn create_mapping_state(
        sdtm_domain: Domain,
        study_id: &str,
        source_columns: &[String],
        column_hints: BTreeMap<String, ColumnHint>,
    ) -> MappingState {
        let result =
            Self::generate_suggestions(&sdtm_domain, source_columns, &column_hints, 0.6);
        let domain_code = sdtm_domain.code.clone();
        MappingState::new(&domain_code, study_id, sdtm_domain, result, column_hints)
    }

    /// Extract column hints from a DataFrame
    pub fn extract_column_hints(df: &DataFrame) -> BTreeMap<String, ColumnHint> {
        let mut hints = BTreeMap::new();
        let row_count = df.height();

        for name in df.get_column_names() {
            if let Ok(col) = df.column(name) {
                let null_count = col.null_count();
                let null_ratio = if row_count > 0 {
                    null_count as f64 / row_count as f64
                } else {
                    0.0
                };

                let is_numeric = matches!(
                    col.dtype(),
                    polars::datatypes::DataType::Int8
                        | polars::datatypes::DataType::Int16
                        | polars::datatypes::DataType::Int32
                        | polars::datatypes::DataType::Int64
                        | polars::datatypes::DataType::UInt8
                        | polars::datatypes::DataType::UInt16
                        | polars::datatypes::DataType::UInt32
                        | polars::datatypes::DataType::UInt64
                        | polars::datatypes::DataType::Float32
                        | polars::datatypes::DataType::Float64
                );

                let unique_ratio = if row_count > 0 {
                    if let Ok(unique) = col.n_unique() {
                        unique as f64 / row_count as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                hints.insert(
                    name.to_string(),
                    ColumnHint {
                        is_numeric,
                        null_ratio,
                        unique_ratio,
                        label: None,
                    },
                );
            }
        }

        hints
    }

    /// Get sample values from a column
    pub fn get_sample_values(df: &DataFrame, column: &str, limit: usize) -> Vec<String> {
        let Ok(col) = df.column(column) else {
            return Vec::new();
        };

        let mut samples = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Try to get unique sample values
        if let Ok(str_col) = col.str() {
            for i in 0..df.height().min(limit * 3) {
                if let Some(val) = str_col.get(i) {
                    if !val.is_empty() && seen.insert(val.to_string()) {
                        samples.push(val.to_string());
                        if samples.len() >= limit {
                            break;
                        }
                    }
                }
            }
        } else {
            // For non-string columns, format as string using Display (not Debug)
            for i in 0..df.height().min(limit) {
                if let Ok(val) = col.get(i) {
                    // Use Display formatting which gives clean output without type info
                    let formatted = format!("{}", val);
                    if formatted != "null" && !formatted.is_empty() && seen.insert(formatted.clone()) {
                        samples.push(formatted);
                        if samples.len() >= limit {
                            break;
                        }
                    }
                }
            }
        }

        samples
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use sdtm_model::VariableType;

    fn make_variable(name: &str, label: &str, core: Option<&str>) -> Variable {
        Variable {
            name: name.to_string(),
            label: Some(label.to_string()),
            data_type: VariableType::Char,
            length: None,
            role: None,
            core: core.map(String::from),
            codelist_code: None,
            order: None,
        }
    }

    fn test_domain() -> Domain {
        Domain {
            code: "DM".to_string(),
            description: Some("Demographics".to_string()),
            class_name: Some("SPECIAL PURPOSE".to_string()),
            dataset_class: None,
            label: Some("Demographics".to_string()),
            structure: None,
            dataset_name: None,
            variables: vec![
                make_variable("STUDYID", "Study Identifier", Some("Req")),
                make_variable("USUBJID", "Unique Subject Identifier", Some("Req")),
                make_variable("AGE", "Age", Some("Exp")),
            ],
        }
    }

    #[test]
    fn test_mapping_state_accept_suggestion() {
        let domain = test_domain();
        let columns = vec!["AGE".to_string()];
        let hints = BTreeMap::new();

        let mut state =
            MappingService::create_mapping_state(domain, "STUDY01", &columns, hints);

        // Should have suggestion for AGE
        assert!(state.get_suggestion_for("AGE").is_some());

        // Accept it
        state.accept_suggestion("AGE");
        assert!(state.get_accepted_for("AGE").is_some());
        assert_eq!(state.variable_status("AGE"), VariableMappingStatus::Accepted);
    }

    #[test]
    fn test_mapping_state_clear() {
        let domain = test_domain();
        let columns = vec!["AGE".to_string()];
        let hints = BTreeMap::new();

        let mut state =
            MappingService::create_mapping_state(domain, "STUDY01", &columns, hints);

        state.accept_suggestion("AGE");
        assert!(state.get_accepted_for("AGE").is_some());

        state.clear_mapping("AGE");
        assert!(state.get_accepted_for("AGE").is_none());
    }

    #[test]
    fn test_extract_column_hints() {
        let df = DataFrame::new(vec![
            Series::new("NAME".into(), vec!["Alice", "Bob", "Charlie"]).into(),
            Series::new("AGE".into(), vec![25i64, 30, 35]).into(),
        ])
        .unwrap();

        let hints = MappingService::extract_column_hints(&df);

        assert!(hints.contains_key("NAME"));
        assert!(hints.contains_key("AGE"));
        assert!(!hints["NAME"].is_numeric);
        assert!(hints["AGE"].is_numeric);
    }

    #[test]
    fn test_ct_variable_mapping_and_detection() {
        // Create domain with DTHFL that has a codelist code
        let domain = Domain {
            code: "DM".to_string(),
            description: Some("Demographics".to_string()),
            class_name: None,
            dataset_class: None,
            label: None,
            structure: None,
            dataset_name: None,
            variables: vec![
                Variable {
                    name: "DTHFL".to_string(),
                    label: Some("Subject Death Flag".to_string()),
                    data_type: VariableType::Char,
                    length: None,
                    role: Some("Record Qualifier".to_string()),
                    core: Some("Exp".to_string()),
                    codelist_code: Some("C66742".to_string()),
                    order: None,
                },
            ],
        };

        let columns = vec!["DEATH_FLAG".to_string()];
        let hints = BTreeMap::new();

        let mut state = MappingService::create_mapping_state(domain, "STUDY01", &columns, hints);

        // Manually accept a mapping for DTHFL
        state.accept_manual("DTHFL", "DEATH_FLAG");

        // Verify the mapping is stored correctly
        let accepted = state.get_accepted_for("DTHFL");
        assert!(
            accepted.is_some(),
            "DTHFL should be in accepted mappings after accept_manual"
        );
        assert_eq!(accepted.unwrap().0, "DEATH_FLAG");

        // Verify we can find the variable with codelist
        let dthfl_var = state
            .sdtm_domain
            .variables
            .iter()
            .find(|v| v.name == "DTHFL");
        assert!(dthfl_var.is_some());
        assert_eq!(
            dthfl_var.unwrap().codelist_code.as_deref(),
            Some("C66742")
        );

        // Now check if the transform detection would work
        // This simulates what rebuild_transforms_if_needed does
        let ct_transforms: Vec<_> = state
            .sdtm_domain
            .variables
            .iter()
            .filter(|v| v.codelist_code.is_some())
            .filter(|v| state.get_accepted_for(&v.name).is_some())
            .map(|v| (v.name.clone(), v.codelist_code.clone()))
            .collect();

        assert!(
            !ct_transforms.is_empty(),
            "Should detect CT normalization for DTHFL"
        );
        assert_eq!(ct_transforms[0].0, "DTHFL");
        assert_eq!(ct_transforms[0].1, Some("C66742".to_string()));
    }

    #[test]
    fn test_ct_detection_with_loaded_dm_domain() {
        use sdtm_standards::load_default_sdtm_ig_domains;

        // Load the actual DM domain from standards
        let domains = load_default_sdtm_ig_domains().expect("load domains");
        let dm = domains
            .into_iter()
            .find(|d| d.code == "DM")
            .expect("DM domain");

        // Verify DTHFL has codelist_code
        let dthfl = dm
            .variables
            .iter()
            .find(|v| v.name == "DTHFL")
            .expect("DTHFL");
        assert_eq!(dthfl.codelist_code.as_deref(), Some("C66742"));

        // Create mapping state
        let columns = vec![
            "DEATH_FLAG".to_string(),
            "SEX".to_string(),
            "RACE".to_string(),
        ];
        let hints = BTreeMap::new();
        let mut state = MappingService::create_mapping_state(dm, "STUDY01", &columns, hints);

        // Accept mappings for CT variables
        state.accept_manual("DTHFL", "DEATH_FLAG");
        state.accept_manual("SEX", "SEX");
        state.accept_manual("RACE", "RACE");

        // Verify all three are accepted
        assert!(state.get_accepted_for("DTHFL").is_some());
        assert!(state.get_accepted_for("SEX").is_some());
        assert!(state.get_accepted_for("RACE").is_some());

        // Detect CT transforms
        let ct_transforms: Vec<_> = state
            .sdtm_domain
            .variables
            .iter()
            .filter(|v| v.codelist_code.is_some())
            .filter(|v| state.get_accepted_for(&v.name).is_some())
            .map(|v| v.name.clone())
            .collect();

        // Should detect DTHFL, SEX, RACE (all have CT codelists and are accepted)
        assert!(
            ct_transforms.contains(&"DTHFL".to_string()),
            "DTHFL should be detected. Got: {:?}",
            ct_transforms
        );
        assert!(
            ct_transforms.contains(&"SEX".to_string()),
            "SEX should be detected. Got: {:?}",
            ct_transforms
        );
        assert!(
            ct_transforms.contains(&"RACE".to_string()),
            "RACE should be detected. Got: {:?}",
            ct_transforms
        );
    }
}
