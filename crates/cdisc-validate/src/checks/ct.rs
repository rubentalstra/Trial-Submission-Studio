//! Controlled terminology validation (SDTMIG Chapter 10).
//!
//! Checks that values conform to controlled terminology.

use std::collections::BTreeSet;

use cdisc_common::any_to_string;
use cdisc_model::ct::{Codelist, ResolvedCodelist, TerminologyRegistry};
use cdisc_model::{Domain, Variable};
use polars::prelude::{AnyValue, DataFrame};

use crate::issue::Issue;
use crate::util::CaseInsensitiveSet;

const MAX_INVALID_VALUES: usize = 5;

/// Check that values conform to controlled terminology.
pub fn check(
    domain: &Domain,
    df: &DataFrame,
    columns: &CaseInsensitiveSet,
    ct_registry: &TerminologyRegistry,
) -> Vec<Issue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        let Some(column) = columns.get(&variable.name) else {
            continue;
        };

        if let Some(resolved) = resolve_ct(ct_registry, variable) {
            if let Some(issue) = check_ct_values(variable, df, column, &resolved) {
                issues.push(issue);
            }
        }
    }

    issues
}

/// Check CT values for a single variable.
fn check_ct_values(
    variable: &Variable,
    df: &DataFrame,
    column: &str,
    resolved: &ResolvedCodelist,
) -> Option<Issue> {
    let ct = resolved.codelist;
    let invalid = collect_invalid_ct_values(df, column, ct);

    if invalid.is_empty() {
        return None;
    }

    let invalid_values: Vec<String> = invalid.into_iter().take(MAX_INVALID_VALUES).collect();
    let invalid_count = invalid_values.len() as u64;

    Some(Issue::CtViolation {
        variable: variable.name.clone(),
        codelist_code: ct.code.clone(),
        codelist_name: ct.name.clone(),
        extensible: ct.extensible,
        invalid_count,
        invalid_values,
        allowed_count: ct.terms.len(),
    })
}

/// Collect values not in the codelist.
fn collect_invalid_ct_values(df: &DataFrame, column: &str, ct: &Codelist) -> BTreeSet<String> {
    let mut invalid = BTreeSet::new();

    let Ok(series) = df.column(column) else {
        return invalid;
    };

    // Build lookup set of valid submission values (uppercase for matching)
    let submission_values: BTreeSet<String> = ct
        .submission_values()
        .iter()
        .map(|value| value.to_uppercase())
        .collect();

    for idx in 0..df.height() {
        let raw = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            continue;
        }

        let normalized = ct.normalize(trimmed);
        if normalized.is_empty() {
            continue;
        }

        let key = normalized.to_uppercase();
        if !submission_values.contains(&key) {
            invalid.insert(trimmed.to_string());
        }
    }

    invalid
}

/// Resolve codelist for a variable.
fn resolve_ct<'a>(
    registry: &'a TerminologyRegistry,
    variable: &Variable,
) -> Option<ResolvedCodelist<'a>> {
    let codelist_code = variable.codelist_code.as_ref()?;
    let code = codelist_code.split(';').next()?.trim();

    if code.is_empty() {
        return None;
    }

    registry.resolve(code, None)
}
