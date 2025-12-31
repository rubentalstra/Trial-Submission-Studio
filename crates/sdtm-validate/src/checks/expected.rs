//! Expected variable checks (SDTMIG 4.1).
//!
//! Checks that Expected (Exp) variables are present (warnings only).

use sdtm_model::{CoreDesignation, Domain};

use crate::issue::Issue;
use crate::util::CaseInsensitiveSet;

/// Check expected variables are present.
pub fn check(domain: &Domain, columns: &CaseInsensitiveSet) -> Vec<Issue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        if variable.core != Some(CoreDesignation::Expected) {
            continue;
        }

        if columns.get(&variable.name).is_none() {
            issues.push(Issue::ExpectedMissing {
                variable: variable.name.clone(),
            });
        }
    }

    issues
}
