//! P21 validation rules loaded from CSV.
//!
//! The rules module provides access to Pinnacle 21 validation rules
//! loaded from `standards/pinnacle21/Rules.csv`.

mod category;
mod loader;
mod registry;

pub use category::Category;
pub use loader::{load_default_rules, load_rules, LoadError};
pub use registry::{Rule, RuleRegistry};
