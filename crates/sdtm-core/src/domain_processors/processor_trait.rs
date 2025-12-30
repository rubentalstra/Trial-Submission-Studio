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
    fn process(&self, domain: &Domain, df: &mut DataFrame, context: &PipelineContext)
    -> Result<()>;
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
            .map(std::convert::AsRef::as_ref)
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

    // Register all standard domain processors (macro-generated at compile time)
    registry.register(Box::new(AEProcessor));
    registry.register(Box::new(CMProcessor));
    registry.register(Box::new(DAProcessor));
    registry.register(Box::new(DMProcessor));
    registry.register(Box::new(DSProcessor));
    registry.register(Box::new(EXProcessor));
    registry.register(Box::new(IEProcessor));
    registry.register(Box::new(LBProcessor));
    registry.register(Box::new(MHProcessor));
    registry.register(Box::new(PEProcessor));
    registry.register(Box::new(PRProcessor));
    registry.register(Box::new(QSProcessor));
    registry.register(Box::new(SEProcessor));
    registry.register(Box::new(TAProcessor));
    registry.register(Box::new(TEProcessor));
    registry.register(Box::new(TSProcessor));
    registry.register(Box::new(VSProcessor));

    registry
}

// ============================================================================
// Domain Processor Implementations
// ============================================================================
//
// Each processor delegates to the corresponding process_XX function in its
// domain module. The implementations are intentionally simple and explicit.

struct AEProcessor;
impl DomainProcessor for AEProcessor {
    fn domain_code(&self) -> &'static str {
        "AE"
    }
    fn description(&self) -> &'static str {
        "Adverse Events processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::ae::process_ae(d, df, ctx)
    }
}

struct CMProcessor;
impl DomainProcessor for CMProcessor {
    fn domain_code(&self) -> &'static str {
        "CM"
    }
    fn description(&self) -> &'static str {
        "Concomitant Medications processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::cm::process_cm(d, df, ctx)
    }
}

struct DAProcessor;
impl DomainProcessor for DAProcessor {
    fn domain_code(&self) -> &'static str {
        "DA"
    }
    fn description(&self) -> &'static str {
        "Drug Accountability processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::da::process_da(d, df, ctx)
    }
}

struct DMProcessor;
impl DomainProcessor for DMProcessor {
    fn domain_code(&self) -> &'static str {
        "DM"
    }
    fn description(&self) -> &'static str {
        "Demographics processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::dm::process_dm(d, df, ctx)
    }
}

struct DSProcessor;
impl DomainProcessor for DSProcessor {
    fn domain_code(&self) -> &'static str {
        "DS"
    }
    fn description(&self) -> &'static str {
        "Disposition processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::ds::process_ds(d, df, ctx)
    }
}

struct EXProcessor;
impl DomainProcessor for EXProcessor {
    fn domain_code(&self) -> &'static str {
        "EX"
    }
    fn description(&self) -> &'static str {
        "Exposure processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::ex::process_ex(d, df, ctx)
    }
}

struct IEProcessor;
impl DomainProcessor for IEProcessor {
    fn domain_code(&self) -> &'static str {
        "IE"
    }
    fn description(&self) -> &'static str {
        "Inclusion/Exclusion processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::ie::process_ie(d, df, ctx)
    }
}

struct LBProcessor;
impl DomainProcessor for LBProcessor {
    fn domain_code(&self) -> &'static str {
        "LB"
    }
    fn description(&self) -> &'static str {
        "Laboratory Results processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::lb::process_lb(d, df, ctx)
    }
}

struct MHProcessor;
impl DomainProcessor for MHProcessor {
    fn domain_code(&self) -> &'static str {
        "MH"
    }
    fn description(&self) -> &'static str {
        "Medical History processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::mh::process_mh(d, df, ctx)
    }
}

struct PEProcessor;
impl DomainProcessor for PEProcessor {
    fn domain_code(&self) -> &'static str {
        "PE"
    }
    fn description(&self) -> &'static str {
        "Physical Examination processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::pe::process_pe(d, df, ctx)
    }
}

struct PRProcessor;
impl DomainProcessor for PRProcessor {
    fn domain_code(&self) -> &'static str {
        "PR"
    }
    fn description(&self) -> &'static str {
        "Procedures processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::pr::process_pr(d, df, ctx)
    }
}

struct QSProcessor;
impl DomainProcessor for QSProcessor {
    fn domain_code(&self) -> &'static str {
        "QS"
    }
    fn description(&self) -> &'static str {
        "Questionnaires processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::qs::process_qs(d, df, ctx)
    }
}

struct SEProcessor;
impl DomainProcessor for SEProcessor {
    fn domain_code(&self) -> &'static str {
        "SE"
    }
    fn description(&self) -> &'static str {
        "Subject Elements processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::se::process_se(d, df, ctx)
    }
}

struct TAProcessor;
impl DomainProcessor for TAProcessor {
    fn domain_code(&self) -> &'static str {
        "TA"
    }
    fn description(&self) -> &'static str {
        "Trial Arms processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::ta::process_ta(d, df, ctx)
    }
}

struct TEProcessor;
impl DomainProcessor for TEProcessor {
    fn domain_code(&self) -> &'static str {
        "TE"
    }
    fn description(&self) -> &'static str {
        "Trial Elements processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::te::process_te(d, df, ctx)
    }
}

struct TSProcessor;
impl DomainProcessor for TSProcessor {
    fn domain_code(&self) -> &'static str {
        "TS"
    }
    fn description(&self) -> &'static str {
        "Trial Summary processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::ts::process_ts(d, df, ctx)
    }
}

struct VSProcessor;
impl DomainProcessor for VSProcessor {
    fn domain_code(&self) -> &'static str {
        "VS"
    }
    fn description(&self) -> &'static str {
        "Vital Signs processor"
    }
    fn process(&self, d: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
        super::vs::process_vs(d, df, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_all_domains() {
        let registry = default_registry();
        assert!(
            registry.len() >= 17,
            "Expected at least 17 domain processors"
        );

        // Check specific domains are registered
        for code in [
            "AE", "CM", "DA", "DM", "DS", "EX", "IE", "LB", "MH", "PE", "PR", "QS", "SE", "TA",
            "TE", "TS", "VS",
        ] {
            let processor = registry.get(code);
            assert_eq!(
                processor.domain_code(),
                code,
                "Processor for {code} should return correct code"
            );
        }
    }

    #[test]
    fn unknown_domain_returns_default() {
        let registry = default_registry();
        let processor = registry.get("UNKNOWN");
        assert_eq!(
            processor.domain_code(),
            "*",
            "Unknown domain should return default processor"
        );
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

    #[test]
    fn processor_descriptions_are_set() {
        let registry = default_registry();

        // Macro-generated processors should have descriptive descriptions
        assert_eq!(registry.get("AE").description(), "Adverse Events processor");
        assert_eq!(registry.get("DM").description(), "Demographics processor");
        assert_eq!(registry.get("VS").description(), "Vital Signs processor");

        // Default processor has its own description
        assert_eq!(
            registry.get("UNKNOWN").description(),
            "Default processor for unknown domains"
        );
    }
}
