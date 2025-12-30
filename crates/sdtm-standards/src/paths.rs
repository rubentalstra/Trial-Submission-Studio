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
/// let root = sdtm_standards::standards_root();
/// let sdtm_ig_dir = root.join("sdtmig/v3_4");
/// ```
pub fn standards_root() -> PathBuf {
    if let Ok(root) = std::env::var(STANDARDS_ENV_VAR) {
        return PathBuf::from(root);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../standards")
}

/// SDTM-IG v3.4 directory path.
pub fn sdtm_ig_path() -> PathBuf {
    standards_root().join("sdtmig/v3_4")
}

/// Controlled terminology directory for a specific version.
pub fn ct_path(version_dir: &str) -> PathBuf {
    standards_root().join("ct").join(version_dir)
}
