//! Pipeline context for SDTM processing and validation.
//!
//! This module centralizes study-level metadata, CT registry, and processing
//! options to ensure consistent behavior across pipeline stages.
//!
//! # SDTMIG v3.4 Reference
//!
//! Per Chapter 3, submission metadata and standards must be consistent
//! throughout the processing pipeline.

use std::collections::BTreeMap;

use sdtm_model::Domain;
use sdtm_model::ct::{Codelist, TerminologyRegistry};

/// Options controlling SDTM processing behavior.
///
/// These options determine how the transpiler handles transformations during
/// domain processing. Some transformations are SDTMIG-approved derivations,
/// while others are convenience features that should be disabled in strict mode.
#[derive(Debug, Clone, Copy)]
pub struct ProcessingOptions {
    /// Add STUDYID prefix to USUBJID values.
    ///
    /// SDTMIG 4.1.2: "USUBJID is a unique subject identifier that is a
    /// concatenation of STUDYID and a subject identifier unique within that study."
    pub prefix_usubjid: bool,

    /// Automatically assign sequence numbers (--SEQ).
    ///
    /// SDTMIG 4.1.5: "The --SEQ variable [...] is a unique number for each record
    /// within a domain for a subject."
    pub assign_sequence: bool,

    /// Log warnings when values are rewritten/normalized.
    pub warn_on_rewrite: bool,

    /// Allow CT normalization with lenient matching.
    ///
    /// When enabled, CT values that don't exactly match submission values can
    /// still be normalized using compact-key matching.
    ///
    /// Default: true (for backward compatibility)
    /// Strict mode: false
    pub allow_lenient_ct_matching: bool,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            prefix_usubjid: true,
            assign_sequence: true,
            warn_on_rewrite: true,
            allow_lenient_ct_matching: true,
        }
    }
}

impl ProcessingOptions {
    /// Create options for strict SDTMIG-conformant processing.
    ///
    /// This disables lenient CT matching while preserving documented SDTMIG
    /// derivations (USUBJID prefix and sequence assignment).
    pub fn strict() -> Self {
        Self {
            prefix_usubjid: true,
            assign_sequence: true,
            warn_on_rewrite: true,
            allow_lenient_ct_matching: false,
        }
    }
}

/// Centralized context for the study processing pipeline.
#[derive(Debug)]
pub struct PipelineContext {
    /// Study identifier (e.g., "CDISC01").
    pub study_id: String,
    /// SDTMIG domain definitions loaded from standards.
    pub standards: Vec<Domain>,
    /// Map of domain code (uppercase) to domain definition for quick lookup.
    pub standards_map: BTreeMap<String, Domain>,
    /// Controlled Terminology registry.
    pub ct_registry: TerminologyRegistry,
    /// Reference start dates (RFSTDTC) by USUBJID for SDY derivation.
    pub reference_starts: BTreeMap<String, String>,
    /// Processing options (prefixing, sequencing, etc.).
    pub options: ProcessingOptions,
}

impl PipelineContext {
    /// Creates a new pipeline context with the given study ID.
    pub fn new(study_id: impl Into<String>) -> Self {
        Self {
            study_id: study_id.into(),
            standards: Vec::new(),
            standards_map: BTreeMap::new(),
            ct_registry: TerminologyRegistry::default(),
            reference_starts: BTreeMap::new(),
            options: ProcessingOptions::default(),
        }
    }

    /// Sets the SDTMIG domain standards.
    pub fn with_standards(mut self, standards: Vec<Domain>) -> Self {
        self.standards_map.clear();
        for domain in &standards {
            self.standards_map
                .insert(domain.code.to_uppercase(), domain.clone());
        }
        self.standards = standards;
        self
    }

    /// Sets the Controlled Terminology registry.
    pub fn with_ct_registry(mut self, ct_registry: TerminologyRegistry) -> Self {
        self.ct_registry = ct_registry;
        self
    }

    /// Sets the processing options.
    pub fn with_options(mut self, options: ProcessingOptions) -> Self {
        self.options = options;
        self
    }

    /// Adds or updates reference start dates from a map.
    pub fn add_reference_starts(&mut self, starts: BTreeMap<String, String>) {
        for (usubjid, rfstdtc) in starts {
            self.reference_starts.entry(usubjid).or_insert(rfstdtc);
        }
    }

    /// Gets the domain definition by code (case-insensitive).
    pub fn get_domain(&self, code: &str) -> Option<&Domain> {
        self.standards_map.get(&code.to_uppercase())
    }

    /// Gets all domain codes.
    pub fn domain_codes(&self) -> Vec<String> {
        self.standards.iter().map(|d| d.code.clone()).collect()
    }

    /// Resolves a codelist for a domain variable using the CT registry.
    pub fn resolve_ct(&self, domain: &Domain, variable: &str) -> Option<&Codelist> {
        let variable = domain
            .variables
            .iter()
            .find(|var| var.name.eq_ignore_ascii_case(variable))?;
        let codelist_code = variable.codelist_code.as_ref()?;
        let code = codelist_code.split(';').next()?.trim();
        if code.is_empty() {
            return None;
        }
        self.ct_registry
            .resolve(code, None)
            .map(|resolved| resolved.codelist)
    }
}

impl Default for PipelineContext {
    fn default() -> Self {
        Self::new("")
    }
}
