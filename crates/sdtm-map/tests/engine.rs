use std::collections::BTreeMap;

use sdtm_map::MappingEngine;
use sdtm_model::{ColumnHint, Domain, Variable, VariableType};

fn sample_domain() -> Domain {
    Domain {
        code: "DM".to_string(),
        description: None,
        class_name: None,
        label: None,
        structure: None,
        dataset_name: Some("DM".to_string()),
        variables: vec![
            Variable {
                name: "STUDYID".to_string(),
                label: None,
                data_type: VariableType::Char,
                length: None,
                role: None,
                core: None,
                codelist_code: None,
            },
            Variable {
                name: "USUBJID".to_string(),
                label: None,
                data_type: VariableType::Char,
                length: None,
                role: None,
                core: None,
                codelist_code: None,
            },
            Variable {
                name: "AGE".to_string(),
                label: None,
                data_type: VariableType::Num,
                length: None,
                role: None,
                core: None,
                codelist_code: None,
            },
        ],
    }
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
