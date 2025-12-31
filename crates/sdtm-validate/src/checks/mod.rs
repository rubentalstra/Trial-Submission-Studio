//! Validation check modules.
//!
//! Each module performs a specific type of validation check.

mod ct;
mod datatype;
pub mod dates;
mod expected;
mod identifier;
mod length;
mod required;
mod sequence;

use polars::prelude::DataFrame;
use sdtm_model::Domain;
use sdtm_model::ct::TerminologyRegistry;

use crate::report::ValidationReport;
use crate::util::CaseInsensitiveSet;

/// Run all validation checks on a domain.
pub fn run_all(
    domain: &Domain,
    df: &DataFrame,
    ct_registry: Option<&TerminologyRegistry>,
) -> ValidationReport {
    let column_lookup = build_column_lookup(df);
    let mut report = ValidationReport::new(&domain.name);

    // 1. Required variable checks (presence + population)
    for issue in required::check(domain, df, &column_lookup) {
        report.add(issue);
    }

    // 2. Expected variable checks (presence only, warnings)
    for issue in expected::check(domain, df, &column_lookup) {
        report.add(issue);
    }

    // 3. Data type validation (Num columns must be numeric)
    for issue in datatype::check(domain, df, &column_lookup) {
        report.add(issue);
    }

    // 4. ISO 8601 date format validation
    for issue in dates::check(domain, df, &column_lookup) {
        report.add(issue);
    }

    // 5. Sequence uniqueness (--SEQ must be unique per USUBJID)
    for issue in sequence::check(domain, df, &column_lookup) {
        report.add(issue);
    }

    // 6. Text length validation
    for issue in length::check(domain, df, &column_lookup) {
        report.add(issue);
    }

    // 7. Identifier null checks
    for issue in identifier::check(domain, df, &column_lookup) {
        report.add(issue);
    }

    // 8. Controlled terminology validation
    if let Some(registry) = ct_registry {
        for issue in ct::check(domain, df, &column_lookup, registry) {
            report.add(issue);
        }
    }

    report
}

/// Build case-insensitive column name lookup.
fn build_column_lookup(df: &DataFrame) -> CaseInsensitiveSet {
    CaseInsensitiveSet::from_iter(df.get_column_names_owned())
}
