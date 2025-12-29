//! Re-export of domain set utilities from sdtm-transform.
//!
//! This module provides backward compatibility for internal imports.
//! All domain set functionality is now implemented in the `sdtm-transform` crate.

pub use sdtm_transform::domain_sets::{
    build_report_domains, domain_map_by_code, is_supporting_domain,
};
