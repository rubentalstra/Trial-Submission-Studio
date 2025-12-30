//! Transformation execution context and results.
//!
//! Provides runtime context for executing transformations and
//! structures for collecting results and diagnostics.

use crate::pipeline::{DomainPipeline, TransformRule, TransformType};
use std::collections::HashMap;
use std::sync::Arc;

/// Runtime context for executing transformations.
///
/// Contains all the information needed during transformation execution,
/// including study-level constants and controlled terminology lookups.
#[derive(Debug, Clone)]
pub struct TransformContext {
    /// The STUDYID value to use.
    pub study_id: String,

    /// The reference start date column (typically "RFSTDTC" from DM).
    /// Used for study day calculations.
    pub reference_start_date: Option<String>,

    /// CT resolution mode.
    pub ct_mode: CtResolutionMode,

    /// Loaded controlled terminology for lookups.
    /// Key: codelist code, Value: map of submission_value -> submission_value (normalized).
    pub ct_lookup: Arc<HashMap<String, HashMap<String, String>>>,

    /// Custom CT mappings that override standard lookups.
    /// Key: codelist code, Value: map of source_value -> target_value.
    pub custom_ct_maps: HashMap<String, HashMap<String, String>>,

    /// Whether to include diagnostic information in results.
    pub collect_diagnostics: bool,
}

impl TransformContext {
    /// Create a new context with minimal configuration.
    pub fn new(study_id: impl Into<String>) -> Self {
        Self {
            study_id: study_id.into(),
            reference_start_date: None,
            ct_mode: CtResolutionMode::default(),
            ct_lookup: Arc::new(HashMap::new()),
            custom_ct_maps: HashMap::new(),
            collect_diagnostics: false,
        }
    }

    /// Set the reference start date for study day calculations.
    pub fn with_reference_date(mut self, column: impl Into<String>) -> Self {
        self.reference_start_date = Some(column.into());
        self
    }

    /// Set the CT resolution mode.
    pub fn with_ct_mode(mut self, mode: CtResolutionMode) -> Self {
        self.ct_mode = mode;
        self
    }

    /// Set the CT lookup table.
    pub fn with_ct_lookup(mut self, lookup: Arc<HashMap<String, HashMap<String, String>>>) -> Self {
        self.ct_lookup = lookup;
        self
    }

    /// Enable diagnostic collection.
    pub fn with_diagnostics(mut self) -> Self {
        self.collect_diagnostics = true;
        self
    }

    /// Look up a CT value in the loaded terminology.
    pub fn resolve_ct(&self, codelist_code: &str, value: &str) -> CtLookupResult {
        // Check custom mappings first
        if let Some(custom_map) = self.custom_ct_maps.get(codelist_code) {
            if let Some(mapped) = custom_map.get(value) {
                return CtLookupResult::CustomMapped(mapped.clone());
            }
        }

        // Check standard CT
        if let Some(codelist) = self.ct_lookup.get(codelist_code) {
            // Try exact match first
            if let Some(term) = codelist.get(value) {
                return CtLookupResult::Found(term.clone());
            }

            // Try case-insensitive match
            let upper_value = value.to_uppercase();
            for (key, term) in codelist {
                if key.to_uppercase() == upper_value {
                    return CtLookupResult::Found(term.clone());
                }
            }

            // Try compact key match (remove spaces, special chars)
            let compact_value = Self::compact_key(value);
            for (key, term) in codelist {
                if Self::compact_key(key) == compact_value {
                    return CtLookupResult::Found(term.clone());
                }
            }
        }

        // Not found - behavior depends on mode
        match self.ct_mode {
            CtResolutionMode::Strict => CtLookupResult::NotFound,
            CtResolutionMode::Lenient => CtLookupResult::PassThrough(value.to_string()),
        }
    }

    /// Create a compact key for fuzzy matching.
    fn compact_key(s: &str) -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_uppercase()
    }
}

impl Default for TransformContext {
    fn default() -> Self {
        Self::new("STUDY001")
    }
}

/// CT resolution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CtResolutionMode {
    /// Strict mode: unmatched values result in error.
    Strict,
    /// Lenient mode: unmatched values pass through unchanged.
    #[default]
    Lenient,
}

/// Result of a CT lookup operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CtLookupResult {
    /// Found in standard CT.
    Found(String),
    /// Found via custom mapping.
    CustomMapped(String),
    /// Not found (strict mode).
    NotFound,
    /// Passed through unchanged (lenient mode).
    PassThrough(String),
}

impl CtLookupResult {
    /// Get the resolved value, if any.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Found(v) | Self::CustomMapped(v) | Self::PassThrough(v) => Some(v),
            Self::NotFound => None,
        }
    }

    /// Check if this was successfully resolved.
    pub fn is_resolved(&self) -> bool {
        !matches!(self, Self::NotFound)
    }
}

/// Result of executing a transformation pipeline.
#[derive(Debug, Clone, Default)]
pub struct TransformResult {
    /// Number of rows processed.
    pub rows_processed: usize,

    /// Per-rule execution statistics.
    pub rule_stats: Vec<RuleStats>,

    /// Diagnostic messages (if diagnostics enabled).
    pub diagnostics: Vec<Diagnostic>,

    /// Overall success status.
    pub success: bool,
}

impl TransformResult {
    /// Create a new successful result.
    pub fn success(rows: usize) -> Self {
        Self {
            rows_processed: rows,
            success: true,
            ..Default::default()
        }
    }

    /// Create a new failed result.
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            diagnostics: vec![Diagnostic::error(message)],
            ..Default::default()
        }
    }

    /// Add rule statistics.
    pub fn add_rule_stats(&mut self, stats: RuleStats) {
        self.rule_stats.push(stats);
    }

    /// Add a diagnostic.
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Count errors.
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Error)
            .count()
    }

    /// Count warnings.
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Warning)
            .count()
    }
}

/// Statistics for a single rule execution.
#[derive(Debug, Clone)]
pub struct RuleStats {
    /// The target variable name.
    pub target_variable: String,

    /// The transform type applied.
    pub transform_type: TransformType,

    /// Number of values successfully transformed.
    pub success_count: usize,

    /// Number of null/missing values.
    pub null_count: usize,

    /// Number of values that failed transformation.
    pub error_count: usize,

    /// Time taken in milliseconds.
    pub duration_ms: u64,
}

impl RuleStats {
    /// Create stats from a rule.
    pub fn from_rule(rule: &TransformRule) -> Self {
        Self {
            target_variable: rule.target_variable.clone(),
            transform_type: rule.transform_type.clone(),
            success_count: 0,
            null_count: 0,
            error_count: 0,
            duration_ms: 0,
        }
    }

    /// Calculate success rate.
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.error_count;
        if total == 0 {
            1.0
        } else {
            self.success_count as f64 / total as f64
        }
    }
}

/// A diagnostic message from transformation.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level.
    pub level: DiagnosticLevel,

    /// Message text.
    pub message: String,

    /// Associated variable (if any).
    pub variable: Option<String>,

    /// Row index (if applicable).
    pub row: Option<usize>,

    /// Original value (if applicable).
    pub original_value: Option<String>,
}

impl Diagnostic {
    /// Create an error diagnostic.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.into(),
            variable: None,
            row: None,
            original_value: None,
        }
    }

    /// Create a warning diagnostic.
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: message.into(),
            variable: None,
            row: None,
            original_value: None,
        }
    }

    /// Create an info diagnostic.
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Info,
            message: message.into(),
            variable: None,
            row: None,
            original_value: None,
        }
    }

    /// Add variable context.
    pub fn with_variable(mut self, var: impl Into<String>) -> Self {
        self.variable = Some(var.into());
        self
    }

    /// Add row context.
    pub fn with_row(mut self, row: usize) -> Self {
        self.row = Some(row);
        self
    }

    /// Add original value context.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.original_value = Some(value.into());
        self
    }
}

/// Diagnostic severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// Informational message.
    Info,
    /// Warning - transformation succeeded but with caveats.
    Warning,
    /// Error - transformation failed.
    Error,
}

/// Builder for creating pipelines from domain metadata.
#[derive(Debug)]
pub struct PipelineBuilder {
    domain_code: String,
    rules: Vec<TransformRule>,
}

impl PipelineBuilder {
    /// Create a new builder for a domain.
    pub fn new(domain_code: impl Into<String>) -> Self {
        Self {
            domain_code: domain_code.into(),
            rules: Vec::new(),
        }
    }

    /// Add a rule.
    pub fn add_rule(mut self, rule: TransformRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Build the pipeline.
    pub fn build(self) -> DomainPipeline {
        DomainPipeline {
            domain_code: self.domain_code,
            rules: self.rules,
            custom_ct_maps: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_ct_lookup() {
        let mut ct = HashMap::new();
        let mut sex_terms = HashMap::new();
        sex_terms.insert("M".to_string(), "M".to_string());
        sex_terms.insert("F".to_string(), "F".to_string());
        sex_terms.insert("MALE".to_string(), "M".to_string());
        sex_terms.insert("FEMALE".to_string(), "F".to_string());
        ct.insert("C66731".to_string(), sex_terms);

        let ctx = TransformContext::new("TEST001").with_ct_lookup(Arc::new(ct));

        // Exact match
        assert_eq!(
            ctx.resolve_ct("C66731", "M"),
            CtLookupResult::Found("M".to_string())
        );

        // Case insensitive
        assert_eq!(
            ctx.resolve_ct("C66731", "male"),
            CtLookupResult::Found("M".to_string())
        );

        // Not found in lenient mode
        assert_eq!(
            ctx.resolve_ct("C66731", "UNKNOWN"),
            CtLookupResult::PassThrough("UNKNOWN".to_string())
        );

        // Not found in strict mode
        let ctx_strict = TransformContext::new("TEST001").with_ct_mode(CtResolutionMode::Strict);
        assert_eq!(
            ctx_strict.resolve_ct("C66731", "UNKNOWN"),
            CtLookupResult::NotFound
        );
    }

    #[test]
    fn test_transform_result() {
        let mut result = TransformResult::success(100);
        result.add_diagnostic(Diagnostic::warning("Test warning").with_variable("AESEV"));
        result.add_diagnostic(Diagnostic::error("Test error"));

        assert!(result.success);
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 1);
    }
}
