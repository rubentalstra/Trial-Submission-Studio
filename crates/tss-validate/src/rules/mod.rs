//! P21 validation rules loaded from CSV.
//!
//! The rules module provides access to Pinnacle 21 validation rules
//! loaded from `standards/validation/sdtm/Rules.csv`.

mod category;
mod loader;
mod registry;

pub use category::Category;
pub use loader::{LoadError, load_default_rules, load_rules};
pub use registry::{Rule, RuleRegistry};
