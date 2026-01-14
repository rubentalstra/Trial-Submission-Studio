//! Standards registry for multi-standard support.
//!
//! Provides unified access to SDTM, ADaM, and SEND standards.

use crate::adam::AdamDataset;
use crate::ct::{CtVersion, TerminologyRegistry};
use crate::sdtm::SdtmDomain;
use crate::send::SendDomain;
use crate::traits::Standard;

use crate::error::Result;
use crate::{adam_ig, ct, sdtm_ig, send_ig};

/// Configuration for loading standards.
#[derive(Debug, Clone)]
pub struct StandardsConfig {
    /// CT version to load.
    pub ct_version: CtVersion,
    /// Whether to load SDTM-IG.
    pub load_sdtm: bool,
    /// Whether to load ADaM-IG.
    pub load_adam: bool,
    /// Whether to load SEND-IG.
    pub load_send: bool,
}

impl Default for StandardsConfig {
    fn default() -> Self {
        Self {
            ct_version: CtVersion::default(),
            load_sdtm: true,
            load_adam: true,
            load_send: true,
        }
    }
}

impl StandardsConfig {
    /// Create config for SDTM only.
    pub fn sdtm_only() -> Self {
        Self {
            load_sdtm: true,
            load_adam: false,
            load_send: false,
            ..Default::default()
        }
    }

    /// Create config for ADaM workflow (requires SDTM as source + ADaM as target).
    pub fn adam_workflow() -> Self {
        Self {
            load_sdtm: true, // Source data
            load_adam: true, // Target structure
            load_send: false,
            ..Default::default()
        }
    }

    /// Create config for SEND only.
    pub fn send_only() -> Self {
        Self {
            load_sdtm: false,
            load_adam: false,
            load_send: true,
            ..Default::default()
        }
    }
}

/// Unified registry of all loaded CDISC standards.
#[derive(Debug)]
pub struct StandardsRegistry {
    /// Controlled Terminology.
    pub ct: TerminologyRegistry,
    /// SDTM domains (if loaded).
    pub sdtm_domains: Vec<SdtmDomain>,
    /// ADaM datasets (if loaded).
    pub adam_datasets: Vec<AdamDataset>,
    /// SEND domains (if loaded).
    pub send_domains: Vec<SendDomain>,
}

impl StandardsRegistry {
    /// Load standards with the given configuration.
    pub fn load(config: &StandardsConfig) -> Result<Self> {
        let ct = ct::load(config.ct_version)?;

        let sdtm_domains = if config.load_sdtm {
            sdtm_ig::load()?
        } else {
            Vec::new()
        };

        let adam_datasets = if config.load_adam {
            adam_ig::load()?
        } else {
            Vec::new()
        };

        let send_domains = if config.load_send {
            send_ig::load()?
        } else {
            Vec::new()
        };

        Ok(Self {
            ct,
            sdtm_domains,
            adam_datasets,
            send_domains,
        })
    }

    /// Load all standards with default configuration.
    pub fn load_all() -> Result<Self> {
        Self::load(&StandardsConfig::default())
    }

    /// Load SDTM standards only.
    pub fn load_sdtm_only() -> Result<Self> {
        Self::load(&StandardsConfig::sdtm_only())
    }

    /// Load ADaM workflow standards (SDTM + ADaM).
    pub fn load_adam_workflow() -> Result<Self> {
        Self::load(&StandardsConfig::adam_workflow())
    }

    /// Load SEND standards only.
    pub fn load_send_only() -> Result<Self> {
        Self::load(&StandardsConfig::send_only())
    }

    /// Check if a standard is loaded.
    pub fn has_standard(&self, standard: Standard) -> bool {
        match standard {
            Standard::Sdtm => !self.sdtm_domains.is_empty(),
            Standard::Adam => !self.adam_datasets.is_empty(),
            Standard::Send => !self.send_domains.is_empty(),
        }
    }

    /// Get available standards.
    pub fn available_standards(&self) -> Vec<Standard> {
        let mut standards = Vec::new();
        if !self.sdtm_domains.is_empty() {
            standards.push(Standard::Sdtm);
        }
        if !self.adam_datasets.is_empty() {
            standards.push(Standard::Adam);
        }
        if !self.send_domains.is_empty() {
            standards.push(Standard::Send);
        }
        standards
    }

    /// Find an SDTM domain by name (case-insensitive).
    pub fn find_sdtm_domain(&self, name: &str) -> Option<&SdtmDomain> {
        self.sdtm_domains
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(name))
    }

    /// Find the SUPPQUAL template domain.
    ///
    /// This is the base domain for creating SUPP domains.
    pub fn find_suppqual_template(&self) -> Option<&SdtmDomain> {
        self.find_sdtm_domain("SUPPQUAL")
    }

    /// Find an ADaM dataset by name (case-insensitive).
    pub fn find_adam_dataset(&self, name: &str) -> Option<&AdamDataset> {
        self.adam_datasets
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(name))
    }

    /// Find a SEND domain by name (case-insensitive).
    pub fn find_send_domain(&self, name: &str) -> Option<&SendDomain> {
        self.send_domains
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all() {
        let registry = StandardsRegistry::load_all().expect("load all standards");
        assert!(registry.has_standard(Standard::Sdtm));
        assert!(registry.has_standard(Standard::Adam));
        assert!(registry.has_standard(Standard::Send));
    }

    #[test]
    fn test_load_sdtm_only() {
        let registry = StandardsRegistry::load_sdtm_only().expect("load SDTM only");
        assert!(registry.has_standard(Standard::Sdtm));
        assert!(!registry.has_standard(Standard::Adam));
        assert!(!registry.has_standard(Standard::Send));
    }

    #[test]
    fn test_load_adam_workflow() {
        let registry = StandardsRegistry::load_adam_workflow().expect("load ADaM workflow");
        assert!(
            registry.has_standard(Standard::Sdtm),
            "ADaM workflow requires SDTM"
        );
        assert!(registry.has_standard(Standard::Adam));
        assert!(!registry.has_standard(Standard::Send));
    }

    #[test]
    fn test_find_domain() {
        let registry = StandardsRegistry::load_sdtm_only().expect("load registry");
        let ae = registry.find_sdtm_domain("AE");
        assert!(ae.is_some(), "Should find AE domain");
    }

    #[test]
    fn test_find_suppqual_template() {
        let registry = StandardsRegistry::load_sdtm_only().expect("load registry");
        let suppqual = registry.find_suppqual_template();
        assert!(suppqual.is_some(), "Should find SUPPQUAL template");
    }
}
