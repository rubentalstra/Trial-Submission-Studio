#![deny(unsafe_code)]

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct SourceRef {
    /// Relative path (or other stable identifier) for the input.
    pub source: String,
    /// Record number within the parsed input (1-based, excluding header).
    pub record: u64,
    /// Optional column name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DerivationStep {
    pub rule_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
