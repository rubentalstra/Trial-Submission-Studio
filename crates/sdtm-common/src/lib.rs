//! Shared utilities for SDTM crates.
//!
//! This crate provides common utilities used across the SDTM workspace,
//! including Polars DataFrame helpers.

pub mod polars;

// Re-export commonly used functions at crate root for convenience
pub use polars::{
    any_to_f64, any_to_i64, any_to_string, any_to_string_non_empty, format_numeric, parse_f64,
    parse_i64,
};
