//! SDTM data transformation utilities.
//!
//! This crate provides transformation logic extracted from sdtm-core:
//!
//! - **datetime**: ISO 8601 date/time parsing and validation per SDTMIG v3.4
//!
//! # Architecture
//!
//! This crate sits between sdtm-model (pure types) and sdtm-core (orchestration),
//! providing reusable transformation logic that can be used independently.

// TODO(docs): Add documentation for remaining public items (Phase 4 - PR-028)
#![allow(missing_docs)]

pub mod datetime;
