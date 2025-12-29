//! SDTM data transformation utilities.
//!
//! This crate provides transformation logic extracted from sdtm-core:
//!
//! - **datetime**: ISO 8601 date/time parsing and validation per SDTMIG v3.4
//! - **frame**: Domain frame types for SDTM datasets
//! - **frame_builder**: DataFrame construction utilities
//! - **data_utils**: DataFrame value extraction and SDTM identifier sanitization
//! - **domain_sets**: Domain collection utilities for lookup and reporting
//! - **wide**: Wide-to-long format conversion for LB, VS, IE domains
//! - **suppqual**: Supplemental Qualifier (SUPP--) dataset generation
//! - **relationships**: Relationship dataset generation (RELREC, RELSPEC, RELSUB)
//!
//! # Architecture
//!
//! This crate sits between sdtm-model (pure types) and sdtm-core (orchestration),
//! providing reusable transformation logic that can be used independently.

// TODO(docs): Add documentation for remaining public items (Phase 4 - PR-028)
#![allow(missing_docs)]

pub mod data_utils;
pub mod datetime;
pub mod domain_sets;
pub mod frame;
pub mod frame_builder;
pub mod relationships;
pub mod suppqual;
pub mod wide;
