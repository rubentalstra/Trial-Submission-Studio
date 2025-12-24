use std::collections::BTreeMap;

use sdtm_model::Domain;

use crate::utils::normalize_text;

pub fn build_variable_patterns(domain: &Domain) -> BTreeMap<String, Vec<String>> {
    let mut patterns = BTreeMap::new();
    for variable in &domain.variables {
        let name = variable.name.trim().to_string();
        if name.is_empty() {
            continue;
        }
        let mut values = Vec::new();
        values.push(normalize_text(&name));
        let name_upper = name.to_uppercase();
        if name_upper.starts_with("--") {
            values.push(normalize_text(&name_upper.replace("--", "")));
        }
        patterns.insert(name, values);
    }
    patterns
}
