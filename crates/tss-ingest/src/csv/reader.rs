//! CSV file reading with explicit header row configuration.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use polars::prelude::*;

use crate::error::{IngestError, Result};

use super::header::{CsvHeaders, parse_csv_line};

/// Maximum file size for CSV loading (500 MB default).
pub const MAX_CSV_FILE_SIZE: u64 = 500 * 1024 * 1024;

/// Check file size before loading.
///
/// This is a sync function meant to be called via `spawn_blocking` from async contexts.
pub fn check_file_size(path: &Path) -> Result<()> {
    check_file_size_with_limit(path, MAX_CSV_FILE_SIZE)
}

/// Check file size against a custom limit.
pub fn check_file_size_with_limit(path: &Path, max_size: u64) -> Result<()> {
    let metadata = std::fs::metadata(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            IngestError::FileNotFound {
                path: path.to_path_buf(),
            }
        } else {
            IngestError::FileRead {
                path: path.to_path_buf(),
                source: e,
            }
        }
    })?;

    if metadata.len() > max_size {
        return Err(IngestError::FileTooLarge {
            path: path.to_path_buf(),
            size: metadata.len(),
            max_size,
        });
    }

    Ok(())
}

/// Detect encoding and validate it's supported (UTF-8 only).
///
/// Checks for UTF-16 BOM markers which are not supported.
pub fn validate_encoding(path: &Path) -> Result<()> {
    let mut file = File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            IngestError::FileNotFound {
                path: path.to_path_buf(),
            }
        } else {
            IngestError::FileRead {
                path: path.to_path_buf(),
                source: e,
            }
        }
    })?;

    let mut buffer = [0u8; 4];
    let bytes_read = file.read(&mut buffer).map_err(|e| IngestError::FileRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    // Check for UTF-16 BOM (not supported)
    if bytes_read >= 2 {
        // UTF-16 LE BOM
        if buffer[0..2] == [0xFF, 0xFE] {
            return Err(IngestError::UnsupportedEncoding {
                path: path.to_path_buf(),
                encoding: "UTF-16 LE",
            });
        }
        // UTF-16 BE BOM
        if buffer[0..2] == [0xFE, 0xFF] {
            return Err(IngestError::UnsupportedEncoding {
                path: path.to_path_buf(),
                encoding: "UTF-16 BE",
            });
        }
    }

    // UTF-8 BOM is acceptable (handled in read_first_lines)
    Ok(())
}

/// Validate DataFrame shape after loading.
///
/// Checks for:
/// - Empty DataFrame (no rows)
/// - Empty column names
/// - Warns about wide datasets (>500 columns)
pub fn validate_dataframe_shape(df: &DataFrame, path: &Path) -> Result<()> {
    // Empty DataFrame check
    if df.height() == 0 {
        return Err(IngestError::EmptyDataFrame {
            path: path.to_path_buf(),
        });
    }

    // Wide dataset warning (>500 columns)
    if df.width() > 500 {
        tracing::warn!(
            path = %path.display(),
            columns = df.width(),
            "Dataset has more than 500 columns - may impact performance"
        );
    }

    // Check for empty column names
    for name in df.get_column_names() {
        if name.trim().is_empty() {
            return Err(IngestError::EmptyColumnName {
                path: path.to_path_buf(),
            });
        }
    }

    Ok(())
}

/// Reads the first N lines from a file.
fn read_first_lines(path: &Path, n: usize) -> Result<Vec<String>> {
    let file = File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            IngestError::FileNotFound {
                path: path.to_path_buf(),
            }
        } else {
            IngestError::FileRead {
                path: path.to_path_buf(),
                source: e,
            }
        }
    })?;

    let reader = BufReader::new(file);
    let mut lines = Vec::with_capacity(n);

    for line_result in reader.lines().take(n) {
        let line = line_result.map_err(|e| IngestError::FileRead {
            path: path.to_path_buf(),
            source: e,
        })?;
        // Skip BOM if present
        let cleaned = line.strip_prefix('\u{feff}').unwrap_or(&line).to_string();
        lines.push(cleaned);
    }

    Ok(lines)
}

/// Reads CSV headers with explicit row count.
///
/// - `header_rows = 1`: Single header row (column names only)
/// - `header_rows = 2`: Double header (row 1 = labels, row 2 = column names)
pub fn read_csv_schema(path: &Path, header_rows: usize) -> Result<CsvHeaders> {
    let lines = read_first_lines(path, header_rows.max(1))?;

    if lines.is_empty() {
        return Err(IngestError::EmptyCsv {
            path: path.to_path_buf(),
        });
    }

    match header_rows {
        2 if lines.len() >= 2 => {
            let labels = parse_csv_line(&lines[0]);
            let columns = parse_csv_line(&lines[1]);
            Ok(CsvHeaders::double(labels, columns))
        }
        _ => {
            let columns = parse_csv_line(&lines[0]);
            if columns.is_empty() || columns.iter().all(String::is_empty) {
                return Err(IngestError::NoHeaderDetected {
                    path: path.to_path_buf(),
                });
            }
            Ok(CsvHeaders::single(columns))
        }
    }
}

/// Reads a CSV file into a Polars DataFrame with explicit header configuration.
///
/// - `header_rows = 1`: Single header row
/// - `header_rows = 2`: Double header (labels + column names)
///
/// Returns both the DataFrame and the header information.
pub fn read_csv_table(path: &Path, header_rows: usize) -> Result<(DataFrame, CsvHeaders)> {
    let headers = read_csv_schema(path, header_rows)?;

    // Skip additional rows beyond the first header row
    let skip_rows = header_rows.saturating_sub(1);

    let df = CsvReadOptions::default()
        .with_has_header(true)
        .with_skip_rows(skip_rows)
        .with_infer_schema_length(Some(100))
        .try_into_reader_with_file_path(Some(path.to_path_buf()))
        .map_err(|e| IngestError::CsvParse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?
        .finish()
        .map_err(|e| IngestError::CsvParse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

    Ok((df, headers))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_read_csv_schema_single_header() {
        let file = create_temp_csv("A,B,C\n1,2,3\n4,5,6\n");
        let headers = read_csv_schema(file.path(), 1).unwrap();

        assert_eq!(headers.columns, vec!["A", "B", "C"]);
        assert!(headers.labels.is_none());
        assert_eq!(headers.skip_rows, 1);
    }

    #[test]
    fn test_read_csv_schema_double_header() {
        let file = create_temp_csv(
            "Subject Identifier,Study Name,Visit Date\nUSUBJID,STUDYID,VISITDTC\nS001,STUDY1,2024-01-01\n",
        );
        let headers = read_csv_schema(file.path(), 2).unwrap();

        assert_eq!(headers.columns, vec!["USUBJID", "STUDYID", "VISITDTC"]);
        assert!(headers.labels.is_some());
        assert_eq!(
            headers.labels.as_ref().unwrap(),
            &vec!["Subject Identifier", "Study Name", "Visit Date"]
        );
        assert_eq!(headers.skip_rows, 2);
    }

    #[test]
    fn test_read_csv_schema_empty_file() {
        let file = create_temp_csv("");
        let result = read_csv_schema(file.path(), 1);

        assert!(matches!(result, Err(IngestError::EmptyCsv { .. })));
    }

    #[test]
    fn test_read_csv_schema_with_bom() {
        let file = create_temp_csv("\u{feff}A,B,C\n1,2,3\n");
        let headers = read_csv_schema(file.path(), 1).unwrap();

        assert_eq!(headers.columns, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_read_csv_table_single_header() {
        let file = create_temp_csv("A,B,C\n1,2,3\n4,5,6\n");
        let (df, headers) = read_csv_table(file.path(), 1).unwrap();

        assert_eq!(headers.columns, vec!["A", "B", "C"]);
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 3);
    }

    #[test]
    fn test_read_csv_table_double_header() {
        let file = create_temp_csv("Label A,Label B,Label C\nA,B,C\n1,2,3\n4,5,6\n");
        let (df, headers) = read_csv_table(file.path(), 2).unwrap();

        assert_eq!(headers.columns, vec!["A", "B", "C"]);
        assert_eq!(
            headers.labels.as_ref().unwrap(),
            &vec!["Label A", "Label B", "Label C"]
        );
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 3);
    }
}
