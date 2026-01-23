//! Metadata file loading.
//!
//! Loads column metadata from Items.csv files for human-readable labels.

use std::path::Path;

use tss_standards::any_to_string;

use crate::csv::read_csv_table;
use crate::error::Result;

use super::detection::detect_items_schema;
use super::types::{SourceColumn, StudyMetadata};

/// Loads metadata from an explicit Items.csv file path.
///
/// Use this when the user has explicitly selected which file contains column
/// metadata (via manual source assignment). Only loads column labels - no
/// controlled terminology processing.
///
/// # Arguments
/// - `items_path`: Path to the Items.csv file
/// - `header_rows`: Number of header rows (1 = single header, 2 = double header with labels)
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use tss_ingest::load_items_metadata;
///
/// let items_path = Path::new("mockdata/STUDY001/Items.csv");
/// let metadata = load_items_metadata(items_path, 2)?;
///
/// // Use metadata for column labels
/// if let Some(col) = metadata.get_item("AGE") {
///     println!("AGE label: {}", col.label);
/// }
/// ```
pub fn load_items_metadata(items_path: &Path, header_rows: usize) -> Result<StudyMetadata> {
    let mut metadata = StudyMetadata::new();
    load_items_into(items_path, &mut metadata, header_rows)?;
    Ok(metadata)
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

    tracing::info!(count = metadata.items.len(), "Loaded items from metadata");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_items_file() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().unwrap();

        // Create Items.csv with double-header format
        let items_content = r#""Item Identifier","Item Label","Data Type","Required Flag","Format Name","Content Length"
"ID","Label","DataType","Mandatory","FormatName","ContentLength"
"AGE","Age in Years","integer","True","","3"
"SEX","Gender","text","True","SEX","1"
"RACE","Race","text","False","RACE","1"
"#;
        let items_path = dir.path().join("Items.csv");
        std::fs::write(&items_path, items_content).unwrap();

        (dir, items_path)
    }

    #[test]
    fn test_load_items_metadata() {
        let (_dir, items_path) = create_items_file();

        // Test with double headers (2 rows)
        let metadata = load_items_metadata(&items_path, 2).unwrap();

        assert!(!metadata.items.is_empty());
        assert_eq!(metadata.items.len(), 3);

        // Check items
        let age = metadata.get_item("AGE").unwrap();
        assert_eq!(age.label, "Age in Years");
        assert_eq!(age.data_type, Some("integer".to_string()));
        assert!(age.mandatory);

        let sex = metadata.get_item("SEX").unwrap();
        assert_eq!(sex.label, "Gender");
        assert_eq!(sex.format_name, Some("SEX".to_string()));

        let race = metadata.get_item("RACE").unwrap();
        assert!(!race.mandatory);
    }

    #[test]
    fn test_load_items_metadata_empty_file() {
        let dir = TempDir::new().unwrap();
        let items_path = dir.path().join("Empty_Items.csv");

        // Create empty Items.csv (just headers)
        let items_content = r#""Item Identifier","Item Label"
"ID","Label"
"#;
        std::fs::write(&items_path, items_content).unwrap();

        let metadata = load_items_metadata(&items_path, 2).unwrap();
        assert!(metadata.items.is_empty());
    }
}
