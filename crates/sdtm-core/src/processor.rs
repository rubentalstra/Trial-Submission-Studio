//! Domain processing and transformation engine.
//!
//! This module provides the main [`process_domain`] function that applies
//! SDTM-specific transformations to domain DataFrames. Processing includes:
//!
//! - USUBJID prefix application (STUDYID-SUBJID format)
//! - Domain-specific business rules (via domain_processors)
//! - Controlled Terminology normalization
//! - Sequence number (--SEQ) assignment
//!
//! # SDTMIG v3.4 Reference
//!
//! - Section 4.1.2: USUBJID construction (STUDYID concatenation)
//! - Section 4.1.5: --SEQ variable assignment per subject
//! - Chapter 6: Domain-specific processing rules
//! - Chapter 10: Controlled Terminology conformance

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::*;
use tracing::warn;

use sdtm_model::{CaseInsensitiveSet, Domain, VariableType};

use crate::domain_processors;
use crate::pipeline_context::{PipelineContext, SequenceAssignmentMode, UsubjidPrefixMode};
use polars::lazy::dsl::{cols, int_range};
use sdtm_normalization::data_utils::{column_trimmed_values, strip_all_quotes};
use sdtm_normalization::normalization::ct::normalize_ct_value;

/// Input for domain processing operations.
///
/// Bundles together the domain definition, mutable DataFrame reference,
/// pipeline context, and optional sequence tracker for cross-file continuity.
pub struct DomainProcessInput<'a> {
    /// The SDTM domain definition with variable specifications.
    pub domain: &'a Domain,
    /// The DataFrame to process (modified in place).
    pub data: &'a mut DataFrame,
    /// Pipeline context with study metadata, CT registry, and options.
    pub context: &'a PipelineContext,
    /// Optional tracker for cross-file sequence number continuity.
    pub sequence_tracker: Option<&'a mut BTreeMap<String, i64>>,
}

/// Normalize controlled terminology values in a DataFrame.
///
/// This function iterates through columns with CT constraints and normalizes
/// values to their preferred terms.
///
/// When `ct_matching` is `Lenient`, compact-key matching is allowed. When
/// `ct_matching` is `Strict`, only exact matches and defined synonyms
/// are normalized.
fn normalize_ct_columns(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if context.ct_registry.catalogs.is_empty() {
        return Ok(());
    }
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());
    let mut expressions = Vec::new();

    for variable in &domain.variables {
        if !matches!(variable.data_type, VariableType::Char) {
            continue;
        }
        let Some(ct) = context.resolve_ct(domain, &variable.name) else {
            continue;
        };
        let Some(column_name) = column_lookup.get(&variable.name) else {
            continue;
        };
        if df.column(column_name).is_err() {
            continue;
        }

        let ct_clone = ct.clone();
        let options = context.options.normalization.clone();

        let expr = col(column_name)
            .map(
                move |c: Column| {
                    let ca = c.str()?;
                    let out: StringChunked = ca.apply_values(|s| {
                        if s.trim().is_empty() {
                            std::borrow::Cow::Borrowed("")
                        } else {
                            std::borrow::Cow::Owned(normalize_ct_value(&ct_clone, s, &options))
                        }
                    });
                    Ok(out.into_column())
                },
                |_, field| Ok(Field::new(field.name().clone(), DataType::String)),
            )
            .alias(column_name);
        expressions.push(expr);
    }

    if !expressions.is_empty() {
        let new_df = df.clone().lazy().with_columns(expressions).collect()?;
        *df = new_df;
    }
    Ok(())
}

fn apply_base_rules(domain: &Domain, df: &mut DataFrame, context: &PipelineContext) -> Result<()> {
    if matches!(context.options.usubjid_prefix, UsubjidPrefixMode::Skip) {
        return Ok(());
    }
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());
    let Some(usubjid_col) = domain
        .column_name("USUBJID")
        .and_then(|name| column_lookup.get(name))
    else {
        return Ok(());
    };
    let study_col = domain
        .column_name("STUDYID")
        .and_then(|name| column_lookup.get(name));

    // Use direct ChunkedArray iteration for speed
    let usubjid_ca = df.column(usubjid_col)?.str()?;
    let study_ca = if let Some(name) = study_col {
        df.column(name)?.str().ok()
    } else {
        None
    };

    let mut updated_builder =
        polars::prelude::StringChunkedBuilder::new(usubjid_col.into(), df.height());
    let mut changed = false;

    for (idx, opt_u) in usubjid_ca.into_iter().enumerate() {
        let raw_usubjid = opt_u.unwrap_or("");
        let mut usubjid = strip_all_quotes(raw_usubjid);
        let study_val = if let Some(ca) = study_ca {
            ca.get(idx).unwrap_or("")
        } else {
            context.study_id.as_str()
        };
        let study_val = strip_all_quotes(study_val);

        if !study_val.is_empty() && !usubjid.is_empty() {
            let prefix = format!("{study_val}-");
            if !usubjid.starts_with(&prefix) {
                usubjid = format!("{prefix}{usubjid}");
            }
        }
        if usubjid != raw_usubjid {
            changed = true;
        }
        updated_builder.append_value(usubjid);
    }

    if changed {
        let new_series = updated_builder.finish().into_series();
        df.with_column(new_series)?;
        if context.options.warn_on_rewrite {
            warn!(
                domain = %domain.code,
                "USUBJID values updated with study prefix"
            );
        }
    }
    Ok(())
}

/// Process a domain DataFrame through the SDTM transformation pipeline.
///
/// Applies the following transformations in order:
///
/// 1. **Base rules** - USUBJID prefix application (STUDYID-SUBJID format)
/// 2. **Domain-specific rules** - Business logic per domain (AE, DM, CM, etc.)
/// 3. **CT normalization** - Normalize values to CDISC Controlled Terminology
/// 4. **Sequence assignment** - Generate --SEQ values per subject
///
/// # Arguments
///
/// * `input` - Processing input containing domain, data, context, and tracker
///
/// # Errors
///
/// Returns an error if any processing step fails.
pub fn process_domain(input: DomainProcessInput<'_>) -> Result<()> {
    let DomainProcessInput {
        domain,
        data,
        context,
        sequence_tracker,
    } = input;
    apply_base_rules(domain, data, context)?;
    domain_processors::process_domain(domain, data, context)?;
    normalize_ct_columns(domain, data, context)?;
    assign_sequence(domain, data, context, sequence_tracker)?;
    Ok(())
}

/// Assign --SEQ values based on USUBJID grouping.
///
/// Uses tracker if provided for cross-file sequence continuity.
/// Skips assignment if `context.options.sequence_assignment` is `Skip`.
fn assign_sequence(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
    sequence_tracker: Option<&mut BTreeMap<String, i64>>,
) -> Result<()> {
    if matches!(
        context.options.sequence_assignment,
        SequenceAssignmentMode::Skip
    ) {
        return Ok(());
    }
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());
    let Some(seq_col) = domain.infer_seq_column() else {
        return Ok(());
    };
    let Some(usubjid_col) = domain.column_name("USUBJID") else {
        return Ok(());
    };
    let seq_col_name = column_lookup.get(seq_col).unwrap_or(seq_col);
    let usubjid_col_name = column_lookup.get(usubjid_col).unwrap_or(usubjid_col);
    if !needs_sequence_assignment(df, seq_col_name, usubjid_col_name)? {
        return Ok(());
    }
    assign_sequence_values(
        domain,
        df,
        seq_col_name,
        usubjid_col_name,
        sequence_tracker,
        context,
    )?;
    Ok(())
}

fn assign_sequence_values(
    domain: &Domain,
    df: &mut DataFrame,
    seq_column: &str,
    group_column: &str,
    tracker: Option<&mut BTreeMap<String, i64>>,
    context: &PipelineContext,
) -> Result<()> {
    let is_tracked = tracker.is_some();
    if df.height() == 0 {
        return Ok(());
    }

    // Calculate local sequence (1-based index within group)
    // We use lazy execution for optimization
    let lazy = df.clone().lazy();

    // If we have a tracker, we need to incorporate offsets
    let (new_df, max_updates) = if let Some(tracker_map) = &tracker {
        // 1. Create DataFrame from tracker
        let keys: Vec<String> = tracker_map.keys().cloned().collect();
        let offsets: Vec<i64> = tracker_map.values().cloned().collect();
        let tracker_df = DataFrame::new(vec![
            Series::new(group_column.into(), keys).into(),
            Series::new("offset".into(), offsets).into(),
        ])?;

        // 2. Join and calculate
        // We use left join to keep all rows
        let joined = lazy.join(
            tracker_df.lazy(),
            [col(group_column)],
            [col(group_column)],
            JoinArgs::new(JoinType::Left),
        );

        // 3. Calculate SEQ = offset + cumcount + 1
        // cumcount().over(group) gives 0-based index within group
        let seq_expr = col("offset").fill_null(lit(0))
            + int_range(lit(0), col(group_column).len(), 1, DataType::Int64)
                .over([col(group_column)])
            + lit(1);

        let res_df = joined
            .with_column(seq_expr.cast(DataType::Float64).alias(seq_column))
            .drop(cols(["offset"]))
            .collect()?;

        // 4. Calculate updates for tracker
        let updates = res_df
            .clone()
            .lazy()
            .group_by([col(group_column)])
            .agg([col(seq_column).max().alias("max_seq")])
            .collect()?;

        (res_df, Some(updates))
    } else {
        // Simple case: just cumcount + 1
        let seq_expr = int_range(lit(0), col(group_column).len(), 1, DataType::Int64)
            .over([col(group_column)])
            + lit(1);
        let res_df = lazy
            .with_column(seq_expr.cast(DataType::Float64).alias(seq_column))
            .collect()?;
        (res_df, None)
    };

    *df = new_df;

    // Update tracker if needed
    if let Some(updates_df) = max_updates {
        if let Some(tracker_map) = tracker {
            let groups = updates_df
                .column(group_column)?
                .as_materialized_series()
                .str()?;
            let maxes = updates_df
                .column("max_seq")?
                .as_materialized_series()
                .f64()?;

            for (opt_g, opt_m) in groups.into_iter().zip(maxes.into_iter()) {
                if let (Some(g), Some(m)) = (opt_g, opt_m) {
                    tracker_map.insert(g.to_string(), m as i64);
                }
            }
        }
    }

    if context.options.warn_on_rewrite {
        let message = if is_tracked {
            "Sequence values recalculated with tracker"
        } else {
            "Sequence values recalculated"
        };
        warn!(
            domain = %domain.code,
            sequence = %seq_column,
            "{message}"
        );
    }

    Ok(())
}

fn needs_sequence_assignment(df: &DataFrame, seq_column: &str, group_column: &str) -> Result<bool> {
    let Some(seq_values) = column_trimmed_values(df, seq_column) else {
        return Ok(true);
    };
    let Some(group_values) = column_trimmed_values(df, group_column) else {
        return Ok(true);
    };
    let mut groups: BTreeMap<String, BTreeSet<i64>> = BTreeMap::new();
    let mut has_value = false;
    for (group, value_str) in group_values.iter().zip(seq_values.iter()) {
        let group = group.trim().to_string();
        if group.is_empty() {
            continue;
        }
        if value_str.is_empty() {
            continue;
        }
        if let Some(parsed) = parse_sequence_value(value_str) {
            has_value = true;
            let entry = groups.entry(group).or_default();
            if entry.contains(&parsed) {
                return Ok(true);
            }
            entry.insert(parsed);
        } else {
            return Ok(true);
        }
    }
    Ok(!has_value)
}

fn parse_sequence_value(text: &str) -> Option<i64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(value) = trimmed.parse::<i64>() {
        return Some(value);
    }
    if let Ok(value) = trimmed.parse::<f64>()
        && value.is_finite()
    {
        let rounded = value.round();
        if (value - rounded).abs() <= f64::EPSILON && rounded >= 0.0 && rounded <= i64::MAX as f64 {
            return Some(rounded as i64);
        }
    }
    None
}
