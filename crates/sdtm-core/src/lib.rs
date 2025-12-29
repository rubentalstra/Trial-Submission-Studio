//! Core SDTM processing engine and domain transformation logic.
//!
//! This crate provides the main processing pipeline for transforming source data
//! into SDTM-compliant datasets. It handles domain-specific business rules,
//! controlled terminology normalization, sequence assignment, and relationship
//! dataset generation.
//!
//! # Architecture
//!
//! The processing pipeline consists of several stages:
//!
//! 1. **Frame Building** ([`frame_builder`]) - Converts CSV tables to Polars DataFrames
//! 2. **Domain Processing** ([`processor`]) - Applies domain-specific transformations
//! 3. **CT Normalization** - Normalizes values to CDISC Controlled Terminology
//! 4. **Supplemental Qualifiers** ([`suppqual`]) - Generates SUPP-- datasets
//! 5. **Relationships** ([`relationships`]) - Builds RELREC, RELSPEC, RELSUB datasets
//!
//! # SDTMIG v3.4 Reference
//!
//! This crate implements the SDTM Implementation Guide v3.4 requirements for:
//! - Chapter 4: Domain structure and variable conventions
//! - Chapter 6: Domain-specific processing rules
//! - Chapter 8: Relationship datasets (RELREC, RELSPEC, RELSUB)
//! - Chapter 10: Controlled Terminology conformance

//!
//! # Example
//!
//! ```rust,ignore
//! use sdtm_core::pipeline_context::PipelineContext;
//! use sdtm_core::processor::{process_domain, DomainProcessInput};
//!
//! let context = PipelineContext::new("STUDY01")
//!     .with_standards(domain_definitions)
//!     .with_ct_registry(ct_registry);
//!
//! let input = DomainProcessInput {
//!     domain: &domain,
//!     data: &mut dataframe,
//!     context: &context,
//!     sequence_tracker: None,
//! };
//!
//! process_domain(input)?;
//! ```

// TODO(docs): Add documentation for remaining public items (Phase 4 - PR-028)
#![allow(missing_docs)]
// TODO(clippy): Fix these clippy warnings (Phase 4)
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::wildcard_imports)]

mod ct_utils;
mod data_utils;
mod datetime;
pub mod domain_processors;

// Internal modules - public for sdtm-cli but not part of stable API
#[doc(hidden)]
pub mod domain_sets;
pub mod frame;
#[doc(hidden)]
pub mod frame_builder;

pub mod pipeline_context;
pub mod processor;
pub mod relationships;
pub mod suppqual;
mod wide;
