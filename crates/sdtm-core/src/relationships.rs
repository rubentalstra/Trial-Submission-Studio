//! Re-export of relationship dataset generation from sdtm-transform.
//!
//! This module provides backward compatibility for internal imports.
//! All relationship functionality is now implemented in the `sdtm-transform` crate.

pub use sdtm_transform::relationships::{
    build_relationship_frames, build_relrec, build_relspec, build_relsub, RelationshipConfig,
};
