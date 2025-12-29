//! SDTM column mapping engine using fuzzy matching.
//!
//! This crate provides automated mapping between source data columns and SDTM
//! domain variables using a combination of:
//!
//! - **Jaro-Winkler similarity** for name matching
//! - **Synonym matching** from variable metadata
//! - **Label comparison** for semantic similarity
//! - **Token-based overlap** for partial matches
//! - **Hint-based adjustments** for data type and pattern matching
//!
//! # Scoring Algorithm
//!
//! The mapping engine calculates confidence scores (0.0 to 1.0+) using:
//!
//! 1. **Base score**: Jaro-Winkler similarity between column name and variable name
//! 2. **Synonym boost** (+15%): Applied when column matches a variable's known synonyms
//! 3. **Label boost** (+10%): Applied when column label closely matches variable label
//! 4. **Penalties**: Applied for type mismatches, missing codes, or low-quality data
//!
//! Scores above 1.0 are possible due to multiplicative boosts.
//!
//! # Confidence Levels
//!
//! Mappings are categorized into confidence levels:
//!
//! - **High** (≥0.95): Excellent match, likely correct
//! - **Medium** (≥0.80): Good match, should be reviewed
//! - **Low** (≥0.60): Weak match, needs manual verification
//! - **Rejected** (<0.60): Below threshold, not included in results
//!
//! # Example
//!
//! ```ignore
//! use sdtm_map::{MappingEngine, ConfidenceLevel};
//! use std::collections::BTreeMap;
//!
//! let engine = MappingEngine::new(domain, 0.6, BTreeMap::new());
//! let result = engine.suggest(&columns);
//!
//! // Get only high-confidence mappings
//! let high = result.filter_by_level(ConfidenceLevel::High);
//! ```

pub mod engine;
pub mod patterns;
pub mod repository;
pub mod utils;

pub use engine::{ConfidenceLevel, ConfidenceThresholds, MappingEngine, MappingResult};
pub use patterns::{build_synonym_map, build_variable_patterns, match_synonyms};
pub use repository::{
    MappingConfigLoader, MappingMetadata, MappingRepository, StoredMappingConfig,
};
pub use utils::{merge_mapping_configs, merge_mappings};
