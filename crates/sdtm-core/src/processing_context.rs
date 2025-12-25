use std::collections::BTreeMap;

use sdtm_model::{ControlledTerminology, CtRegistry, Domain};

#[derive(Debug, Clone, Copy)]
pub struct ProcessingOptions {
    pub prefix_usubjid: bool,
    pub assign_sequence: bool,
    pub warn_on_rewrite: bool,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            prefix_usubjid: true,
            assign_sequence: true,
            warn_on_rewrite: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessingContext<'a> {
    pub study_id: &'a str,
    pub reference_starts: Option<&'a BTreeMap<String, String>>,
    pub ct_registry: Option<&'a CtRegistry>,
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

    pub fn resolve_ct(&self, domain: &Domain, variable: &str) -> Option<&'a ControlledTerminology> {
        let registry = self.ct_registry?;
        let code = domain
            .variables
            .iter()
            .find(|var| var.name.eq_ignore_ascii_case(variable))
            .and_then(|var| var.codelist_code.as_ref());
        if let Some(code) = code {
            for entry in split_codelist_codes(code) {
                let code_key = entry.to_uppercase();
                if let Some(ct) = registry.by_code.get(&code_key) {
                    return Some(ct);
                }
            }
        }
        let name_key = variable.to_uppercase();
        registry.by_name.get(&name_key)
    }
}

fn split_codelist_codes(raw: &str) -> Vec<String> {
    let text = raw.trim();
    if text.is_empty() {
        return Vec::new();
    }
    for sep in [';', ',', ' '] {
        if text.contains(sep) {
            return text
                .split(sep)
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect();
        }
    }
    vec![text.to_string()]
}
