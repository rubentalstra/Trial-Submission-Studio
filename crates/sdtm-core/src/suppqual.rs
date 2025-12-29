//! Re-export of SUPPQUAL generation from sdtm-transform.
//!
//! This module provides backward compatibility for internal imports.
//! All SUPPQUAL functionality is now implemented in the `sdtm-transform` crate.

pub use sdtm_transform::suppqual::{build_suppqual, suppqual_dataset_code, SuppqualInput};
