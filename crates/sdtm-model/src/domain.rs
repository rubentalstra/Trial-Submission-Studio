use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    Char,
    Num,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub label: Option<String>,
    pub data_type: VariableType,
    pub length: Option<u32>,
    pub role: Option<String>,
    pub core: Option<String>,
    pub codelist_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub code: String,
    pub description: Option<String>,
    pub class_name: Option<String>,
    pub label: Option<String>,
    pub structure: Option<String>,
    pub dataset_name: Option<String>,
    pub variables: Vec<Variable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub dataset_name: String,
    pub class_name: Option<String>,
    pub label: Option<String>,
    pub structure: Option<String>,
}
