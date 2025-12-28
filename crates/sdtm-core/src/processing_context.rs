use std::collections::BTreeMap;

use sdtm_model::Domain;
use sdtm_model::ct::{Codelist, TerminologyRegistry};

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
            allow_lenient_ct_matching: true,
        }
    }
}

impl ProcessingOptions {
    /// Create options for strict SDTMIG-conformant processing.
    ///
    /// This disables lenient CT matching while preserving documented
    /// SDTMIG derivations.
    ///
    /// Note: Value normalization (e.g., SEX "FEMALE"→"F", RACE mappings)
    /// is always enabled as it's required for SDTM CT compliance.
    pub fn strict() -> Self {
        Self {
            prefix_usubjid: true,  // SDTMIG-approved: 4.1.2
            assign_sequence: true, // SDTMIG-approved: 4.1.5
            warn_on_rewrite: true,
            allow_lenient_ct_matching: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProcessingContext<'a> {
    pub study_id: &'a str,
    pub reference_starts: Option<&'a BTreeMap<String, String>>,
    pub ct_registry: Option<&'a TerminologyRegistry>,
    pub options: ProcessingOptions,
}

impl<'a> ProcessingContext<'a> {
    pub fn new(study_id: &'a str) -> Self {
        Self {
            study_id,
            reference_starts: None,
            ct_registry: None,
            options: ProcessingOptions::default(),
        }
    }

    pub(crate) fn with_reference_starts(
        mut self,
        reference_starts: &'a BTreeMap<String, String>,
    ) -> Self {
        self.reference_starts = Some(reference_starts);
        self
    }

    pub fn with_ct_registry(mut self, ct_registry: &'a TerminologyRegistry) -> Self {
        self.ct_registry = Some(ct_registry);
        self
    }

    pub fn with_options(mut self, options: ProcessingOptions) -> Self {
        self.options = options;
        self
    }

    pub(crate) fn resolve_ct(&self, domain: &Domain, variable: &str) -> Option<&'a Codelist> {
        let registry = self.ct_registry?;
        let variable = domain
            .variables
            .iter()
            .find(|var| var.name.eq_ignore_ascii_case(variable))?;

        // Get the codelist code from the variable metadata
        let codelist_code = variable.codelist_code.as_ref()?;

        // Resolve the first codelist code (some variables have multiple separated by ';')
        let code = codelist_code.split(';').next()?.trim();
        if code.is_empty() {
            return None;
        }

        registry
            .resolve(code, None)
            .map(|resolved| resolved.codelist)
    }
}
