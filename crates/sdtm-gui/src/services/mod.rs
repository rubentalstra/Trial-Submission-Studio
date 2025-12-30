//! Business logic services
//!
//! Services encapsulate operations that interact with the core crates.

mod mapping;
mod study_loader;

pub use mapping::{MappingService, MappingState, VariableMappingStatus};
pub use study_loader::StudyLoader;
