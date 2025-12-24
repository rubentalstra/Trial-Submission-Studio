use std::collections::BTreeMap;
use sdtm_map::MappingEngine;
use sdtm_model::{ColumnHint, Domain};
use sdtm_standards::load_default_sdtm_ig_domains;

fn sample_domain() -> Domain {
    let domains = load_default_sdtm_ig_domains().expect("standards");
    domains
        .into_iter()
        .find(|domain| domain.code == "DM")
        .expect("DM domain")
}

#[test]
fn suggests_mappings_and_unmapped() {
    let domain = sample_domain();
    let mut hints = BTreeMap::new();
    hints.insert(
        "AGE".to_string(),
        ColumnHint {
            is_numeric: true,
            unique_ratio: 1.0,
            null_ratio: 0.0,
        },
    );
    let engine = MappingEngine::new(domain, 0.5, hints);
    let columns = vec!["Study Id".to_string(), "AGE".to_string(), "OTHER".to_string()];
    let result = engine.suggest(&columns);

    assert!(result.mappings.iter().any(|m| m.target_variable == "STUDYID"));
    assert!(result.mappings.iter().any(|m| m.target_variable == "AGE"));
    assert!(result.unmapped_columns.contains(&"OTHER".to_string()));
}
