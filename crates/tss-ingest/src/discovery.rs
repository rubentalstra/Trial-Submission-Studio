//! File discovery for study folders.

use std::path::{Path, PathBuf};

use crate::error::{IngestError, Result};

/// Lists all CSV files in a directory.
///
/// Returns files sorted by filename.
pub fn list_csv_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Err(IngestError::DirectoryNotFound {
            path: dir.to_path_buf(),
        });
    }

    let mut files = Vec::new();

    let entries = std::fs::read_dir(dir).map_err(|e| IngestError::DirectoryRead {
        path: dir.to_path_buf(),
        source: e,
    })?;

    for entry_result in entries {
        let entry = entry_result.map_err(|e| IngestError::DirectoryRead {
            path: dir.to_path_buf(),
            source: e,
        })?;

        let path = entry.path();

        // Skip directories
        if !path.is_file() {
            continue;
        }

        // Check for .csv extension (case-insensitive)
        let is_csv = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("csv"))
            .unwrap_or(false);

        if is_csv {
            files.push(path);
        }
    }

    // Sort by filename
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create some CSV files
        for name in &["STUDY_DM.csv", "STUDY_AE.csv", "README.csv"] {
            let path = dir.path().join(name);
            std::fs::write(&path, "header\ndata").unwrap();
        }

        dir
    }

    #[test]
    fn test_list_csv_files() {
        let dir = create_test_dir();
        let files = list_csv_files(dir.path()).unwrap();

        assert_eq!(files.len(), 3);
        // Should be sorted by filename
        assert!(
            files[0]
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains("README")
        );
    }

    #[test]
    fn test_list_csv_files_empty_dir() {
        let dir = TempDir::new().unwrap();
        let files = list_csv_files(dir.path()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_list_csv_files_not_a_directory() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.csv");
        std::fs::write(&file_path, "data").unwrap();

        let result = list_csv_files(&file_path);
        assert!(result.is_err());
    }
}
