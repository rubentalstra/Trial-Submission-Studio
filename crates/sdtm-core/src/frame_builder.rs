//! DataFrame construction from CSV tables and mappings.
//!
//! Provides functions to build SDTM domain DataFrames from source CSV tables,
//! handling column mapping, type conversion, and wide-format transformations.
//!
//! # Key Functions
//!
//! - [`build_domain_frame`]: Simple frame construction without mapping
//! - [`build_domain_frame_with_mapping`]: Frame construction with column mapping
//! - [`build_mapped_domain_frame`]: Auto-mapped frame with wide-format detection

use std::collections::BTreeSet;

use anyhow::Result;

use sdtm_ingest::build_column_hints;
use sdtm_ingest::CsvTable;
use sdtm_map::MappingEngine;
use sdtm_model::{Domain, MappingConfig};

use crate::frame::DomainFrame;
use crate::wide::{build_ie_wide_frame, build_lb_wide_frame, build_vs_wide_frame};

// Re-export from sdtm-transform
pub use sdtm_transform::frame_builder::{
    build_domain_frame, build_domain_frame_with_mapping, collect_table_columns,
};
pub(crate) use sdtm_transform::frame_builder::build_domain_frame_from_records;

/// Build a domain frame with automatic column mapping and wide-format detection.
///
/// For LB, VS, and IE domains, attempts wide-format transformation first.
/// Otherwise uses the mapping engine to suggest column mappings.
///
/// # Returns
///
/// A tuple of (mapping config, domain frame, set of used source columns).
pub fn build_mapped_domain_frame(
    table: &CsvTable,
    domain: &Domain,
    study_id: &str,
) -> Result<(MappingConfig, DomainFrame, BTreeSet<String>)> {
    let domain_code = domain.code.to_uppercase();
    let wide_result = match domain_code.as_str() {
        "LB" => build_lb_wide_frame(table, domain, study_id)?,
        "VS" => build_vs_wide_frame(table, domain, study_id)?,
        "IE" => build_ie_wide_frame(table, domain, study_id)?,
        _ => None,
    };
    if let Some((mapping, frame, used_columns)) = wide_result {
        return Ok((mapping, frame, used_columns));
    }

    let hints = build_column_hints(table);
    let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
    let mapping_result = engine.suggest(&table.headers);
    let mapping = engine.to_config(study_id, mapping_result);
    let frame = build_domain_frame_with_mapping(table, domain, Some(&mapping))?;
    let used_columns = mapping
        .mappings
        .iter()
        .map(|item| item.source_column.clone())
        .collect::<BTreeSet<String>>();
    Ok((mapping, frame, used_columns))
}
