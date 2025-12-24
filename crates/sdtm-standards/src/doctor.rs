#![deny(unsafe_code)]

use crate::manifest::{ManifestFile, Pins, Policy};
use crate::registry::{Conflict, VerifySummary};

#[derive(Debug, Clone, serde::Serialize)]
pub struct DoctorReport {
    pub schema: String,
    pub schema_version: u32,
    pub pins: Pins,
    pub policy: Option<Policy>,
    pub files: Vec<ManifestFile>,
    pub counts: DoctorCounts,
    pub conflicts: Vec<Conflict>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DoctorCounts {
    pub files: usize,
    pub sdtm_domains: usize,
    pub sdtmig_domains: usize,
    pub sdtm_variables: usize,
    pub sdtmig_variables: usize,
    pub ct_codelists: usize,
}

impl DoctorReport {
    pub fn from_verify_summary(
        summary: &VerifySummary,
        policy: Option<Policy>,
        files: Vec<ManifestFile>,
        conflicts: Vec<Conflict>,
    ) -> Self {
        Self {
            schema: "cdisc-transpiler.standards-doctor".to_string(),
            schema_version: 1,
            pins: summary.manifest_pins.clone(),
            policy,
            files,
            counts: DoctorCounts {
                files: summary.file_count,
                sdtm_domains: summary.domain_count_sdtm,
                sdtmig_domains: summary.domain_count_sdtmig,
                sdtm_variables: summary.variable_count_sdtm,
                sdtmig_variables: summary.variable_count_sdtmig,
                ct_codelists: summary.codelist_count,
            },
            conflicts,
        }
    }
}
