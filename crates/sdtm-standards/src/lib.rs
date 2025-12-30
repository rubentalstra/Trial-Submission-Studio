//! SDTM standards and controlled terminology loaders.
//!
//! This crate loads SDTM-IG domain definitions and controlled terminology (CT)
//! from offline CSV files in the `standards/` directory.
//!
//! # Standards Directory Structure
//!
//! ```text
//! standards/
//! ├── ct/                  # Controlled Terminology by version
//! │   ├── 2024-03-29/      # CT version (default)
//! │   └── 2025-09-26/      # CT version (latest)
//! └── sdtmig/v3_4/         # SDTM-IG v3.4
//!     ├── Datasets.csv     # Domain metadata
//!     └── Variables.csv    # Variable definitions
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use sdtm_standards::{ct, sdtm_ig, CtVersion};
//!
//! // Load SDTM-IG domains
//! let domains = sdtm_ig::load()?;
//! let ae = domains.iter().find(|d| d.name == "AE").unwrap();
//!
//! // Load CT with version selection
//! let registry = ct::load(CtVersion::default())?;  // 2024-03-29
//! let registry = ct::load(CtVersion::latest())?;   // 2025-09-26
//! ```

pub mod ct;
pub mod error;
pub mod paths;
pub mod sdtm_ig;

// Re-export main types
pub use ct::CtVersion;
pub use error::{Result, StandardsError};
pub use paths::{STANDARDS_ENV_VAR, standards_root};

// Convenience re-exports for common operations
pub use ct::load as load_ct;
pub use sdtm_ig::load as load_sdtm_ig;

// Compatibility aliases for old API names
/// Load SDTM-IG domains from default location.
///
/// This is an alias for [`sdtm_ig::load()`].
pub fn load_default_sdtm_ig_domains() -> Result<Vec<sdtm_model::Domain>> {
    sdtm_ig::load()
}

/// Load CT registry from default location with default version.
///
/// This is an alias for [`ct::load(CtVersion::default())`].
pub fn load_default_ct_registry() -> Result<sdtm_model::TerminologyRegistry> {
    ct::load(CtVersion::default())
}
