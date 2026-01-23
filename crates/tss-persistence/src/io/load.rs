//! Project loading operations.

use std::fs;
use std::path::Path;

use crate::error::{PersistenceError, Result};
use crate::types::{CURRENT_SCHEMA_VERSION, MAGIC_BYTES, ProjectFile};

/// Load a project from a .tss file.
pub fn load_project(path: &Path) -> Result<ProjectFile> {
    // Read the file
    let bytes = fs::read(path).map_err(|e| PersistenceError::Io {
        operation: "read",
        path: path.to_path_buf(),
        source: e,
    })?;

    // Validate and parse
    parse_project_bytes(&bytes, path)
}

/// Load a project asynchronously.
///
/// Spawns the load operation on a blocking thread pool to avoid
/// blocking the async runtime.
pub async fn load_project_async(path: std::path::PathBuf) -> Result<ProjectFile> {
    tokio::task::spawn_blocking(move || load_project(&path))
        .await
        .map_err(|e| PersistenceError::Deserialization {
            source: Box::new(e),
        })?
}

/// Parse project bytes and validate the format.
fn parse_project_bytes(bytes: &[u8], path: &Path) -> Result<ProjectFile> {
    // Minimum size: magic (4) + version (4) + some payload
    if bytes.len() < 12 {
        return Err(PersistenceError::InvalidFormat {
            path: path.to_path_buf(),
            reason: "File too small".to_string(),
        });
    }

    // Check magic bytes
    if bytes[0..4] != MAGIC_BYTES {
        return Err(PersistenceError::InvalidFormat {
            path: path.to_path_buf(),
            reason: "Not a TSS project file (invalid magic bytes)".to_string(),
        });
    }

    // Read schema version
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

    if version > CURRENT_SCHEMA_VERSION {
        return Err(PersistenceError::UnsupportedVersion {
            found: version,
            max_supported: CURRENT_SCHEMA_VERSION,
            path: path.to_path_buf(),
        });
    }

    // Extract rkyv payload
    let payload = &bytes[8..];

    // Deserialize with rkyv high-level API
    let project: ProjectFile = rkyv::from_bytes::<ProjectFile, rkyv::rancor::Error>(payload)
        .map_err(|e| PersistenceError::Deserialization {
            source: Box::new(std::io::Error::other(format!(
                "rkyv deserialization failed: {e}"
            ))),
        })?;

    tracing::info!("Loaded project from {}", path.display());
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::save::save_project;
    use crate::types::{StudyMetadata, WorkflowTypeSnapshot};
    use tempfile::tempdir;

    #[test]
    fn test_load_project_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.tss");

        // Create and save
        let study = StudyMetadata::new("TEST_STUDY", "/path/to/study", WorkflowTypeSnapshot::Sdtm);
        let mut project = ProjectFile::new(study);
        project.study.ct_version = Some("2024-03-29".to_string());

        save_project(&mut project, &path).unwrap();

        // Load and verify
        let loaded = load_project(&path).unwrap();
        assert_eq!(loaded.study.study_id, "TEST_STUDY");
        assert_eq!(loaded.study.ct_version, Some("2024-03-29".to_string()));
    }

    #[test]
    fn test_load_invalid_magic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid.tss");

        // Write invalid file
        fs::write(&path, b"NOT_A_TSS_FILE_DATA").unwrap();

        let result = load_project(&path);
        assert!(matches!(
            result,
            Err(PersistenceError::InvalidFormat { .. })
        ));
    }

    #[test]
    fn test_load_unsupported_version() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("future.tss");

        // Write file with future version
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC_BYTES);
        bytes.extend_from_slice(&999u32.to_le_bytes()); // Future version
        bytes.extend_from_slice(&[0u8; 100]); // Dummy payload

        fs::write(&path, bytes).unwrap();

        let result = load_project(&path);
        assert!(matches!(
            result,
            Err(PersistenceError::UnsupportedVersion { .. })
        ));
    }
}
