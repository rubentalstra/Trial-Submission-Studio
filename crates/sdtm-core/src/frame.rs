//! Re-export of domain frame types from sdtm-transform.
//!
//! This module provides backward compatibility for internal imports.
//! All frame types are now implemented in the `sdtm-transform` crate.

pub use sdtm_transform::frame::{DomainFrame, DomainFrameMeta};
