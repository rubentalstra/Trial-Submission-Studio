//! CDISC standards and controlled terminology loaders.
//!
//! This crate loads CDISC Implementation Guide definitions and Controlled
//! Terminology (CT) from offline CSV files in the `standards/` directory.
//!
//! # Supported Standards
//!
//! - **SDTM-IG v3.4**: Clinical trial tabulation domains
//! - **ADaM-IG v1.3**: Analysis-ready datasets
//! - **SEND-IG v3.1.1**: Nonclinical study domains
//!
//! # Standards Directory Structure
//!
//! ```text
//! standards/
//! ├── Terminology/             # Controlled Terminology by version
//! │   ├── 2024-03-29/          # CT version (default)
//! │   └── 2025-09-26/          # CT version (latest)
//! ├── sdtm/ig/v3.4/            # SDTM-IG v3.4
//! │   ├── Datasets.csv
//! │   └── Variables.csv
//! ├── adam/ig/v1.3/            # ADaM-IG v1.3
//! │   ├── DataStructures.csv
//! │   └── Variables.csv
//! └── send/ig/v3.1.1/            # SEND-IG v3.1.1
//!     ├── Datasets.csv
//!     └── Variables.csv
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use cdisc_standards::{StandardsRegistry, StandardsConfig};
//!
//! // Load all standards
//! let registry = StandardsRegistry::load_all()?;
//!
//! // Or load specific standards
//! let registry = StandardsRegistry::load_sdtm_only()?;
//!
//! // Access domains
//! let ae = registry.find_sdtm_domain("AE").unwrap();
//! println!("AE has {} variables", ae.variables.len());
//! ```

pub mod adam_ig;
pub mod ct;
pub mod error;
pub mod paths;
pub mod registry;
pub mod sdtm_ig;
pub mod send_ig;

// Re-export main types
pub use ct::CtVersion;
pub use error::{Result, StandardsError};
pub use paths::{STANDARDS_ENV_VAR, standards_root};
pub use registry::{StandardsConfig, StandardsRegistry};

// Convenience re-exports for common operations
pub use adam_ig::load as load_adam_ig;
pub use ct::load as load_ct;
pub use sdtm_ig::load as load_sdtm_ig;
pub use send_ig::load as load_send_ig;
