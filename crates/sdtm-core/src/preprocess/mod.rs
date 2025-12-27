//! Domain-specific preprocessing modules.
//!
//! This module provides per-domain preprocessing logic that transforms raw input
//! data into SDTM-compliant structures. Preprocessing runs before domain processing
//! and handles field inference, value normalization, and rule-driven transformations.
//!
//! # Architecture
//!
//! The preprocessing system is driven by a rule table that defines:
//! - Which domains require preprocessing
//! - What rules apply to each domain
//! - Execution order and dependencies
//!
//! # Modules
//!
//! - `common` - Shared utilities for preprocessing operations
//! - `rule_table` - Rule definitions and execution engine
//! - Per-domain modules (da, ds, ex, ie, qs, pe) for domain-specific logic
//!
//! # SDTMIG References
//!
//! Preprocessing implements heuristic inference based on source data patterns.
//! While these derivations help populate common SDTM variables, sponsors should
//! validate inferred values against their study metadata per SDTMIG requirements.

mod common;
mod da;
mod ds;
mod ex;
mod ie;
mod pe;
mod qs;
mod rule_table;

pub use common::{
    PreprocessConfig, PreprocessContext, column_hint_for_domain_table, get_column_value,
    get_column_values, has_column, set_column_values,
};
pub use rule_table::{
    DomainPreprocessor, PreprocessRegistry, PreprocessRule, RuleCategory, RuleExecutor,
    RuleMetadata, build_default_preprocess_registry,
};

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_ingest::CsvTable;
use sdtm_model::{Domain, MappingConfig};

use crate::ProcessingContext;

/// Fill missing test fields based on source data and column hints.
///
/// This function performs heuristic inference to populate missing SDTM variables
/// from source column headers and labels. The inference is gated by the
/// `allow_heuristic_inference` option in `ProcessingOptions`.
///
/// # SDTMIG References
///
/// While this function helps populate common test-related variables, the
/// derivations are based on heuristics rather than explicit SDTMIG rules.
/// Sponsors should validate the inferred values against their study metadata.
///
/// # Domains Handled
///
/// - **QS**: QSTEST, QSTESTCD, QSCAT from ORRES column hints
/// - **PE**: PETEST, PETESTCD from ORRES column hints
/// - **DS**: DSDECOD, DSTERM from CT matches and completion columns
/// - **EX**: EXTRT from treatment-related column hints
/// - **DA**: DAORRES, DATEST, DATESTCD, DAORRESU from column patterns
/// - **IE**: IETEST, IETESTCD, IECAT from column patterns
///
/// # Arguments
///
/// * `domain` - The domain metadata
/// * `mapping` - The mapping configuration used for this domain
/// * `table` - The source CSV table
/// * `df` - The DataFrame to update
/// * `ctx` - The processing context (contains options)
pub fn fill_missing_test_fields(
    domain: &Domain,
    mapping: &MappingConfig,
    table: &CsvTable,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    // Gate heuristic inference behind the option
    // When disabled, only explicit mappings from the mapping config are used
    if !ctx.options.allow_heuristic_inference {
        return Ok(());
    }

    let code = domain.code.to_uppercase();
    let preprocess_ctx = PreprocessContext::new(domain, mapping, table, ctx);

    match code.as_str() {
        "QS" => qs::preprocess_qs(&preprocess_ctx, df),
        "PE" => pe::preprocess_pe(&preprocess_ctx, df),
        "DS" => ds::preprocess_ds(&preprocess_ctx, df),
        "EX" => ex::preprocess_ex(&preprocess_ctx, df),
        "DA" => da::preprocess_da(&preprocess_ctx, df),
        "IE" => ie::preprocess_ie(&preprocess_ctx, df),
        _ => Ok(()),
    }
}

/// Preprocess a domain using the default registry.
///
/// This is the primary entry point for rule-driven preprocessing.
pub fn preprocess_domain(
    domain: &Domain,
    mapping: &MappingConfig,
    table: &CsvTable,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    if !ctx.options.allow_heuristic_inference {
        return Ok(());
    }

    let registry = build_default_preprocess_registry();
    let preprocess_ctx = PreprocessContext::new(domain, mapping, table, ctx);

    registry.process(&domain.code, &preprocess_ctx, df)
}
