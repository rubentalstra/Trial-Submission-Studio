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
    assert!(!registry.catalogs.is_empty());
    assert!(registry.catalogs.contains_key("SDTM CT"));
    assert!(registry.catalogs.contains_key("SEND CT"));
}

#[test]
fn loads_send_country_codelist() {
    let registry = load_default_ct_registry().expect("load ct");
    let send_catalog = registry.catalogs.get("SEND CT").expect("send ct");
    let ct = send_catalog
        .by_submission
        .get("COUNTRY")
        .expect("country codelist");
    assert_eq!(ct.codelist_code, "C66786");
    assert!(ct.submission_values.iter().any(|value| value == "USA"));
}

#[test]
fn loads_p21_rules() {
    let rules = load_default_p21_rules().expect("load rules");
    assert!(!rules.is_empty());
}
