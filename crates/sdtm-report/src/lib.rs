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
mod sas;
mod xpt;

// Re-export public types and functions
pub use dataset_xml::{write_dataset_xml, write_dataset_xml_outputs, DatasetXmlOptions};
pub use define_xml::{write_define_xml, DefineXmlOptions};
pub use sas::{generate_sas_program, write_sas_outputs, SasProgramOptions};
pub use xpt::{build_xpt_dataset_with_name, write_xpt_outputs};
