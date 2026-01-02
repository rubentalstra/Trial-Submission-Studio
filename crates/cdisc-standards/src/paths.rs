//! Standards directory path resolution.

use std::path::PathBuf;

/// Environment variable for overriding the standards directory.
pub const STANDARDS_ENV_VAR: &str = "CDISC_STANDARDS_DIR";

/// Get the standards root directory.
///
/// Resolution order:
/// 1. `CDISC_STANDARDS_DIR` environment variable
/// 2. `standards/` directory relative to workspace root
///
/// # Example
///
/// ```rust,ignore
/// let root = cdisc_standards::standards_root();
/// let sdtm_ig_dir = root.join("sdtm/ig/v3.4");
/// ```
pub fn standards_root() -> PathBuf {
    if let Ok(root) = std::env::var(STANDARDS_ENV_VAR) {
        return PathBuf::from(root);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../standards")
}

/// SDTM-IG v3.4 directory path.
pub fn sdtm_ig_path() -> PathBuf {
    standards_root().join("sdtm/ig/v3.4")
}

/// ADaM-IG v1.3 directory path.
pub fn adam_ig_path() -> PathBuf {
    standards_root().join("adam/ig/v1.3")
}

/// SEND-IG v3.1 directory path.
pub fn send_ig_path() -> PathBuf {
    standards_root().join("send/ig/v3.1")
}

/// Controlled terminology directory for a specific version.
pub fn ct_path(version_dir: &str) -> PathBuf {
    standards_root().join("terminology").join(version_dir)
}

/// Validation rules directory for a specific standard.
pub fn validation_rules_path(standard: &str) -> PathBuf {
    standards_root().join("validation").join(standard)
}
