use sdtm_core::domain_utils::order_variables_by_role;
use sdtm_model::{Variable, VariableType};

fn make_variable(name: &str, role: &str, order: u32) -> Variable {
    Variable {
        name: name.to_string(),
        label: None,
        data_type: VariableType::Char,
        length: Some(200),
        role: Some(role.to_string()),
        core: Some("Req".to_string()),
        codelist_code: None,
        order: Some(order),
    }
}

#[test]
fn order_variables_by_role_sorts_by_role_then_order() {
    let unordered = vec![
        make_variable("AESTDTC", "Timing", 53),
        make_variable("AETERM", "Topic", 9),
        make_variable("AESEV", "Record Qualifier", 27),
        make_variable("STUDYID", "Identifier", 1),
    ];

    let ordered = order_variables_by_role(&unordered);

    let names: Vec<String> = ordered.into_iter().map(|var| var.name).collect();
    assert_eq!(names, vec!["STUDYID", "AETERM", "AESEV", "AESTDTC"]);
}
