use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;

use sdtm_model::{DatasetClass, DatasetMetadata, Domain, Variable, VariableType};

use crate::csv_utils::{default_standards_root, read_csv_rows};

const DEFAULT_SDTMIG_VERSION: &str = "v3_4";

pub fn load_default_sdtm_ig_domains() -> Result<Vec<Domain>> {
    let root = default_standards_root();
    load_sdtm_ig_domains(&root.join("sdtmig").join(DEFAULT_SDTMIG_VERSION))
}

fn parse_variable_type(raw: &str) -> VariableType {
    match raw.trim().to_lowercase().as_str() {
        "num" | "numeric" => VariableType::Num,
        _ => VariableType::Char,
    }
}

pub fn load_sdtm_ig_domains(base_dir: &Path) -> Result<Vec<Domain>> {
    let datasets = read_csv_rows(&base_dir.join("Datasets.csv"))?;
    let variables = read_csv_rows(&base_dir.join("Variables.csv"))?;
    build_domains(&datasets, &variables, "Dataset Name")
}

fn build_domains(
    datasets: &[BTreeMap<String, String>],
    variables: &[BTreeMap<String, String>],
    dataset_key: &str,
) -> Result<Vec<Domain>> {
    let mut meta = BTreeMap::new();
    for row in datasets {
        let name = row.get(dataset_key).cloned().unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let class_name = row.get("Class").filter(|v| !v.is_empty()).cloned();
        // Parse the class name into the strongly-typed DatasetClass enum
        let dataset_class = class_name
            .as_ref()
            .and_then(|c| c.parse::<DatasetClass>().ok());
        meta.insert(
            name.to_uppercase(),
            DatasetMetadata {
                dataset_name: name.to_uppercase(),
                class_name,
                dataset_class,
                label: row.get("Dataset Label").filter(|v| !v.is_empty()).cloned(),
                structure: row.get("Structure").filter(|v| !v.is_empty()).cloned(),
            },
        );
    }

    let mut grouped: BTreeMap<String, Vec<Variable>> = BTreeMap::new();
    for row in variables {
        let dataset = row
            .get(dataset_key)
            .cloned()
            .unwrap_or_default()
            .to_uppercase();
        let var_name = row.get("Variable Name").cloned().unwrap_or_default();
        if dataset.is_empty() || var_name.is_empty() {
            continue;
        }
        let order = row
            .get("Variable Order")
            .and_then(|value| value.trim().parse::<u32>().ok());
        let variable = Variable {
            name: var_name,
            label: row.get("Variable Label").filter(|v| !v.is_empty()).cloned(),
            data_type: parse_variable_type(row.get("Type").map(String::as_str).unwrap_or("")),
            length: None,
            role: row.get("Role").filter(|v| !v.is_empty()).cloned(),
            core: row.get("Core").filter(|v| !v.is_empty()).cloned(),
            codelist_code: row
                .get("CDISC CT Codelist Code(s)")
                .filter(|v| !v.is_empty())
                .cloned(),
            order,
        };
        grouped.entry(dataset).or_default().push(variable);
    }

    let mut domains = Vec::new();
    for (code, mut vars) in grouped {
        let metadata = meta.get(&code);
        vars.sort_by(compare_variable_order);
        domains.push(Domain {
            code: code.clone(),
            description: metadata.and_then(|m| m.label.clone()),
            class_name: metadata.and_then(|m| m.class_name.clone()),
            dataset_class: metadata.and_then(|m| m.dataset_class),
            label: metadata.and_then(|m| m.label.clone()),
            structure: metadata.and_then(|m| m.structure.clone()),
            dataset_name: Some(code.clone()),
            variables: vars,
        });
    }
    domains.sort_by(|a, b| a.code.cmp(&b.code));
    Ok(domains)
}

fn compare_variable_order(left: &Variable, right: &Variable) -> std::cmp::Ordering {
    match (left.order, right.order) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.name.to_uppercase().cmp(&right.name.to_uppercase()),
    }
}
