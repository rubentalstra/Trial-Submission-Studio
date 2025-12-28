//! Study Pipeline Context for caching standards and metadata.
//!
//! This module provides a centralized context struct that caches all standards,
//! CT registry, and study metadata for use across all pipeline stages.
//!
//! # SDTMIG v3.4 Reference
//!
//! Per Chapter 3, submission metadata and standards must be consistent throughout
//! the processing pipeline. This context ensures:
//! - Domain metadata is loaded once and reused
//! - CT resolution uses a consistent registry version
//! - Study-level metadata is propagated to all stages

use std::collections::BTreeMap;

use sdtm_model::{Domain, TerminologyRegistry};

use crate::processing_context::{ProcessingContext, ProcessingOptions};

/// Centralized context for the study processing pipeline.
///
/// This struct caches all standards, CT registry, and study metadata
/// once and provides them to all pipeline stages. This avoids repeated loading
/// and ensures consistency across the pipeline.
///
/// # Example
///
/// ```ignore
/// let pipeline = StudyPipelineContext::new("STUDY001")
///     .with_standards(standards)
///     .with_ct_registry(ct_registry);
///
/// // Create processing contexts for individual operations
/// let ctx = pipeline.processing_context();
/// ```
#[derive(Debug)]
pub struct StudyPipelineContext {
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

impl StudyPipelineContext {
    /// Creates a new pipeline context with the given study ID.
    ///
    /// The context starts empty; use builder methods to add standards and metadata.
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

    /// Sets the reference start dates map.
    pub fn with_reference_starts(mut self, reference_starts: BTreeMap<String, String>) -> Self {
        self.reference_starts = reference_starts;
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

    /// Creates a `ProcessingContext` for use in domain processing.
    ///
    /// The returned context references data owned by this pipeline context,
    /// ensuring consistency across all processing operations.
    pub fn processing_context(&self) -> ProcessingContext<'_> {
        let mut ctx = ProcessingContext::new(&self.study_id)
            .with_ct_registry(&self.ct_registry)
            .with_options(self.options);

        if !self.reference_starts.is_empty() {
            ctx = ctx.with_reference_starts(&self.reference_starts);
        }

        ctx
    }
}

impl Default for StudyPipelineContext {
    fn default() -> Self {
        Self::new("")
    }
}
