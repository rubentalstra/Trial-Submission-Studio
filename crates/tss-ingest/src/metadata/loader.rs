//! Metadata file loading.

use std::path::Path;

use tss_common::any_to_string;

use crate::csv::read_csv_table;
use crate::error::{IngestError, Result};

use super::detection::{detect_codelist_schema, detect_items_schema};
use super::types::{SourceColumn, StudyCodelist, StudyMetadata};

/// Discovers and loads study metadata from a directory.
///
/// Looks for Items.csv and CodeLists.csv files (with common naming patterns).
///
/// # Arguments
/// - `dir`: The directory containing metadata files
/// - `header_rows`: Number of header rows (1 = single header, 2 = double header with labels)
pub fn load_study_metadata(dir: &Path, header_rows: usize) -> Result<StudyMetadata> {
    if !dir.is_dir() {
        return Err(IngestError::DirectoryNotFound {
            path: dir.to_path_buf(),
        });
    }

    let mut metadata = StudyMetadata::new();

    // Find Items.csv
    if let Some(items_path) = find_metadata_file(dir, "Items") {
        tracing::debug!(path = %items_path.display(), "Loading Items.csv");
        load_items_into(&items_path, &mut metadata, header_rows)?;
    }

    // Find CodeLists.csv
    if let Some(codelists_path) = find_metadata_file(dir, "CodeLists") {
        tracing::debug!(path = %codelists_path.display(), "Loading CodeLists.csv");
        load_codelists_into(&codelists_path, &mut metadata, header_rows)?;
    }

    Ok(metadata)
}

/// Finds a metadata file with common naming patterns.
fn find_metadata_file(dir: &Path, suffix: &str) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let upper = name.to_uppercase();
                // Match patterns like "STUDY_Items.csv", "Items.csv", "STUDY_ITEMS.CSV"
                if upper.ends_with(&format!("{}.CSV", suffix.to_uppercase()))
                    || upper.ends_with(&format!("_{}.CSV", suffix.to_uppercase()))
                {
                    return Some(path);
                }
            }
        }
    }

    None
}

/// Loads items from Items.csv into the metadata.
fn load_items_into(path: &Path, metadata: &mut StudyMetadata, header_rows: usize) -> Result<()> {
    let (df, _headers) = read_csv_table(path, header_rows)?;

    if df.height() == 0 {
        return Ok(()); // Empty file, nothing to load
    }

    // Detect schema dynamically
    let schema = detect_items_schema(&df, path)?;

    tracing::debug!(
        id_col = %schema.id.name,
        label_col = %schema.label.name,
        data_type_col = ?schema.data_type.as_ref().map(|c| &c.name),
        "Detected Items schema"
    );

    // Extract columns
    let id_col = df.column(&schema.id.name)?;
    let label_col = df.column(&schema.label.name)?;

    let data_type_col = schema
        .data_type
        .as_ref()
        .and_then(|c| df.column(&c.name).ok());
    let mandatory_col = schema
        .mandatory
        .as_ref()
        .and_then(|c| df.column(&c.name).ok());
    let format_col = schema
        .format_name
        .as_ref()
        .and_then(|c| df.column(&c.name).ok());
    let length_col = schema
        .content_length
        .as_ref()
        .and_then(|c| df.column(&c.name).ok());

    // Iterate and build SourceColumn entries
    for row_idx in 0..df.height() {
        let id = any_to_string(id_col.get(row_idx)?);
        let label = any_to_string(label_col.get(row_idx)?);

        if id.is_empty() {
            continue; // Skip rows with empty ID
        }

        let mut item = SourceColumn::new(id, label);

        // Data type
        if let Some(col) = &data_type_col {
            let val = any_to_string(col.get(row_idx)?);
            if !val.is_empty() {
                item = item.with_data_type(val);
            }
        }

        // Mandatory
        if let Some(col) = &mandatory_col {
            let val = any_to_string(col.get(row_idx)?).to_lowercase();
            let mandatory = val == "true" || val == "yes" || val == "y" || val == "1";
            item = item.with_mandatory(mandatory);
        }

        // Format name
        if let Some(col) = &format_col {
            let val = any_to_string(col.get(row_idx)?);
            if !val.is_empty() {
                item = item.with_format(val);
            }
        }

        // Content length
        if let Some(col) = &length_col {
            let val = any_to_string(col.get(row_idx)?);
            if let Ok(len) = val.parse::<usize>() {
                item = item.with_length(len);
            }
        }

        metadata.add_item(item);
    }

    tracing::info!(count = metadata.items.len(), "Loaded items");
    Ok(())
}

/// Loads codelists from CodeLists.csv into the metadata.
fn load_codelists_into(
    path: &Path,
    metadata: &mut StudyMetadata,
    header_rows: usize,
) -> Result<()> {
    let (df, _headers) = read_csv_table(path, header_rows)?;

    if df.height() == 0 {
        return Ok(()); // Empty file, nothing to load
    }

    // Detect schema dynamically
    let schema = detect_codelist_schema(&df, path)?;

    tracing::debug!(
        format_col = %schema.format_name.name,
        value_col = %schema.code_value.name,
        text_col = %schema.code_text.name,
        "Detected CodeLists schema"
    );

    // Extract columns
    let format_col = df.column(&schema.format_name.name)?;
    let value_col = df.column(&schema.code_value.name)?;
    let text_col = df.column(&schema.code_text.name)?;

    // Build codelists
    for row_idx in 0..df.height() {
        let format_name = any_to_string(format_col.get(row_idx)?);
        let code_value = any_to_string(value_col.get(row_idx)?);
        let code_text = any_to_string(text_col.get(row_idx)?);

        if format_name.is_empty() {
            continue;
        }

        // Get or create codelist
        let key = format_name.to_uppercase();
        let codelist = metadata
            .codelists
            .entry(key)
            .or_insert_with(|| StudyCodelist::new(&format_name));

        codelist.insert(&code_value, &code_text);
    }

    tracing::info!(count = metadata.codelists.len(), "Loaded codelists");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create Items.csv
        let items_content = r#""Item Identifier","Item Label","Data Type","Required Flag","Format Name","Content Length"
"ID","Label","DataType","Mandatory","FormatName","ContentLength"
"AGE","Age in Years","integer","True","","3"
"SEX","Gender","text","True","SEX","1"
"RACE","Race","text","False","RACE","1"
"#;
        let items_path = dir.path().join("TEST_Items.csv");
        std::fs::write(&items_path, items_content).unwrap();

        // Create CodeLists.csv
        let codelists_content = r#""Format Name","Data Type","Code Value","Code Text"
"FormatName","DataType","CodeValue","CodeText"
"SEX","text","M","Male"
"SEX","text","F","Female"
"RACE","integer","1","Asian"
"RACE","integer","2","Black"
"#;
        let codelists_path = dir.path().join("TEST_CodeLists.csv");
        std::fs::write(&codelists_path, codelists_content).unwrap();

        dir
    }

    #[test]
    fn test_load_study_metadata() {
        let dir = create_test_dir();
        // Test with double headers (2 rows)
        let metadata = load_study_metadata(dir.path(), 2).unwrap();

        assert!(!metadata.items.is_empty());
        assert!(!metadata.codelists.is_empty());

        // Check items
        let age = metadata.get_item("AGE").unwrap();
        assert_eq!(age.label, "Age in Years");
        assert_eq!(age.data_type, Some("integer".to_string()));
        assert!(age.mandatory);

        let sex = metadata.get_item("SEX").unwrap();
        assert_eq!(sex.format_name, Some("SEX".to_string()));

        // Check codelists
        let sex_cl = metadata.get_codelist("SEX").unwrap();
        assert_eq!(sex_cl.lookup("M"), Some("Male"));
        assert_eq!(sex_cl.lookup("F"), Some("Female"));
    }

    #[test]
    fn test_find_metadata_file() {
        let dir = create_test_dir();

        let items = find_metadata_file(dir.path(), "Items");
        assert!(items.is_some());

        let codelists = find_metadata_file(dir.path(), "CodeLists");
        assert!(codelists.is_some());

        let missing = find_metadata_file(dir.path(), "Missing");
        assert!(missing.is_none());
    }
}
