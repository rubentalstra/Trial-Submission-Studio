use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, anyhow};

use sdtm_model::Domain;

use crate::DomainFrame;

pub fn domain_map_by_code(domains: &[Domain]) -> BTreeMap<String, &Domain> {
    let mut map = BTreeMap::new();
    for domain in domains {
        map.insert(domain.code.to_uppercase(), domain);
    }
    map
}

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
        if let Some(parent) = code.strip_prefix("SUPP") {
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

pub fn is_supporting_domain(code: &str) -> bool {
    let upper = code.to_uppercase();
    upper.starts_with("SUPP") || matches!(upper.as_str(), "RELREC" | "RELSPEC" | "RELSUB")
}
