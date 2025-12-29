//! Domain processor trait and registry.
//!
//! This module provides a trait-based abstraction for domain processors,
//! enabling plugin-style registration and easier testing.
//!
//! # Architecture
//!
//! The [`DomainProcessor`] trait defines a common interface for all domain
//! processors. Each processor is registered in the [`ProcessorRegistry`]
//! which provides lookup by domain code.
//!
//! # Example
//!
//! ```ignore
//! use sdtm_core::domain_processors::{DomainProcessor, ProcessorRegistry};
//!
//! let registry = ProcessorRegistry::default();
//! if let Some(processor) = registry.get("DM") {
//!     processor.process(domain, df, context)?;
//! }
//! ```

use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

/// Trait for domain-specific processing logic.
///
/// Implementors of this trait provide domain-specific transformations for
/// SDTM datasets, including:
///
/// - Controlled terminology normalization
/// - Variable derivations (e.g., --DY from --DTC)
/// - Result standardization (ORRES → STRESC, STRESN)
/// - Date validation and formatting
///
/// # Implementing a Processor
///
/// Each domain processor should:
/// 1. Implement this trait for a unit struct
/// 2. Register the processor in [`ProcessorRegistry::default()`]
///
/// # Example
///
/// ```ignore
/// struct DMProcessor;
///
/// impl DomainProcessor for DMProcessor {
///     fn domain_code(&self) -> &'static str {
///         "DM"
///     }
///
///     fn process(
///         &self,
///         domain: &Domain,
///         df: &mut DataFrame,
///         context: &PipelineContext,
///     ) -> Result<()> {
///         // DM-specific processing logic
///         Ok(())
///     }
/// }
/// ```
pub trait DomainProcessor: Send + Sync {
    /// Returns the domain code this processor handles (e.g., "DM", "AE", "LB").
    ///
    /// The code should be uppercase and match the SDTM domain abbreviation.
    fn domain_code(&self) -> &'static str;

    /// Returns a human-readable description of the processor.
    fn description(&self) -> &'static str {
        "Domain processor"
    }

    /// Process a domain DataFrame according to SDTM-IG rules.
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain metadata (variables, labels, etc.)
    /// * `df` - Mutable DataFrame to transform in-place
    /// * `context` - Pipeline context with study metadata and CT registry
    ///
    /// # Errors
    ///
    /// Returns an error if processing fails (e.g., missing required columns,
    /// invalid data, CT resolution failures).
    fn process(
        &self,
        domain: &Domain,
        df: &mut DataFrame,
        context: &PipelineContext,
    ) -> Result<()>;
}

/// Registry of domain processors indexed by domain code.
///
/// The registry provides lookup of processors by their domain code,
/// with a fallback to the default processor for unknown domains.
///
/// # Thread Safety
///
/// The registry is thread-safe and can be shared across threads.
/// The default registry is cached using [`OnceLock`] for efficiency.
pub struct ProcessorRegistry {
    processors: HashMap<&'static str, Box<dyn DomainProcessor>>,
    default_processor: Box<dyn DomainProcessor>,
}

impl ProcessorRegistry {
    /// Creates a new empty registry with the given default processor.
    pub fn new(default_processor: Box<dyn DomainProcessor>) -> Self {
        Self {
            processors: HashMap::new(),
            default_processor,
        }
    }

    /// Registers a processor for its domain code.
    ///
    /// If a processor for this domain is already registered, it is replaced.
    pub fn register(&mut self, processor: Box<dyn DomainProcessor>) {
        self.processors.insert(processor.domain_code(), processor);
    }

    /// Gets the processor for a domain code.
    ///
    /// Returns the registered processor if found, otherwise returns the
    /// default processor.
    pub fn get(&self, domain_code: &str) -> &dyn DomainProcessor {
        let code = domain_code.to_uppercase();
        self.processors
            .get(code.as_str())
            .map(|p| p.as_ref())
            .unwrap_or(self.default_processor.as_ref())
    }

    /// Returns the number of registered processors (excluding default).
    pub fn len(&self) -> usize {
        self.processors.len()
    }

    /// Returns true if no processors are registered (excluding default).
    pub fn is_empty(&self) -> bool {
        self.processors.is_empty()
    }

    /// Returns an iterator over all registered domain codes.
    pub fn domain_codes(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.processors.keys().copied()
    }
}

/// Default processor that handles unknown domains.
///
/// This processor applies minimal transformations suitable for any domain,
/// such as cleaning NA-like values from common columns.
pub struct DefaultProcessor;

impl DomainProcessor for DefaultProcessor {
    fn domain_code(&self) -> &'static str {
        "*" // Wildcard for default
    }

    fn description(&self) -> &'static str {
        "Default processor for unknown domains"
    }

    fn process(
        &self,
        domain: &Domain,
        df: &mut DataFrame,
        context: &PipelineContext,
    ) -> Result<()> {
        // Delegate to the existing default processor
        super::default::process_default(domain, df, context)
    }
}

/// Cached default registry with all standard processors.
static DEFAULT_REGISTRY: OnceLock<ProcessorRegistry> = OnceLock::new();

/// Returns the default processor registry with all SDTM domain processors.
///
/// The registry is cached on first access for efficiency.
///
/// # Registered Processors
///
/// The following domains are registered:
/// - AE, CM, DA, DM, DS, EX, IE, LB, MH, PE, PR, QS, SE, TA, TE, TS, VS
///
/// Unknown domains fall back to the default processor.
pub fn default_registry() -> &'static ProcessorRegistry {
    DEFAULT_REGISTRY.get_or_init(build_default_registry)
}

/// Builds the default registry with all standard processors.
fn build_default_registry() -> ProcessorRegistry {
    let mut registry = ProcessorRegistry::new(Box::new(DefaultProcessor));

    // Register all standard domain processors
    // Note: These use wrapper structs that delegate to the existing functions.
    // In Phase 2 PR-012/PR-013, these will be replaced with direct trait implementations.
    registry.register(Box::new(FunctionProcessor::new("AE", super::ae::process_ae)));
    registry.register(Box::new(FunctionProcessor::new("CM", super::cm::process_cm)));
    registry.register(Box::new(FunctionProcessor::new("DA", super::da::process_da)));
    registry.register(Box::new(FunctionProcessor::new("DM", super::dm::process_dm)));
    registry.register(Box::new(FunctionProcessor::new("DS", super::ds::process_ds)));
    registry.register(Box::new(FunctionProcessor::new("EX", super::ex::process_ex)));
    registry.register(Box::new(FunctionProcessor::new("IE", super::ie::process_ie)));
    registry.register(Box::new(FunctionProcessor::new("LB", super::lb::process_lb)));
    registry.register(Box::new(FunctionProcessor::new("MH", super::mh::process_mh)));
    registry.register(Box::new(FunctionProcessor::new("PE", super::pe::process_pe)));
    registry.register(Box::new(FunctionProcessor::new("PR", super::pr::process_pr)));
    registry.register(Box::new(FunctionProcessor::new("QS", super::qs::process_qs)));
    registry.register(Box::new(FunctionProcessor::new("SE", super::se::process_se)));
    registry.register(Box::new(FunctionProcessor::new("TA", super::ta::process_ta)));
    registry.register(Box::new(FunctionProcessor::new("TE", super::te::process_te)));
    registry.register(Box::new(FunctionProcessor::new("TS", super::ts::process_ts)));
    registry.register(Box::new(FunctionProcessor::new("VS", super::vs::process_vs)));

    registry
}

/// Wrapper that adapts a processing function to the DomainProcessor trait.
///
/// This enables gradual migration from function-based processors to
/// trait-based processors without breaking existing code.
struct FunctionProcessor {
    code: &'static str,
    process_fn: fn(&Domain, &mut DataFrame, &PipelineContext) -> Result<()>,
}

impl FunctionProcessor {
    fn new(
        code: &'static str,
        process_fn: fn(&Domain, &mut DataFrame, &PipelineContext) -> Result<()>,
    ) -> Self {
        Self { code, process_fn }
    }
}

impl DomainProcessor for FunctionProcessor {
    fn domain_code(&self) -> &'static str {
        self.code
    }

    fn description(&self) -> &'static str {
        "Function-based processor"
    }

    fn process(
        &self,
        domain: &Domain,
        df: &mut DataFrame,
        context: &PipelineContext,
    ) -> Result<()> {
        (self.process_fn)(domain, df, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_all_domains() {
        let registry = default_registry();
        assert!(registry.len() >= 17, "Expected at least 17 domain processors");

        // Check specific domains are registered
        for code in ["AE", "CM", "DA", "DM", "DS", "EX", "IE", "LB", "MH", "PE", "PR", "QS", "SE", "TA", "TE", "TS", "VS"] {
            let processor = registry.get(code);
            assert_eq!(processor.domain_code(), code, "Processor for {code} should return correct code");
        }
    }

    #[test]
    fn unknown_domain_returns_default() {
        let registry = default_registry();
        let processor = registry.get("UNKNOWN");
        assert_eq!(processor.domain_code(), "*", "Unknown domain should return default processor");
    }

    #[test]
    fn domain_codes_iterator() {
        let registry = default_registry();
        let codes: Vec<_> = registry.domain_codes().collect();
        assert!(codes.contains(&"DM"), "Should contain DM");
        assert!(codes.contains(&"AE"), "Should contain AE");
    }

    #[test]
    fn case_insensitive_lookup() {
        let registry = default_registry();

        // All these should resolve to the same processor
        assert_eq!(registry.get("dm").domain_code(), "DM");
        assert_eq!(registry.get("DM").domain_code(), "DM");
        assert_eq!(registry.get("Dm").domain_code(), "DM");
    }
}
