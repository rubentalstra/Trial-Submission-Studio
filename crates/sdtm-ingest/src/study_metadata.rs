use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use polars::prelude::*;

use crate::csv_table::read_csv_table_with_header_match;

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
        if let Some(key) = normalize_numeric_key(trimmed)
            && let Some(text) = self.values_numeric.get(&key)
        {
            return Some(text.clone());
        }
        None
    }
}

type ItemColumnIndices = (
    usize,
    Option<usize>,
    Option<usize>,
    Option<usize>,
    Option<usize>,
    Option<usize>,
);

#[derive(Debug, Clone)]
pub struct AppliedStudyMetadata {
    pub table: DataFrame,
    pub code_to_base: BTreeMap<String, String>,
    pub derived_columns: BTreeSet<String>,
}

impl AppliedStudyMetadata {
    pub fn new(table: DataFrame) -> Self {
        Self {
            table,
            code_to_base: BTreeMap::new(),
            derived_columns: BTreeSet::new(),
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

pub fn apply_study_metadata(
    mut table: DataFrame,
    metadata: &StudyMetadata,
) -> AppliedStudyMetadata {
    if metadata.items.is_empty() && metadata.codelists.is_empty() {
        return AppliedStudyMetadata::new(table);
    }

    let mut code_to_base = BTreeMap::new();
    let mut derived_columns = BTreeSet::new();
    let mut new_columns = Vec::new();

    let column_names: Vec<String> = table
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let header_map: BTreeMap<String, String> = column_names
        .iter()
        .map(|n| (n.to_uppercase(), n.clone()))
        .collect();

    for item in metadata.items.values() {
        let id_upper = item.id.to_uppercase();
        let Some(col_name) = header_map.get(&id_upper) else {
            continue;
        };

        let Some(format_name) = item.format_name.as_ref() else {
            continue;
        };
        let Some(codelist) = metadata.codelists.get(&format_name.to_uppercase()) else {
            continue;
        };
        let Some(base_name) = base_column_name(col_name) else {
            continue;
        };
        let base_upper = base_name.to_uppercase();

        if let Some(existing_base) = header_map.get(&base_upper) {
            let code_series = table.column(col_name).unwrap();
            let base_series = table.column(existing_base).unwrap();

            let decoded_chunked = code_series.str().unwrap().apply(|opt_val| {
                if let Some(val) = opt_val {
                    Some(std::borrow::Cow::Owned(
                        codelist.lookup_text(val).unwrap_or_default(),
                    ))
                } else {
                    Some(std::borrow::Cow::Borrowed(""))
                }
            });
            let decoded_series = decoded_chunked.into_series();

            let base_str = base_series.str().unwrap();
            let mask = base_series.is_null() | base_str.equal("");

            let decoded_col = Column::from(decoded_series);
            if let Ok(new_base) = base_series.zip_with(&mask, &decoded_col) {
                let _ = table.with_column(new_base.with_name(existing_base.into()));
            }

            code_to_base.insert(col_name.clone(), existing_base.clone());
        } else {
            if derived_columns.contains(&base_upper) {
                continue;
            }

            let code_series = table.column(col_name).unwrap();
            let decoded_chunked = code_series.str().unwrap().apply(|opt_val| {
                if let Some(val) = opt_val {
                    if let Some(text) = codelist.lookup_text(val) {
                        return Some(std::borrow::Cow::Owned(text));
                    }
                }
                Some(std::borrow::Cow::Borrowed(""))
            });

            let has_any = decoded_chunked
                .into_iter()
                .any(|opt| opt.map(|s| !s.is_empty()).unwrap_or(false));

            if has_any {
                let mut new_series = decoded_chunked.into_series();
                new_series.rename((&base_name).into());
                new_columns.push(new_series);
                derived_columns.insert(base_upper);
                code_to_base.insert(col_name.clone(), base_name);
            }
        }
    }

    for col in new_columns {
        let _ = table.with_column(col);
    }

    AppliedStudyMetadata {
        table,
        code_to_base,
        derived_columns,
    }
}

fn load_items_csv(path: &Path) -> Result<BTreeMap<String, SourceColumn>> {
    let table = read_csv_table_with_header_match(path, 25, matches_items_header)
        .with_context(|| format!("read items csv: {}", path.display()))?;
    let column_names: Vec<String> = table
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let (id_idx, label_idx, type_idx, mandatory_idx, format_idx, length_idx) =
        items_column_indices(&column_names, path)?;
    Ok(collect_items(
        &table,
        id_idx,
        label_idx,
        type_idx,
        mandatory_idx,
        format_idx,
        length_idx,
    ))
}

fn load_codelists_csv(path: &Path) -> Result<BTreeMap<String, CodeList>> {
    let table = read_csv_table_with_header_match(path, 25, matches_codelists_header)
        .with_context(|| format!("read codelists csv: {}", path.display()))?;
    let column_names: Vec<String> = table
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let (format_idx, value_idx, text_idx) = codelist_column_indices(&column_names, path)?;
    Ok(collect_codelists(&table, format_idx, value_idx, text_idx))
}

fn codelist_column_indices(headers: &[String], path: &Path) -> Result<(usize, usize, usize)> {
    let header_map = build_header_map(headers);
    let format_idx = find_column_index(&header_map, CODELISTS_COLUMN_FORMAT)
        .ok_or_else(|| anyhow::anyhow!("could not find Format column in {}", path.display()))?;
    let value_idx = find_column_index(&header_map, CODELISTS_COLUMN_VALUE)
        .ok_or_else(|| anyhow::anyhow!("could not find Code Value column in {}", path.display()))?;
    let text_idx = find_column_index(&header_map, CODELISTS_COLUMN_TEXT)
        .ok_or_else(|| anyhow::anyhow!("could not find Code Text column in {}", path.display()))?;
    Ok((format_idx, value_idx, text_idx))
}

fn items_column_indices(headers: &[String], path: &Path) -> Result<ItemColumnIndices> {
    let header_map = build_header_map(headers);
    let id_idx = find_column_index(&header_map, ITEMS_COLUMN_ID)
        .ok_or_else(|| anyhow::anyhow!("could not find ID column in {}", path.display()))?;
    let label_idx = find_column_index(&header_map, ITEMS_COLUMN_LABEL);
    let type_idx = find_column_index(&header_map, ITEMS_COLUMN_TYPE);
    let mandatory_idx = find_column_index(&header_map, ITEMS_COLUMN_MANDATORY);
    let format_idx = find_column_index(&header_map, ITEMS_COLUMN_FORMAT);
    let length_idx = find_column_index(&header_map, ITEMS_COLUMN_LENGTH);
    Ok((
        id_idx,
        label_idx,
        type_idx,
        mandatory_idx,
        format_idx,
        length_idx,
    ))
}

fn matches_items_header(headers: &[String]) -> bool {
    let header_map = build_header_map(headers);
    if find_column_index(&header_map, ITEMS_COLUMN_ID).is_none() {
        return false;
    }
    find_column_index(&header_map, ITEMS_COLUMN_LABEL).is_some()
        || find_column_index(&header_map, ITEMS_COLUMN_TYPE).is_some()
        || find_column_index(&header_map, ITEMS_COLUMN_FORMAT).is_some()
        || find_column_index(&header_map, ITEMS_COLUMN_LENGTH).is_some()
        || find_column_index(&header_map, ITEMS_COLUMN_MANDATORY).is_some()
}

fn matches_codelists_header(headers: &[String]) -> bool {
    let header_map = build_header_map(headers);
    find_column_index(&header_map, CODELISTS_COLUMN_FORMAT).is_some()
        && find_column_index(&header_map, CODELISTS_COLUMN_VALUE).is_some()
        && find_column_index(&header_map, CODELISTS_COLUMN_TEXT).is_some()
}

fn collect_items(
    df: &DataFrame,
    id_idx: usize,
    label_idx: Option<usize>,
    type_idx: Option<usize>,
    mandatory_idx: Option<usize>,
    format_idx: Option<usize>,
    length_idx: Option<usize>,
) -> BTreeMap<String, SourceColumn> {
    let mut items = BTreeMap::new();

    let id_series = df
        .select_at_idx(id_idx)
        .unwrap()
        .cast(&DataType::String)
        .unwrap();
    let id_col = id_series.str().unwrap();

    let label_series = label_idx.map(|i| {
        df.select_at_idx(i)
            .unwrap()
            .cast(&DataType::String)
            .unwrap()
    });
    let label_col = label_series.as_ref().map(|s| s.str().unwrap());

    let type_series = type_idx.map(|i| {
        df.select_at_idx(i)
            .unwrap()
            .cast(&DataType::String)
            .unwrap()
    });
    let type_col = type_series.as_ref().map(|s| s.str().unwrap());

    let mandatory_series = mandatory_idx.map(|i| {
        df.select_at_idx(i)
            .unwrap()
            .cast(&DataType::String)
            .unwrap()
    });
    let mandatory_col = mandatory_series.as_ref().map(|s| s.str().unwrap());

    let format_series = format_idx.map(|i| {
        df.select_at_idx(i)
            .unwrap()
            .cast(&DataType::String)
            .unwrap()
    });
    let format_col = format_series.as_ref().map(|s| s.str().unwrap());

    let length_series = length_idx.map(|i| {
        df.select_at_idx(i)
            .unwrap()
            .cast(&DataType::String)
            .unwrap()
    });
    let length_col = length_series.as_ref().map(|s| s.str().unwrap());

    for (i, opt_id) in id_col.into_iter().enumerate() {
        let Some(col_id) = opt_id else { continue };
        let col_id = col_id.trim();
        if col_id.is_empty() {
            continue;
        }
        if matches!(col_id.to_lowercase().as_str(), "id" | "columnid") {
            continue;
        }
        let label = label_col
            .and_then(|s| s.get(i))
            .unwrap_or(col_id)
            .trim()
            .to_string();
        let data_type = type_col
            .and_then(|s| s.get(i))
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());
        let mandatory = mandatory_col
            .and_then(|s| s.get(i))
            .map(parse_bool)
            .unwrap_or(false);
        let format_name = format_col
            .and_then(|s| s.get(i))
            .and_then(parse_format_name);
        let content_length = length_col
            .and_then(|s| s.get(i))
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
    items
}

fn collect_codelists(
    df: &DataFrame,
    format_idx: usize,
    value_idx: usize,
    text_idx: usize,
) -> BTreeMap<String, CodeList> {
    let mut codelists = BTreeMap::new();

    let format_series = df
        .select_at_idx(format_idx)
        .unwrap()
        .cast(&DataType::String)
        .unwrap();
    let format_col = format_series.str().unwrap();

    let value_series = df
        .select_at_idx(value_idx)
        .unwrap()
        .cast(&DataType::String)
        .unwrap();
    let value_col = value_series.str().unwrap();

    let text_series = df
        .select_at_idx(text_idx)
        .unwrap()
        .cast(&DataType::String)
        .unwrap();
    let text_col = text_series.str().unwrap();

    for (i, opt_format) in format_col.into_iter().enumerate() {
        let Some(format_name) = opt_format else {
            continue;
        };
        let format_name = format_name.trim();
        if format_name.is_empty() {
            continue;
        }
        if matches!(
            format_name.to_lowercase().as_str(),
            "formatname" | "format name"
        ) {
            continue;
        }
        let code_value = value_col.get(i).unwrap_or("").trim();
        let code_text = text_col.get(i).unwrap_or("").trim();
        if code_value.is_empty() || code_text.is_empty() {
            continue;
        }
        let key = format_name.to_uppercase();
        let entry = codelists
            .entry(key)
            .or_insert_with(|| CodeList::new(format_name.to_string()));
        entry.insert_value(code_value, code_text);
    }
    codelists
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
