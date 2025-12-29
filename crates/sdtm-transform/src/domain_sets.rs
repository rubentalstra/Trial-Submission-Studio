//! Domain collection utilities for SDTM processing.
//!
//! Provides helper functions for working with collections of domains,
//! including building lookup maps and generating report-ready domain sets
//! with dynamic SUPPQUAL metadata.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};

use sdtm_model::Domain;

use crate::frame::DomainFrame;

/// Build a lookup map from domain code to domain definition.
///
/// Keys are uppercase domain codes for case-insensitive lookup.
pub fn domain_map_by_code(domains: &[Domain]) -> BTreeMap<String, &Domain> {
    let mut map = BTreeMap::new();
    for domain in domains {
        map.insert(domain.code.to_uppercase(), domain);
    }
    map
}

/// Build a complete domain list for reporting, including dynamic SUPP domains.
///
/// Takes the standard domain definitions and processed frames, then generates
/// domain definitions for any SUPP-- datasets found in the frames that don't
/// have explicit definitions.
///
/// # Errors
///
/// Returns an error if SUPPQUAL metadata is missing from standards.
pub fn build_report_domains(standards: &[Domain], frames: &[DomainFrame]) -> Result<Vec<Domain>> {
    let mut domains = standards.to_vec();
    let mut known: BTreeSet<String> = standards
        .iter()
        .map(|domain| domain.code.to_uppercase())
        .collect();
    let suppqual = standards
        .iter()
        .find(|domain| domain.code.eq_ignore_ascii_case("SUPPQUAL"))
        .ok_or_else(|| anyhow!("missing SUPPQUAL metadata"))?;

    for frame in frames {
        let code = frame.domain_code.to_uppercase();
        if known.contains(&code) {
            continue;
        }
        if let Some(parent) = code
            .strip_prefix("SUPP")
            .or_else(|| code.strip_prefix("SQ"))
        {
            if parent.is_empty() {
                continue;
            }
            let label = format!("Supplemental Qualifiers for {parent}");
            let mut domain = suppqual.clone();
            domain.code = code.clone();
            domain.dataset_name = Some(code.clone());
            domain.label = Some(label.clone());
            domain.description = Some(label);
            domains.push(domain);
            known.insert(code);
        }
    }
    domains.sort_by(|a, b| a.code.cmp(&b.code));
    Ok(domains)
}

/// Check if a domain code represents a supporting/relationship domain.
///
/// Supporting domains include SUPP-- (supplemental qualifiers), SQ--,
/// and relationship datasets (RELREC, RELSPEC, RELSUB).
pub fn is_supporting_domain(code: &str) -> bool {
    let upper = code.to_uppercase();
    upper.starts_with("SUPP")
        || upper.starts_with("SQ")
        || matches!(upper.as_str(), "RELREC" | "RELSPEC" | "RELSUB")
}
