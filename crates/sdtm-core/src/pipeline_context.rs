//! Pipeline context for SDTM processing and validation.
//!
//! This module centralizes study-level metadata, CT registry, and processing
//! options to ensure consistent behavior across pipeline stages.
//!
//! # Architecture
//!
//! The [`PipelineContext`] bundles several concerns for convenience:
//!
//! - **Study metadata**: `study_id` identifying the clinical study
//! - **Standards registry**: SDTMIG domain definitions (`standards`, `standards_map`)
//! - **CT registry**: Controlled Terminology lookup (`ct_registry`)
//! - **Reference dates**: Subject baseline dates for --DY derivation (`reference_starts`)
//! - **Options**: Processing behavior configuration ([`ProcessingOptions`])
//!
//! The [`ProcessingOptions`] type is extracted to allow configuration reuse.
//!
//! # SDTMIG v3.4 Reference
//!
//! Per Chapter 3, submission metadata and standards must be consistent
//! throughout the processing pipeline.

use std::collections::BTreeMap;

use sdtm_model::Domain;
use sdtm_model::ct::{Codelist, TerminologyRegistry};
pub use sdtm_model::options::{
    CtMatchingMode, NormalizationOptions, ProcessingOptions, SequenceAssignmentMode,
    UsubjidPrefixMode,
};

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
