//! Business logic services
//!
//! Services encapsulate operations that interact with the core crates.

mod mapping;
mod study_loader;

pub use mapping::{MappingService, MappingState, VariableStatus, VariableStatusIcon};
pub use study_loader::StudyLoader;
