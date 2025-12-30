//! Business logic services
//!
//! Services encapsulate operations that interact with the core crates.

mod mapping;
mod processing;
mod study_loader;

pub use mapping::{MappingService, MappingState, MappingSummary};
pub use processing::{ProcessingService, TransformResult};
pub use study_loader::StudyLoader;
