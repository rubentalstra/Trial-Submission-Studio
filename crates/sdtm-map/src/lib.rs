pub mod engine;
pub mod patterns;
pub mod repository;
pub mod utils;

pub use engine::{MappingEngine, MappingResult};
pub use patterns::{build_synonym_map, build_variable_patterns, match_synonyms};
pub use repository::{
    MappingConfigLoader, MappingMetadata, MappingRepository, StoredMappingConfig,
};
pub use utils::{merge_mapping_configs, merge_mappings};
