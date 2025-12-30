//! Business logic services
//!
//! Services encapsulate operations that interact with the core crates.

mod mapping;
mod processing;
mod study_loader;

#[allow(unused_imports)]
pub use mapping::{CodelistDisplayInfo, MappingService, MappingState, VariableMappingStatus};
// Processing exports will be used when Transform tab is implemented
#[allow(unused_imports)]
pub use processing::{ProcessingService, TransformResult};
pub use study_loader::StudyLoader;
