//! Minimal SDTM column-to-variable mapping for manual workflows.
//!
//! This module provides fuzzy matching and scoring to help users manually map
//! source data columns to SDTM domain variables. It uses Jaro-Winkler similarity
//! as the base algorithm with adjustments for label matching, suffix patterns,
//! and type compatibility.
//!
//! # Design Philosophy
//!
//! - **Simple**: Pure Jaro-Winkler scoring with minimal adjustments
//! - **Explainable**: Score breakdowns show why a match scored as it did
//! - **Session-only**: No persistence, mappings live for the session duration
//! - **Centralized**: GUI calls this module for scoring instead of reimplementing
//!
//! # Example
//!
//! ```ignore
//! use tss_submit::map::{MappingState, VariableStatus};
//! use std::collections::BTreeMap;
//!
//! // Create mapping state with auto-suggestions
//! let mut state = MappingState::new(domain, "STUDY01", &columns, hints, 0.6);
//!
//! // Check status
//! match state.status("USUBJID") {
//!     VariableStatus::Suggested => {
//!         // Accept the suggestion
//!         state.accept_suggestion("USUBJID").unwrap();
//!     }
//!     VariableStatus::Unmapped => {
//!         // Manual mapping
//!         state.accept_manual("USUBJID", "SUBJECT_ID").unwrap();
//!     }
//!     VariableStatus::Accepted => {}
//!     _ => {}
//! }
//!
//! // Use scorer for dropdown sorting
//! let scores = state.scorer().score_all_for_variable("AETERM", &available_cols);
//!
//! // Export final config
//! let config = state.to_config();
//! ```

mod error;
mod score;
mod state;

pub use error::MappingError;
pub use score::{ColumnScore, ScoreComponent, ScoringEngine, Suggestion};
pub use state::{Mapping, MappingConfig, MappingState, MappingSummary, VariableStatus};
