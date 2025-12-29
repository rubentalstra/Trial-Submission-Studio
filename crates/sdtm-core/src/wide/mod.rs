//! Re-export of wide format processing from sdtm-transform.
//!
//! This module provides backward compatibility for internal imports.
//! All wide format functionality is now implemented in the `sdtm-transform` crate.

pub use sdtm_transform::wide::{build_ie_wide_frame, build_lb_wide_frame, build_vs_wide_frame};
