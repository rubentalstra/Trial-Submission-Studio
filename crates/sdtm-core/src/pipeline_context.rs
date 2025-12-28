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

/// Mode for applying STUDYID prefixes to USUBJID values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsubjidPrefixMode {
    /// Do not add STUDYID prefixes.
    Skip,
    /// Add STUDYID prefixes when missing.
    Prefix,
}

/// Mode for assigning --SEQ values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceAssignmentMode {
    /// Do not assign sequence values.
    Skip,
    /// Assign sequence values when missing or invalid.
    Assign,
}

/// Mode for controlled terminology matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CtMatchingMode {
    /// Require exact or synonym matches only.
    Strict,
    /// Allow compact-key matching for normalization.
    Lenient,
}

/// Options controlling SDTM processing behavior.
#[derive(Debug, Clone, Copy)]
pub struct ProcessingOptions {
    /// Add STUDYID prefix to USUBJID values.
    ///
    /// SDTMIG 4.1.2: "USUBJID is a unique subject identifier that is a
    /// concatenation of STUDYID and a subject identifier unique within that study."
    pub usubjid_prefix: UsubjidPrefixMode,

    /// Automatically assign sequence numbers (--SEQ).
    ///
    /// SDTMIG 4.1.5: "The --SEQ variable [...] is a unique number for each record
    /// within a domain for a subject."
    pub sequence_assignment: SequenceAssignmentMode,

    /// Log warnings when values are rewritten/normalized.
    pub warn_on_rewrite: bool,

    /// Controlled terminology matching mode.
    pub ct_matching: CtMatchingMode,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            usubjid_prefix: UsubjidPrefixMode::Prefix,
            sequence_assignment: SequenceAssignmentMode::Assign,
            warn_on_rewrite: true,
            ct_matching: CtMatchingMode::Lenient,
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
            usubjid_prefix: UsubjidPrefixMode::Prefix,
            sequence_assignment: SequenceAssignmentMode::Assign,
            warn_on_rewrite: true,
            ct_matching: CtMatchingMode::Strict,
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
