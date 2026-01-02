//! SEND (Standard for Exchange of Nonclinical Data) types per SENDIG v3.1.
//!
//! This module provides SEND-specific types for representing domains and variables
//! in nonclinical/animal study tabulation datasets.

pub mod domain;
pub mod enums;

pub use domain::{SendDomain, SendVariable};
pub use enums::{SendDatasetClass, SendStudyType};
