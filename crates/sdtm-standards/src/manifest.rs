#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub manifest: ManifestHeader,
    #[serde(default)]
    pub notes: Option<ManifestNotes>,
    pub pins: Pins,
    #[serde(default)]
    pub policy: Option<Policy>,
    pub files: Vec<ManifestFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestHeader {
    pub schema: String,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestNotes {
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pins {
    pub sdtm: String,
    pub sdtmig: String,
    pub ct: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub precedence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFile {
    pub path: String,
    pub sha256: String,
    pub kind: String,
    pub role: String,
    #[serde(default)]
    pub notes: Option<String>,
}
