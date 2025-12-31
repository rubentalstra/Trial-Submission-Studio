//! XPT dataset and library structures.
//!
//! An XPT file can contain one or more datasets (members).
//! A library is a container for multiple datasets.

use chrono::NaiveDateTime;

use super::{XptColumn, XptValue};

/// A single dataset (member) in an XPT file.
///
/// Corresponds to a SAS dataset with variables (columns) and observations (rows).
#[derive(Debug, Clone, PartialEq)]
pub struct XptDataset {
    /// Dataset name (1-8 uppercase ASCII characters).
    pub name: String,

    /// Dataset label (max 40 characters).
    pub label: Option<String>,

    /// Dataset type (max 8 characters, optional).
    pub dataset_type: Option<String>,

    /// Column (variable) definitions.
    pub columns: Vec<XptColumn>,

    /// Data rows (observations).
    ///
    /// Each row is a vector of values corresponding to the columns.
    pub rows: Vec<Vec<XptValue>>,
}

impl XptDataset {
    /// Create a new empty dataset.
    ///
    /// # Arguments
    /// * `name` - Dataset name (1-8 characters, will be uppercased)
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: normalize_name(name.into()),
            label: None,
            dataset_type: None,
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Create a dataset with columns.
    ///
    /// # Arguments
    /// * `name` - Dataset name
    /// * `columns` - Column definitions
    #[must_use]
    pub fn with_columns(name: impl Into<String>, columns: Vec<XptColumn>) -> Self {
        Self {
            name: normalize_name(name.into()),
            label: None,
            dataset_type: None,
            columns,
            rows: Vec::new(),
        }
    }

    /// Set the dataset label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        let label_str = label.into();
        self.label = if label_str.is_empty() {
            None
        } else {
            Some(truncate_string(label_str, 40))
        };
        self
    }

    /// Set the dataset type.
    #[must_use]
    pub fn with_type(mut self, dataset_type: impl Into<String>) -> Self {
        let type_str = dataset_type.into();
        self.dataset_type = if type_str.is_empty() {
            None
        } else {
            Some(truncate_string(type_str.to_uppercase(), 8))
        };
        self
    }

    /// Add a column to the dataset.
    pub fn add_column(&mut self, column: XptColumn) {
        self.columns.push(column);
    }

    /// Add a row of values.
    ///
    /// # Panics
    /// Panics if the row length doesn't match the column count.
    pub fn add_row(&mut self, row: Vec<XptValue>) {
        assert_eq!(
            row.len(),
            self.columns.len(),
            "Row length {} doesn't match column count {}",
            row.len(),
            self.columns.len()
        );
        self.rows.push(row);
    }

    /// Try to add a row, returning an error if lengths don't match.
    ///
    /// # Errors
    /// Returns error if row length doesn't match column count.
    pub fn try_add_row(&mut self, row: Vec<XptValue>) -> Result<(), RowLengthError> {
        if row.len() != self.columns.len() {
            return Err(RowLengthError {
                expected: self.columns.len(),
                actual: row.len(),
            });
        }
        self.rows.push(row);
        Ok(())
    }

    /// Get the number of columns.
    #[must_use]
    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    /// Get the number of rows.
    #[must_use]
    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }

    /// Check if the dataset is empty (no rows).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get a column by index.
    #[must_use]
    pub fn column(&self, index: usize) -> Option<&XptColumn> {
        self.columns.get(index)
    }

    /// Get a column by name (case-insensitive).
    #[must_use]
    pub fn column_by_name(&self, name: &str) -> Option<&XptColumn> {
        let upper = name.to_uppercase();
        self.columns.iter().find(|c| c.name == upper)
    }

    /// Get the column index by name (case-insensitive).
    #[must_use]
    pub fn column_index(&self, name: &str) -> Option<usize> {
        let upper = name.to_uppercase();
        self.columns.iter().position(|c| c.name == upper)
    }

    /// Get a value at the specified row and column index.
    #[must_use]
    pub fn value(&self, row: usize, col: usize) -> Option<&XptValue> {
        self.rows.get(row).and_then(|r| r.get(col))
    }

    /// Get a row by index.
    #[must_use]
    pub fn row(&self, index: usize) -> Option<&[XptValue]> {
        self.rows.get(index).map(|r| r.as_slice())
    }

    /// Calculate the observation (row) length in bytes.
    #[must_use]
    pub fn observation_length(&self) -> usize {
        self.columns.iter().map(|c| c.length as usize).sum()
    }

    /// Get the effective label (label or name if no label).
    #[must_use]
    pub fn effective_label(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.name)
    }

    /// Iterate over rows.
    pub fn iter_rows(&self) -> impl Iterator<Item = &[XptValue]> {
        self.rows.iter().map(|r| r.as_slice())
    }

    /// Get column names.
    #[must_use]
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }
}

impl Default for XptDataset {
    fn default() -> Self {
        Self::new("DATA")
    }
}

/// Error when adding a row with wrong number of values.
#[derive(Debug, Clone, Copy)]
pub struct RowLengthError {
    /// Expected number of values (column count).
    pub expected: usize,
    /// Actual number of values provided.
    pub actual: usize,
}

impl std::fmt::Display for RowLengthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "row length mismatch: expected {}, got {}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for RowLengthError {}

/// An XPT library containing one or more datasets.
///
/// Represents the complete contents of an XPT file, which can contain
/// multiple members (datasets).
#[derive(Debug, Clone, PartialEq)]
pub struct XptLibrary {
    /// SAS version that created the file.
    pub sas_version: String,

    /// Operating system name.
    pub os_name: String,

    /// Datetime when the library was created.
    pub created: Option<NaiveDateTime>,

    /// Datetime when the library was last modified.
    pub modified: Option<NaiveDateTime>,

    /// Datasets (members) in the library.
    pub datasets: Vec<XptDataset>,
}

impl XptLibrary {
    /// Create a new empty library with default metadata.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sas_version: "9.4".to_string(),
            os_name: "RUST".to_string(),
            created: None,
            modified: None,
            datasets: Vec::new(),
        }
    }

    /// Create a library with a single dataset.
    #[must_use]
    pub fn single(dataset: XptDataset) -> Self {
        Self {
            sas_version: "9.4".to_string(),
            os_name: "RUST".to_string(),
            created: None,
            modified: None,
            datasets: vec![dataset],
        }
    }

    /// Set the SAS version.
    #[must_use]
    pub fn with_sas_version(mut self, version: impl Into<String>) -> Self {
        self.sas_version = truncate_string(version.into(), 8);
        self
    }

    /// Set the OS name.
    #[must_use]
    pub fn with_os_name(mut self, os: impl Into<String>) -> Self {
        self.os_name = truncate_string(os.into(), 8);
        self
    }

    /// Set the created datetime.
    #[must_use]
    pub fn with_created(mut self, datetime: NaiveDateTime) -> Self {
        self.created = Some(datetime);
        self
    }

    /// Set the modified datetime.
    #[must_use]
    pub fn with_modified(mut self, datetime: NaiveDateTime) -> Self {
        self.modified = Some(datetime);
        self
    }

    /// Add a dataset to the library.
    pub fn add_dataset(&mut self, dataset: XptDataset) {
        self.datasets.push(dataset);
    }

    /// Get the number of datasets.
    #[must_use]
    pub fn num_datasets(&self) -> usize {
        self.datasets.len()
    }

    /// Check if the library is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.datasets.is_empty()
    }

    /// Get a dataset by index.
    #[must_use]
    pub fn dataset(&self, index: usize) -> Option<&XptDataset> {
        self.datasets.get(index)
    }

    /// Get a dataset by name (case-insensitive).
    #[must_use]
    pub fn dataset_by_name(&self, name: &str) -> Option<&XptDataset> {
        let upper = name.to_uppercase();
        self.datasets.iter().find(|d| d.name == upper)
    }

    /// Get the first dataset (most XPT files have only one).
    #[must_use]
    pub fn first(&self) -> Option<&XptDataset> {
        self.datasets.first()
    }

    /// Get a mutable reference to the first dataset.
    #[must_use]
    pub fn first_mut(&mut self) -> Option<&mut XptDataset> {
        self.datasets.first_mut()
    }

    /// Iterate over datasets.
    pub fn iter(&self) -> impl Iterator<Item = &XptDataset> {
        self.datasets.iter()
    }

    /// Get dataset names.
    #[must_use]
    pub fn dataset_names(&self) -> Vec<&str> {
        self.datasets.iter().map(|d| d.name.as_str()).collect()
    }
}

impl Default for XptLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl From<XptDataset> for XptLibrary {
    fn from(dataset: XptDataset) -> Self {
        Self::single(dataset)
    }
}

/// Normalize a dataset name: trim and uppercase.
/// Note: Does NOT truncate; validation will catch names > 8 chars.
fn normalize_name(name: String) -> String {
    name.trim().to_uppercase()
}

/// Truncate a string to maximum length.
fn truncate_string(s: String, max_len: usize) -> String {
    if s.len() <= max_len {
        s
    } else {
        s.chars().take(max_len).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{XptColumn, XptType};

    #[test]
    fn test_dataset_new() {
        let ds = XptDataset::new("test");
        assert_eq!(ds.name, "TEST");
        assert!(ds.label.is_none());
        assert!(ds.columns.is_empty());
        assert!(ds.rows.is_empty());
    }

    #[test]
    fn test_dataset_with_columns() {
        let cols = vec![XptColumn::numeric("age"), XptColumn::character("name", 20)];
        let ds = XptDataset::with_columns("dm", cols);
        assert_eq!(ds.name, "DM");
        assert_eq!(ds.num_columns(), 2);
    }

    #[test]
    fn test_dataset_with_label() {
        let ds = XptDataset::new("dm").with_label("Demographics");
        assert_eq!(ds.label, Some("Demographics".to_string()));
    }

    #[test]
    fn test_dataset_add_row() {
        let mut ds = XptDataset::with_columns(
            "test",
            vec![XptColumn::numeric("x"), XptColumn::character("y", 10)],
        );

        ds.add_row(vec![XptValue::numeric(1.0), XptValue::character("a")]);
        assert_eq!(ds.num_rows(), 1);

        ds.add_row(vec![XptValue::numeric(2.0), XptValue::character("b")]);
        assert_eq!(ds.num_rows(), 2);
    }

    #[test]
    #[should_panic(expected = "Row length")]
    fn test_dataset_add_row_wrong_length() {
        let mut ds = XptDataset::with_columns("test", vec![XptColumn::numeric("x")]);
        ds.add_row(vec![XptValue::numeric(1.0), XptValue::numeric(2.0)]);
    }

    #[test]
    fn test_dataset_try_add_row() {
        let mut ds = XptDataset::with_columns("test", vec![XptColumn::numeric("x")]);

        assert!(ds.try_add_row(vec![XptValue::numeric(1.0)]).is_ok());
        assert!(
            ds.try_add_row(vec![XptValue::numeric(1.0), XptValue::numeric(2.0)])
                .is_err()
        );
    }

    #[test]
    fn test_dataset_column_lookup() {
        let ds = XptDataset::with_columns(
            "test",
            vec![XptColumn::numeric("age"), XptColumn::character("name", 20)],
        );

        assert!(ds.column(0).is_some());
        assert!(ds.column(2).is_none());

        let col = ds.column_by_name("AGE").unwrap();
        assert_eq!(col.data_type, XptType::Num);

        let col = ds.column_by_name("age").unwrap(); // case-insensitive
        assert_eq!(col.data_type, XptType::Num);

        assert!(ds.column_by_name("missing").is_none());

        assert_eq!(ds.column_index("name"), Some(1));
    }

    #[test]
    fn test_dataset_value_access() {
        let mut ds = XptDataset::with_columns(
            "test",
            vec![XptColumn::numeric("x"), XptColumn::numeric("y")],
        );
        ds.add_row(vec![XptValue::numeric(1.0), XptValue::numeric(2.0)]);
        ds.add_row(vec![XptValue::numeric(3.0), XptValue::numeric(4.0)]);

        assert_eq!(ds.value(0, 0).unwrap().as_f64(), Some(1.0));
        assert_eq!(ds.value(1, 1).unwrap().as_f64(), Some(4.0));
        assert!(ds.value(2, 0).is_none());

        let row = ds.row(0).unwrap();
        assert_eq!(row.len(), 2);
    }

    #[test]
    fn test_dataset_observation_length() {
        let ds = XptDataset::with_columns(
            "test",
            vec![
                XptColumn::numeric("x"),       // 8 bytes
                XptColumn::character("y", 20), // 20 bytes
            ],
        );
        assert_eq!(ds.observation_length(), 28);
    }

    #[test]
    fn test_library_new() {
        let lib = XptLibrary::new();
        assert_eq!(lib.sas_version, "9.4");
        assert_eq!(lib.os_name, "RUST");
        assert!(lib.is_empty());
    }

    #[test]
    fn test_library_single() {
        let ds = XptDataset::new("dm");
        let lib = XptLibrary::single(ds);
        assert_eq!(lib.num_datasets(), 1);
        assert_eq!(lib.first().unwrap().name, "DM");
    }

    #[test]
    fn test_library_from_dataset() {
        let ds = XptDataset::new("ae");
        let lib: XptLibrary = ds.into();
        assert_eq!(lib.num_datasets(), 1);
    }

    #[test]
    fn test_library_multiple_datasets() {
        let mut lib = XptLibrary::new();
        lib.add_dataset(XptDataset::new("dm"));
        lib.add_dataset(XptDataset::new("ae"));
        lib.add_dataset(XptDataset::new("cm"));

        assert_eq!(lib.num_datasets(), 3);
        assert_eq!(lib.dataset_names(), vec!["DM", "AE", "CM"]);

        assert!(lib.dataset_by_name("ae").is_some());
        assert!(lib.dataset_by_name("AE").is_some());
        assert!(lib.dataset_by_name("missing").is_none());
    }
}
