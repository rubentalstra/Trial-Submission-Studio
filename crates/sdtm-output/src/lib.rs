//! SDTM report generation library.
//!
//! This crate provides output generation for SDTM data in multiple formats:
//!
//! - **XPT** (SAS Transport): Standard FDA submission format
//! - **Dataset-XML**: CDISC Dataset-XML format for data exchange
//! - **Define-XML**: CDISC Define-XML for metadata documentation
//! - **SAS Programs**: Transformation code for SAS processing

mod common;
mod dataset_xml;
mod define_xml;
pub mod sas;
mod xpt;

// Re-export public types and functions
pub use dataset_xml::{DatasetXmlOptions, write_dataset_xml, write_dataset_xml_outputs};
pub use define_xml::{DefineXmlOptions, write_define_xml};
pub use sas::{SasProgramOptions, generate_sas_program, write_sas_outputs};
pub use xpt::{build_xpt_dataset_with_name, write_xpt_outputs};

// Re-export common utilities for external use
pub use common::{
    SAS_NUMERIC_LEN, VariableTypeExt, dataset_name, has_collected_data, is_expected, is_identifier,
    is_reference_domain, is_required, normalize_study_id, should_upcase, variable_length,
};
