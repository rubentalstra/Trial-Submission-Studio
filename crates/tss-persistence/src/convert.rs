//! Conversion traits between GUI types and persistence types.
//!
//! These traits define how to convert between runtime state and
//! serializable snapshots.
//!
//! # Design Note
//!
//! The conversion is intentionally one-way for some types:
//! - `MappingState` -> `MappingSnapshot` is straightforward
//! - `MappingSnapshot` -> `MappingState` requires external data (DataFrame, column hints)
//!   and is handled via `MappingState::restore_from_snapshot()`
//!
//! This is because `MappingState` contains a `ScoringEngine` that needs
//! source data to initialize, which isn't stored in the snapshot.

use std::collections::BTreeMap;

use crate::types::{
    MappingEntry, MappingSnapshot, SuppActionSnapshot, SuppColumnSnapshot, SuppOriginSnapshot,
};

/// Trait for types that can be converted to a persistence snapshot.
pub trait ToSnapshot {
    /// The snapshot type.
    type Snapshot;

    /// Convert to a snapshot for persistence.
    fn to_snapshot(&self) -> Self::Snapshot;
}

/// Trait for types that can be created from a persistence snapshot.
///
/// Note: This is only implemented for types that can be fully
/// reconstructed from the snapshot alone. Types that need external
/// data (like MappingState) use a different restoration pattern.
pub trait FromSnapshot: Sized {
    /// The snapshot type.
    type Snapshot;

    /// Create from a snapshot.
    fn from_snapshot(snapshot: Self::Snapshot) -> Self;
}

// =============================================================================
// MAPPING CONVERSIONS
// =============================================================================

/// Create a mapping snapshot from MappingState's data.
///
/// This is called by GUI code that has access to MappingState.
/// We can't implement ToSnapshot directly on MappingState because
/// it's defined in tss-submit.
///
/// Note: The `accepted` parameter includes confidence scores from the session,
/// but we intentionally discard them - confidence is only meaningful during
/// active mapping, not for persistence.
pub fn mapping_to_snapshot(
    study_id: &str,
    accepted: &BTreeMap<String, (String, f32)>,
    not_collected: &BTreeMap<String, String>,
    omitted: &std::collections::BTreeSet<String>,
    auto_generated: &std::collections::BTreeSet<String>,
) -> MappingSnapshot {
    MappingSnapshot {
        study_id: study_id.to_string(),
        accepted: accepted
            .iter()
            .map(|(var, (col, _conf))| (var.clone(), MappingEntry::new(col.clone())))
            .collect(),
        not_collected: not_collected.clone(),
        omitted: omitted.clone(),
        auto_generated: auto_generated.clone(),
    }
}

// =============================================================================
// SUPP CONVERSIONS (GUI types will implement these)
// =============================================================================

/// Convert GUI SuppOrigin to snapshot.
#[derive(Debug, Clone, Copy)]
pub enum SuppOriginConvert {
    Crf,
    Derived,
    Assigned,
}

impl From<SuppOriginConvert> for SuppOriginSnapshot {
    fn from(origin: SuppOriginConvert) -> Self {
        match origin {
            SuppOriginConvert::Crf => SuppOriginSnapshot::Crf,
            SuppOriginConvert::Derived => SuppOriginSnapshot::Derived,
            SuppOriginConvert::Assigned => SuppOriginSnapshot::Assigned,
        }
    }
}

impl From<SuppOriginSnapshot> for SuppOriginConvert {
    fn from(snapshot: SuppOriginSnapshot) -> Self {
        match snapshot {
            SuppOriginSnapshot::Crf => SuppOriginConvert::Crf,
            SuppOriginSnapshot::Derived => SuppOriginConvert::Derived,
            SuppOriginSnapshot::Assigned => SuppOriginConvert::Assigned,
        }
    }
}

/// Convert GUI SuppAction to snapshot.
#[derive(Debug, Clone, Copy)]
pub enum SuppActionConvert {
    Pending,
    Include,
    Skip,
}

impl From<SuppActionConvert> for SuppActionSnapshot {
    fn from(action: SuppActionConvert) -> Self {
        match action {
            SuppActionConvert::Pending => SuppActionSnapshot::Pending,
            SuppActionConvert::Include => SuppActionSnapshot::Include,
            SuppActionConvert::Skip => SuppActionSnapshot::Skip,
        }
    }
}

impl From<SuppActionSnapshot> for SuppActionConvert {
    fn from(snapshot: SuppActionSnapshot) -> Self {
        match snapshot {
            SuppActionSnapshot::Pending => SuppActionConvert::Pending,
            SuppActionSnapshot::Include => SuppActionConvert::Include,
            SuppActionSnapshot::Skip => SuppActionConvert::Skip,
        }
    }
}

/// Helper to create SuppColumnSnapshot from component parts.
pub fn supp_column_to_snapshot(
    column: &str,
    qnam: &str,
    qlabel: &str,
    qorig: SuppOriginConvert,
    qeval: Option<&str>,
    action: SuppActionConvert,
) -> SuppColumnSnapshot {
    SuppColumnSnapshot {
        column: column.to_string(),
        qnam: qnam.to_string(),
        qlabel: qlabel.to_string(),
        qorig: qorig.into(),
        qeval: qeval.map(String::from),
        action: action.into(),
    }
}
