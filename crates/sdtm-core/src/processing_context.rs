use std::collections::BTreeMap;

use sdtm_model::{ControlledTerminology, CtRegistry, Domain};

use crate::provenance::ProvenanceTracker;

/// Options controlling SDTM processing behavior.
///
/// These options determine how the transpiler handles various transformations
/// during domain processing. Some transformations are SDTMIG-approved derivations,
/// while others are convenience features that should be disabled in strict mode.
#[derive(Debug, Clone, Copy)]
pub struct ProcessingOptions {
    /// Add STUDYID prefix to USUBJID values.
    ///
    /// SDTMIG 4.1.2: "USUBJID is a unique subject identifier that is a
    /// concatenation of STUDYID and a subject identifier unique within that study."
    ///
    /// This is an SDTMIG-approved derivation when SUBJID is provided but USUBJID
    /// lacks the study prefix.
    pub prefix_usubjid: bool,

    /// Automatically assign sequence numbers (--SEQ).
    ///
    /// SDTMIG 4.1.5: "The --SEQ variable [...] is a unique number for each record
    /// within a domain for a subject."
    ///
    /// This is an SDTMIG-approved derivation when sequence numbers are missing.
    pub assign_sequence: bool,

    /// Log warnings when values are rewritten/normalized.
    pub warn_on_rewrite: bool,

    /// Allow heuristic field inference from source columns.
    ///
    /// When enabled, the preprocessor attempts to infer test names, codes, and
    /// other fields from source column headers and labels. This is a convenience
    /// feature that may not have explicit SDTMIG backing.
    ///
    /// Default: true (for backward compatibility)
    /// Strict mode: should be false
    pub allow_heuristic_inference: bool,

    /// Allow CT normalization with fuzzy/lenient matching.
    ///
    /// When enabled, CT values that don't exactly match submission values can
    /// still be normalized using fuzzy matching algorithms.
    ///
    /// Default: true (for backward compatibility)
    /// Strict mode: should be false
    pub allow_lenient_ct_matching: bool,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            prefix_usubjid: true,
            assign_sequence: true,
            warn_on_rewrite: true,
            // Default to lenient mode for backward compatibility
            // Strict mode should set these to false
            allow_heuristic_inference: true,
            allow_lenient_ct_matching: true,
        }
    }
}

impl ProcessingOptions {
    /// Create options for strict SDTMIG-conformant processing.
    ///
    /// This disables all heuristic inference and lenient matching,
    /// only allowing explicitly documented SDTMIG derivations.
    ///
    /// Note: Value normalization (e.g., SEX "FEMALE"→"F", RACE mappings)
    /// is always enabled as it's required for SDTM CT compliance.
    pub fn strict() -> Self {
        Self {
            prefix_usubjid: true,  // SDTMIG-approved: 4.1.2
            assign_sequence: true, // SDTMIG-approved: 4.1.5
            warn_on_rewrite: true,
            allow_heuristic_inference: false,
            allow_lenient_ct_matching: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProcessingContext<'a> {
    pub study_id: &'a str,
    pub reference_starts: Option<&'a BTreeMap<String, String>>,
    pub ct_registry: Option<&'a CtRegistry>,
    pub options: ProcessingOptions,
    /// Optional provenance tracker for recording derivation metadata.
    /// Clone is cheap (Arc-based).
    pub provenance: Option<ProvenanceTracker>,
}

impl<'a> ProcessingContext<'a> {
    pub fn new(study_id: &'a str) -> Self {
        Self {
            study_id,
            reference_starts: None,
            ct_registry: None,
            options: ProcessingOptions::default(),
            provenance: None,
        }
    }

    pub fn with_reference_starts(mut self, reference_starts: &'a BTreeMap<String, String>) -> Self {
        self.reference_starts = Some(reference_starts);
        self
    }

    pub fn with_ct_registry(mut self, ct_registry: &'a CtRegistry) -> Self {
        self.ct_registry = Some(ct_registry);
        self
    }

    pub fn with_options(mut self, options: ProcessingOptions) -> Self {
        self.options = options;
        self
    }

    /// Enable provenance tracking with a new tracker.
    pub fn with_provenance(mut self) -> Self {
        self.provenance = Some(ProvenanceTracker::new());
        self
    }

    /// Enable provenance tracking with an existing tracker.
    pub fn with_provenance_tracker(mut self, tracker: ProvenanceTracker) -> Self {
        self.provenance = Some(tracker);
        self
    }

    /// Record a provenance entry if tracking is enabled.
    pub fn record_provenance<F>(&self, f: F)
    where
        F: FnOnce(&ProvenanceTracker),
    {
        if let Some(ref tracker) = self.provenance {
            f(tracker);
        }
    }

    pub fn resolve_ct(&self, domain: &Domain, variable: &str) -> Option<&'a ControlledTerminology> {
        let registry = self.ct_registry?;
        let variable = domain
            .variables
            .iter()
            .find(|var| var.name.eq_ignore_ascii_case(variable))?;
        registry
            .resolve_for_variable(variable, None)
            .map(|resolved| resolved.ct)
    }
}
