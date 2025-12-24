use std::path::Path;

use sdtm_standards::{
    load_ct_registry, load_p21_rules, load_sdtm_ig_domains, load_sdtm_domains,
};

#[test]
fn loads_sdtmig_domains() {
    let path = Path::new("../../standards/sdtmig/v3_4");
    let domains = load_sdtm_ig_domains(path).expect("load sdtmig domains");
    assert!(!domains.is_empty());
    let dm = domains.iter().find(|d| d.code == "DM");
    assert!(dm.is_some());
}

#[test]
fn loads_sdtm_domains() {
    let path = Path::new("../../standards/sdtm/v2_0");
    let domains = load_sdtm_domains(path).expect("load sdtm domains");
    assert!(!domains.is_empty());
    let dm = domains.iter().find(|d| d.code == "DM");
    assert!(dm.is_some());
}

#[test]
fn loads_ct_registry() {
    let path = Path::new("../../standards/ct/2024-03-29");
    let registry = load_ct_registry(path).expect("load ct");
    assert!(!registry.by_code.is_empty());
    assert!(!registry.by_name.is_empty());
}

#[test]
fn loads_p21_rules() {
    let path = Path::new("../../standards/p21/Rules.csv");
    let rules = load_p21_rules(path).expect("load rules");
    assert!(!rules.is_empty());
}
