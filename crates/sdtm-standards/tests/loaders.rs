use sdtm_standards::{
    load_default_ct_registry, load_default_p21_rules, load_default_sdtm_domains,
    load_default_sdtm_ig_domains,
};

#[test]
fn loads_sdtmig_domains() {
    let domains = load_default_sdtm_ig_domains().expect("load sdtmig domains");
    assert!(!domains.is_empty());
    let dm = domains.iter().find(|d| d.code == "DM");
    assert!(dm.is_some());
}

#[test]
fn loads_sdtm_domains() {
    let domains = load_default_sdtm_domains().expect("load sdtm domains");
    assert!(!domains.is_empty());
    let dm = domains.iter().find(|d| d.code == "DM");
    assert!(dm.is_some());
}

#[test]
fn loads_ct_registry() {
    let registry = load_default_ct_registry().expect("load ct");
    assert!(!registry.by_code.is_empty());
    assert!(!registry.by_name.is_empty());
}

#[test]
fn loads_p21_rules() {
    let rules = load_default_p21_rules().expect("load rules");
    assert!(!rules.is_empty());
}
