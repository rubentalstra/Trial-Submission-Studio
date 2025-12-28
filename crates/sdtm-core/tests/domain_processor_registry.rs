use sdtm_core::domain_processors::{
    DomainProcessor, DomainProcessorRegistry, build_default_registry, process_domain_with_registry,
};
use sdtm_core::processing_context::ProcessingContext;

use anyhow::Result;
use polars::prelude::*;
use sdtm_model::Domain;
use std::sync::Arc;

#[test]
fn default_registry_has_standard_processors() {
    let registry = build_default_registry();
    let domains = registry.registered_domains();

    // Check that standard processors are registered
    assert!(domains.contains(&"AE".to_string()));
    assert!(domains.contains(&"DM".to_string()));
    assert!(domains.contains(&"LB".to_string()));
    assert!(domains.contains(&"VS".to_string()));
    assert!(domains.contains(&"CM".to_string()));
    assert!(domains.contains(&"EX".to_string()));
    assert!(domains.contains(&"MH".to_string()));
    assert!(domains.contains(&"TS".to_string()));

    // Should have 17 standard processors
    assert_eq!(domains.len(), 17);
}

#[test]
fn registry_returns_processor_for_registered_domain() {
    let registry = build_default_registry();

    // Should find processor for registered domain
    assert!(registry.get("AE").is_some());
    assert!(registry.get("ae").is_some()); // case-insensitive
    assert!(registry.get("DM").is_some());
}

#[test]
fn registry_returns_default_for_unregistered_domain() {
    let registry = build_default_registry();

    // Should return default processor for unknown domain
    let processor = registry.get("UNKNOWN");
    assert!(processor.is_some());
}

#[test]
fn registry_respects_disabled_domains() {
    let mut registry = build_default_registry();

    // Disable AE processor
    registry.disable("AE");

    // Should not return processor for disabled domain
    assert!(registry.get("AE").is_none());
    assert!(registry.is_disabled("AE"));

    // Other domains still work
    assert!(registry.get("DM").is_some());

    // Can re-enable
    registry.enable("AE");
    assert!(registry.get("AE").is_some());
    assert!(!registry.is_disabled("AE"));
}

/// Custom processor for testing
struct CustomTestProcessor;

impl DomainProcessor for CustomTestProcessor {
    fn process(
        &self,
        _domain: &Domain,
        df: &mut DataFrame,
        _ctx: &ProcessingContext,
    ) -> Result<()> {
        // Add a marker column to prove this processor ran
        let marker = Series::new("CUSTOM_MARKER".into(), vec!["processed"; df.height()]);
        df.with_column(marker)?;
        Ok(())
    }

    fn domain_code(&self) -> &str {
        "CUSTOM"
    }
}

#[test]
fn registry_accepts_custom_processors() {
    let mut registry = DomainProcessorRegistry::new();
    registry.register(Arc::new(CustomTestProcessor));

    assert!(registry.get("CUSTOM").is_some());
    assert!(registry.get("custom").is_some()); // case-insensitive
}

#[test]
fn process_with_registry_uses_correct_processor() {
    let mut registry = DomainProcessorRegistry::new();
    registry.register(Arc::new(CustomTestProcessor));

    let domain = Domain {
        code: "CUSTOM".to_string(),
        label: None,
        description: None,
        class_name: None,
        dataset_class: None,
        dataset_name: None,
        structure: None,
        variables: vec![],
    };

    let mut df = df! {
        "COL1" => ["a", "b", "c"]
    }
    .unwrap();

    let ctx = ProcessingContext::new("STUDY01");

    process_domain_with_registry(&domain, &mut df, &ctx, &registry).unwrap();

    // Custom processor should have added marker column
    assert!(df.column("CUSTOM_MARKER").is_ok());
}

#[test]
fn process_with_registry_skips_disabled_domain() {
    let mut registry = DomainProcessorRegistry::new();
    registry.register(Arc::new(CustomTestProcessor));
    registry.disable("CUSTOM");

    let domain = Domain {
        code: "CUSTOM".to_string(),
        label: None,
        description: None,
        class_name: None,
        dataset_class: None,
        dataset_name: None,
        structure: None,
        variables: vec![],
    };

    let mut df = df! {
        "COL1" => ["a", "b", "c"]
    }
    .unwrap();

    let ctx = ProcessingContext::new("STUDY01");

    process_domain_with_registry(&domain, &mut df, &ctx, &registry).unwrap();

    // Custom processor should NOT have run (domain disabled)
    assert!(df.column("CUSTOM_MARKER").is_err());
}
