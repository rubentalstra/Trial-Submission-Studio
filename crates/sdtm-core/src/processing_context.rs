use std::collections::BTreeMap;

use sdtm_model::{ControlledTerminology, CtRegistry, Domain};

#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessingContext<'a> {
    pub study_id: &'a str,
    pub reference_starts: Option<&'a BTreeMap<String, String>>,
    pub ct_registry: Option<&'a CtRegistry>,
}

impl<'a> ProcessingContext<'a> {
    pub fn new(study_id: &'a str) -> Self {
        Self {
            study_id,
            reference_starts: None,
            ct_registry: None,
        }
    }

    pub fn with_reference_starts(
        mut self,
        reference_starts: &'a BTreeMap<String, String>,
    ) -> Self {
        self.reference_starts = Some(reference_starts);
        self
    }

    pub fn with_ct_registry(mut self, ct_registry: &'a CtRegistry) -> Self {
        self.ct_registry = Some(ct_registry);
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
            let code_key = code.to_uppercase();
            if let Some(ct) = registry.by_code.get(&code_key) {
                return Some(ct);
            }
        }
        let name_key = variable.to_uppercase();
        registry.by_name.get(&name_key)
    }
}
