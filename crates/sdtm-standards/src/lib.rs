//! SDTM standards and controlled terminology loaders.
//!
//! This crate loads SDTM-IG domain definitions, controlled terminology (CT),
//! and Pinnacle 21 validation rules from offline CSV files in the `standards/`
//! directory. All standards are committed to the repository for offline operation.
//!
//! # Standards Directory Structure
//!
//! ```text
//! standards/
//! ├── ct/                  # Controlled Terminology by version
//! │   └── 2024-03-29/      # CT version date
//! │       └── SDTM_CT_*.csv
//! ├── pinnacle21/          # Pinnacle 21 validation rules
//! │   └── Rules.csv        # P21 rule definitions
//! └── sdtmig/v3_4/         # SDTM-IG v3.4
//!     ├── Datasets.csv     # Domain metadata
//!     └── Variables.csv    # Variable definitions
//! ```

mod csv_utils;
pub mod ct_loader;
pub mod loaders;
pub mod p21_loader;

// Shared CSV utilities
pub use csv_utils::{STANDARDS_ENV_VAR, default_standards_root, read_csv_rows};

// CT loader (clean model per SDTM_CT_relationships.md)
pub use ct_loader::{load_ct_catalog, load_ct_registry, load_default_ct_registry};

// Domain/Dataset loaders
pub use loaders::{load_default_sdtm_ig_domains, load_sdtm_ig_domains};

// P21 rules loader
pub use p21_loader::{load_default_p21_rules, load_p21_rules};

// Re-export P21 types for convenience
pub use sdtm_validate::P21Category;
