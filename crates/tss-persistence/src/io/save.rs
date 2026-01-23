//! Project saving operations.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::error::{PersistenceError, Result};
use crate::types::{CURRENT_SCHEMA_VERSION, MAGIC_BYTES, ProjectFile};

/// Save a project to a .tss file.
///
/// Uses atomic write (temp file + rename) to prevent data corruption
/// on crash or power loss.
pub fn save_project(project: &mut ProjectFile, path: &Path) -> Result<()> {
    // Update the last saved timestamp
    project.touch();

    // Serialize the project
    let bytes = serialize_project(project)?;

    // Write to a temp file first, then rename for atomicity
    let temp_path = path.with_extension("tss.tmp");

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| PersistenceError::Io {
            operation: "create directory",
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    // Write to temp file
    let mut file = File::create(&temp_path).map_err(|e| PersistenceError::Io {
        operation: "create",
        path: temp_path.clone(),
        source: e,
    })?;

    file.write_all(&bytes).map_err(|e| PersistenceError::Io {
        operation: "write",
        path: temp_path.clone(),
        source: e,
    })?;

    file.sync_all().map_err(|e| PersistenceError::Io {
        operation: "sync",
        path: temp_path.clone(),
        source: e,
    })?;

    // Atomic rename
    fs::rename(&temp_path, path).map_err(|e| PersistenceError::AtomicWriteFailed {
        temp_path: temp_path.clone(),
        target_path: path.to_path_buf(),
        source: e,
    })?;

    tracing::info!("Saved project to {}", path.display());
    Ok(())
}

/// Save a project asynchronously.
///
/// Spawns the save operation on a blocking thread pool to avoid
/// blocking the async runtime.
pub async fn save_project_async(project: ProjectFile, path: std::path::PathBuf) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let mut project = project;
        save_project(&mut project, &path)
    })
    .await
    .map_err(|e| PersistenceError::Serialization {
        source: Box::new(e),
    })?
}

/// Serialize a project to bytes.
///
/// Format:
/// - 4 bytes: Magic ("TSS\x01")
/// - 4 bytes: Schema version (u32 little-endian)
/// - N bytes: rkyv payload
fn serialize_project(project: &ProjectFile) -> Result<Vec<u8>> {
    // Serialize with rkyv using high-level API
    let rkyv_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(project).map_err(|e| {
        PersistenceError::Serialization {
            source: Box::new(std::io::Error::other(format!(
                "rkyv serialization failed: {e}"
            ))),
        }
    })?;

    // Build the final output
    let mut output = Vec::with_capacity(8 + rkyv_bytes.len());

    // Magic bytes
    output.extend_from_slice(&MAGIC_BYTES);

    // Schema version (little-endian)
    output.extend_from_slice(&CURRENT_SCHEMA_VERSION.to_le_bytes());

    // rkyv payload
    output.extend_from_slice(&rkyv_bytes);

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{StudyMetadata, WorkflowTypeSnapshot};
    use tempfile::tempdir;

    #[test]
    fn test_save_project() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.tss");

        let study = StudyMetadata::new("TEST_STUDY", "/path/to/study", WorkflowTypeSnapshot::Sdtm);
        let mut project = ProjectFile::new(study);

        save_project(&mut project, &path).unwrap();

        assert!(path.exists());

        // Check file starts with magic bytes
        let bytes = fs::read(&path).unwrap();
        assert_eq!(&bytes[0..4], &MAGIC_BYTES);
    }
}
