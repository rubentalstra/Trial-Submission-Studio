use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::csv_table::{CsvTable, read_csv_table};

#[derive(Debug, Clone, Default)]
pub struct StudyMetadata {
    pub items: BTreeMap<String, SourceColumn>,
    pub codelists: BTreeMap<String, CodeList>,
}

impl StudyMetadata {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.codelists.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct SourceColumn {
    pub id: String,
    pub label: String,
    pub data_type: Option<String>,
    pub mandatory: bool,
    pub format_name: Option<String>,
    pub content_length: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct CodeList {
    pub format_name: String,
    values: BTreeMap<String, String>,
    values_upper: BTreeMap<String, String>,
    values_numeric: BTreeMap<String, String>,
}

impl CodeList {
    fn new(format_name: String) -> Self {
        Self {
            format_name,
            values: BTreeMap::new(),
            values_upper: BTreeMap::new(),
            values_numeric: BTreeMap::new(),
        }
    }

    fn insert_value(&mut self, code_value: &str, code_text: &str) {
        let trimmed = code_value.trim();
        let text = code_text.trim();
        if trimmed.is_empty() || text.is_empty() {
            return;
        }
        self.values.insert(trimmed.to_string(), text.to_string());
        self.values_upper
            .insert(trimmed.to_uppercase(), text.to_string());
        if let Some(key) = normalize_numeric_key(trimmed) {
            self.values_numeric.insert(key, text.to_string());
        }
    }

    fn lookup_text(&self, raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(text) = self.values.get(trimmed) {
            return Some(text.clone());
        }
        let upper = trimmed.to_uppercase();
        if let Some(text) = self.values_upper.get(&upper) {
            return Some(text.clone());
        }
        if let Some(key) = normalize_numeric_key(trimmed) {
            if let Some(text) = self.values_numeric.get(&key) {
                return Some(text.clone());
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct AppliedStudyMetadata {
    pub table: CsvTable,
    pub code_to_base: BTreeMap<String, String>,
}

impl AppliedStudyMetadata {
    pub fn new(table: CsvTable) -> Self {
        Self {
            table,
            code_to_base: BTreeMap::new(),
        }
    }
}

const ITEMS_COLUMN_ID: &[&str] = &["ID", "Id", "ColumnId", "Column_Id", "ColumnID"];
const ITEMS_COLUMN_LABEL: &[&str] = &["Label", "ColumnLabel", "Column_Label", "Description"];
const ITEMS_COLUMN_TYPE: &[&str] = &["DataType", "Data Type", "Data_Type", "Type"];
const ITEMS_COLUMN_MANDATORY: &[&str] = &["Mandatory", "Required", "Req"];
const ITEMS_COLUMN_FORMAT: &[&str] = &[
    "FormatName",
    "Format Name",
    "Format_Name",
    "CodeList",
    "Codelist",
];
const ITEMS_COLUMN_LENGTH: &[&str] = &[
    "ContentLength",
    "Content Length",
    "Content_Length",
    "Length",
];

const CODELISTS_COLUMN_FORMAT: &[&str] = &[
    "FormatName",
    "Format Name",
    "Format_Name",
    "CodeListName",
    "Name",
];
const CODELISTS_COLUMN_VALUE: &[&str] = &["CodeValue", "Code Value", "Code_Value", "Code", "Value"];
const CODELISTS_COLUMN_TEXT: &[&str] = &[
    "CodeText",
    "Code Text",
    "Code_Text",
    "Text",
    "Label",
    "Decode",
];

pub fn load_study_metadata(study_folder: &Path) -> Result<StudyMetadata> {
    let (items_path, codelists_path) =
        discover_metadata_files(study_folder).context("discover metadata files")?;
    let mut metadata = StudyMetadata::default();
    if let Some(path) = items_path {
        metadata.items = load_items_csv(&path)
            .with_context(|| format!("load study metadata items: {}", path.display()))?;
    }
    if let Some(path) = codelists_path {
        metadata.codelists = load_codelists_csv(&path)
            .with_context(|| format!("load study metadata codelists: {}", path.display()))?;
    }
    Ok(metadata)
}

pub fn apply_study_metadata(table: CsvTable, metadata: &StudyMetadata) -> AppliedStudyMetadata {
    if metadata.items.is_empty() && metadata.codelists.is_empty() {
        return AppliedStudyMetadata::new(table);
    }
    let mut table = table;
    let row_count = table.rows.len();
    let mut labels = table
        .labels
        .unwrap_or_else(|| vec![String::new(); table.headers.len()]);
    if labels.len() < table.headers.len() {
        labels.resize(table.headers.len(), String::new());
    }
    let mut header_map = build_header_map(&table.headers);
    let mut code_to_base = BTreeMap::new();
    let mut new_columns: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut new_labels: BTreeMap<String, String> = BTreeMap::new();
    let mut new_upper: BTreeSet<String> = BTreeSet::new();

    for item in metadata.items.values() {
        let id_upper = item.id.to_uppercase();
        let Some(&code_idx) = header_map.get(&id_upper) else {
            continue;
        };
        if labels
            .get(code_idx)
            .map(|label| label.trim().is_empty())
            .unwrap_or(true)
            && !item.label.trim().is_empty()
        {
            labels[code_idx] = item.label.clone();
        }
        let Some(format_name) = item.format_name.as_ref() else {
            continue;
        };
        let Some(codelist) = metadata.codelists.get(&format_name.to_uppercase()) else {
            continue;
        };
        let Some(base_name) = base_column_name(&table.headers[code_idx]) else {
            continue;
        };
        let base_upper = base_name.to_uppercase();
        if let Some(&base_idx) = header_map.get(&base_upper) {
            let base_label = strip_code_label(&item.label);
            if labels
                .get(base_idx)
                .map(|label| label.trim().is_empty())
                .unwrap_or(true)
                && !base_label.trim().is_empty()
            {
                labels[base_idx] = base_label;
            }
            for row in table.rows.iter_mut() {
                let code_value = row.get(code_idx).map(String::as_str).unwrap_or("");
                let decoded = codelist.lookup_text(code_value);
                if let Some(text) = decoded {
                    if row
                        .get(base_idx)
                        .map(|value| value.trim().is_empty())
                        .unwrap_or(true)
                    {
                        row[base_idx] = text;
                    }
                }
            }
            code_to_base.insert(
                table.headers[code_idx].clone(),
                table.headers[base_idx].clone(),
            );
        } else if !new_upper.contains(&base_upper) {
            let mut decoded_values = Vec::with_capacity(row_count);
            let mut any_value = false;
            for row in &table.rows {
                let code_value = row.get(code_idx).map(String::as_str).unwrap_or("");
                if let Some(text) = codelist.lookup_text(code_value) {
                    decoded_values.push(text);
                    any_value = true;
                } else {
                    decoded_values.push(String::new());
                }
            }
            if any_value {
                new_upper.insert(base_upper.clone());
                new_columns.insert(base_name.clone(), decoded_values);
                let base_label = strip_code_label(&item.label);
                if !base_label.trim().is_empty() {
                    new_labels.insert(base_name.clone(), base_label);
                }
                code_to_base.insert(table.headers[code_idx].clone(), base_name.clone());
            }
        }
    }

    if !new_columns.is_empty() {
        for (name, values) in new_columns {
            if header_map.contains_key(&name.to_uppercase()) {
                continue;
            }
            header_map.insert(name.to_uppercase(), table.headers.len());
            table.headers.push(name.clone());
            labels.push(new_labels.get(&name).cloned().unwrap_or_default());
            for (row_idx, row) in table.rows.iter_mut().enumerate() {
                let value = values.get(row_idx).cloned().unwrap_or_default();
                row.push(value);
            }
        }
    }

    table.labels = if labels.iter().any(|label| !label.trim().is_empty()) {
        Some(labels)
    } else {
        None
    };

    AppliedStudyMetadata {
        table,
        code_to_base,
    }
}

fn load_items_csv(path: &Path) -> Result<BTreeMap<String, SourceColumn>> {
    let table =
        read_csv_table(path).with_context(|| format!("read items csv: {}", path.display()))?;
    let header_map = build_header_map(&table.headers);
    let id_idx = find_column_index(&header_map, ITEMS_COLUMN_ID)
        .ok_or_else(|| anyhow::anyhow!("could not find ID column in {}", path.display()))?;
    let label_idx = find_column_index(&header_map, ITEMS_COLUMN_LABEL);
    let type_idx = find_column_index(&header_map, ITEMS_COLUMN_TYPE);
    let mandatory_idx = find_column_index(&header_map, ITEMS_COLUMN_MANDATORY);
    let format_idx = find_column_index(&header_map, ITEMS_COLUMN_FORMAT);
    let length_idx = find_column_index(&header_map, ITEMS_COLUMN_LENGTH);

    let mut items = BTreeMap::new();
    for row in &table.rows {
        let col_id = row.get(id_idx).map(String::as_str).unwrap_or("").trim();
        if col_id.is_empty() {
            continue;
        }
        if matches!(col_id.to_lowercase().as_str(), "id" | "columnid") {
            continue;
        }
        let label = label_idx
            .and_then(|idx| row.get(idx))
            .map(String::as_str)
            .unwrap_or(col_id)
            .trim()
            .to_string();
        let data_type = type_idx
            .and_then(|idx| row.get(idx))
            .map(String::as_str)
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());
        let mandatory = mandatory_idx
            .and_then(|idx| row.get(idx))
            .map(String::as_str)
            .map(parse_bool)
            .unwrap_or(false);
        let format_name = format_idx
            .and_then(|idx| row.get(idx))
            .map(String::as_str)
            .and_then(parse_format_name);
        let content_length = length_idx
            .and_then(|idx| row.get(idx))
            .map(String::as_str)
            .and_then(parse_content_length);

        items.insert(
            col_id.to_uppercase(),
            SourceColumn {
                id: col_id.to_string(),
                label,
                data_type,
                mandatory,
                format_name,
                content_length,
            },
        );
    }
    Ok(items)
}

fn load_codelists_csv(path: &Path) -> Result<BTreeMap<String, CodeList>> {
    let table =
        read_csv_table(path).with_context(|| format!("read codelists csv: {}", path.display()))?;
    let header_map = build_header_map(&table.headers);
    let format_idx = find_column_index(&header_map, CODELISTS_COLUMN_FORMAT)
        .ok_or_else(|| anyhow::anyhow!("could not find Format column in {}", path.display()))?;
    let value_idx = find_column_index(&header_map, CODELISTS_COLUMN_VALUE)
        .ok_or_else(|| anyhow::anyhow!("could not find Code Value column in {}", path.display()))?;
    let text_idx = find_column_index(&header_map, CODELISTS_COLUMN_TEXT)
        .ok_or_else(|| anyhow::anyhow!("could not find Code Text column in {}", path.display()))?;

    let mut codelists = BTreeMap::new();
    for row in &table.rows {
        let format_name = row.get(format_idx).map(String::as_str).unwrap_or("").trim();
        if format_name.is_empty() {
            continue;
        }
        if matches!(
            format_name.to_lowercase().as_str(),
            "formatname" | "format name"
        ) {
            continue;
        }
        let code_value = row.get(value_idx).map(String::as_str).unwrap_or("").trim();
        let code_text = row.get(text_idx).map(String::as_str).unwrap_or("").trim();
        if code_value.is_empty() || code_text.is_empty() {
            continue;
        }
        let key = format_name.to_uppercase();
        let entry = codelists
            .entry(key)
            .or_insert_with(|| CodeList::new(format_name.to_string()));
        entry.insert_value(code_value, code_text);
    }
    Ok(codelists)
}

fn discover_metadata_files(study_folder: &Path) -> Result<(Option<PathBuf>, Option<PathBuf>)> {
    let mut items = Vec::new();
    let mut codelists = Vec::new();
    if !study_folder.exists() {
        return Ok((None, None));
    }
    for entry in std::fs::read_dir(study_folder)
        .with_context(|| format!("read study folder: {}", study_folder.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("csv"))
            != Some(true)
        {
            continue;
        }
        let filename = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("")
            .to_uppercase();
        if filename.contains("ITEMS") {
            items.push(path);
        } else if filename.contains("CODELIST") {
            codelists.push(path);
        }
    }
    items.sort();
    codelists.sort();
    Ok((items.into_iter().next(), codelists.into_iter().next()))
}

fn build_header_map(headers: &[String]) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for (idx, header) in headers.iter().enumerate() {
        map.insert(header.to_uppercase(), idx);
    }
    map
}

fn find_column_index(map: &BTreeMap<String, usize>, candidates: &[&str]) -> Option<usize> {
    for candidate in candidates {
        let key = candidate.trim().to_uppercase();
        if let Some(idx) = map.get(&key) {
            return Some(*idx);
        }
    }
    None
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_lowercase().as_str(),
        "true" | "yes" | "1" | "y" | "req"
    )
}

fn parse_format_name(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if matches!(trimmed.to_lowercase().as_str(), "nan" | "none") {
        return None;
    }
    Some(trimmed.to_string())
}

fn parse_content_length(value: &str) -> Option<usize> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed
        .parse::<f64>()
        .ok()
        .map(|value| value.round() as usize)
}

fn strip_code_label(label: &str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let upper = trimmed.to_uppercase();
    for suffix in [" - CODE", "- CODE", " CODE"] {
        if upper.ends_with(suffix) {
            let cut = trimmed.len().saturating_sub(suffix.len());
            return trimmed[..cut].trim().to_string();
        }
    }
    trimmed.to_string()
}

fn base_column_name(header: &str) -> Option<String> {
    let upper = header.to_uppercase();
    if upper.ends_with("CD") && header.len() > 2 {
        Some(header[..header.len() - 2].to_string())
    } else {
        None
    }
}

fn normalize_numeric_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = trimmed.parse::<f64>().ok()?;
    let mut text = format!("{parsed}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    if text.is_empty() { None } else { Some(text) }
}
