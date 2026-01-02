//! IEEE ↔ IBM floating-point conversion.
//!
//! This module provides conversion between IEEE 754 double-precision
//! floating-point numbers and IBM mainframe floating-point format used
//! in SAS Transport (XPT) files.

mod ibm;

pub use ibm::{encode_missing, expand_ibm, ibm_to_ieee, ieee_to_ibm, is_missing, truncate_ibm};
