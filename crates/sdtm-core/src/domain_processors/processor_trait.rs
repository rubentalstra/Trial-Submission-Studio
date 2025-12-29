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

/// Macro to create a domain processor from a processing function.
///
/// Generates a struct implementing [`DomainProcessor`] that delegates to
/// the specified processing function. This provides compile-time generation
/// instead of runtime wrapping.
///
/// # Syntax
///
/// ```ignore
/// domain_processor!("AE", AEProcessor, process_ae);
/// domain_processor!("DM", DMProcessor, process_dm, "Demographics domain processor");
/// ```
///
/// # Arguments
///
/// * `code` - The SDTM domain code (e.g., "AE", "DM", "VS")
/// * `struct_name` - The name of the generated processor struct
/// * `fn_name` - The processing function to delegate to
/// * `description` - Optional description string (defaults to "Domain processor")
///
/// # Example
///
/// ```ignore
/// // In ae.rs:
/// pub(super) fn process_ae(domain: &Domain, df: &mut DataFrame, ctx: &PipelineContext) -> Result<()> {
///     // AE-specific processing...
///     Ok(())
/// }
///
/// // In processor_trait.rs:
/// domain_processor!("AE", AEProcessor, super::ae::process_ae, "Adverse Events processor");
///
/// // Register in build_default_registry:
/// registry.register(Box::new(AEProcessor));
/// ```
macro_rules! domain_processor {
    ($code:literal, $struct_name:ident, $fn:path) => {
        struct $struct_name;

        impl DomainProcessor for $struct_name {
            fn domain_code(&self) -> &'static str {
                $code
            }

            fn process(
                &self,
                domain: &Domain,
                df: &mut DataFrame,
                context: &PipelineContext,
            ) -> Result<()> {
                $fn(domain, df, context)
            }
        }
    };
    ($code:literal, $struct_name:ident, $fn:path, $desc:literal) => {
        struct $struct_name;

        impl DomainProcessor for $struct_name {
            fn domain_code(&self) -> &'static str {
                $code
            }

            fn description(&self) -> &'static str {
                $desc
            }

            fn process(
                &self,
                domain: &Domain,
                df: &mut DataFrame,
                context: &PipelineContext,
            ) -> Result<()> {
                $fn(domain, df, context)
            }
        }
    };
}

// Generate all domain processor structs at compile time
domain_processor!("AE", AEProcessor, super::ae::process_ae, "Adverse Events processor");
domain_processor!("CM", CMProcessor, super::cm::process_cm, "Concomitant Medications processor");
domain_processor!("DA", DAProcessor, super::da::process_da, "Drug Accountability processor");
domain_processor!("DM", DMProcessor, super::dm::process_dm, "Demographics processor");
domain_processor!("DS", DSProcessor, super::ds::process_ds, "Disposition processor");
domain_processor!("EX", EXProcessor, super::ex::process_ex, "Exposure processor");
domain_processor!("IE", IEProcessor, super::ie::process_ie, "Inclusion/Exclusion processor");
domain_processor!("LB", LBProcessor, super::lb::process_lb, "Laboratory Results processor");
domain_processor!("MH", MHProcessor, super::mh::process_mh, "Medical History processor");
domain_processor!("PE", PEProcessor, super::pe::process_pe, "Physical Examination processor");
domain_processor!("PR", PRProcessor, super::pr::process_pr, "Procedures processor");
domain_processor!("QS", QSProcessor, super::qs::process_qs, "Questionnaires processor");
domain_processor!("SE", SEProcessor, super::se::process_se, "Subject Elements processor");
domain_processor!("TA", TAProcessor, super::ta::process_ta, "Trial Arms processor");
domain_processor!("TE", TEProcessor, super::te::process_te, "Trial Elements processor");
domain_processor!("TS", TSProcessor, super::ts::process_ts, "Trial Summary processor");
domain_processor!("VS", VSProcessor, super::vs::process_vs, "Vital Signs processor");

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
