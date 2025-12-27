mod ae;
mod cm;
mod common;
mod da;
mod default;
mod dm;
mod ds;
mod ex;
mod ie;
mod lb;
mod mh;
mod pe;
mod pr;
mod qs;
mod se;
mod ta;
mod te;
mod ts;
mod vs;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

/// Trait for domain-specific processing logic.
///
/// Each domain processor implements this trait to apply domain-specific
/// transformations, validations, and derivations to a DataFrame.
pub trait DomainProcessor: Send + Sync {
    /// Process a domain DataFrame, applying domain-specific rules.
    ///
    /// # Arguments
    /// * `domain` - The domain metadata from SDTMIG standards
    /// * `df` - The DataFrame to process (modified in place)
    /// * `ctx` - Processing context with study metadata and options
    fn process(&self, domain: &Domain, df: &mut DataFrame, ctx: &ProcessingContext) -> Result<()>;

    /// Returns the domain code this processor handles (e.g., "AE", "DM").
    fn domain_code(&self) -> &str;
}

/// Registry for domain processors, allowing dynamic processor lookup and configuration.
///
/// This replaces the hard-coded match statement with a configurable registry
/// that supports:
/// - Adding custom processors for sponsor-defined domains
/// - Disabling standard processors via configuration
/// - Overriding standard processors with custom implementations
#[derive(Default)]
pub struct DomainProcessorRegistry {
    processors: HashMap<String, Arc<dyn DomainProcessor>>,
    default_processor: Option<Arc<dyn DomainProcessor>>,
    disabled: HashMap<String, bool>,
}

impl DomainProcessorRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a processor for a domain code.
    ///
    /// If a processor already exists for this domain, it will be replaced.
    pub fn register(&mut self, processor: Arc<dyn DomainProcessor>) {
        let code = processor.domain_code().to_uppercase();
        self.processors.insert(code, processor);
    }

    /// Set the default processor for domains without a specific processor.
    pub fn set_default(&mut self, processor: Arc<dyn DomainProcessor>) {
        self.default_processor = Some(processor);
    }

    /// Disable processing for a specific domain.
    pub fn disable(&mut self, domain_code: &str) {
        self.disabled.insert(domain_code.to_uppercase(), true);
    }

    /// Enable processing for a previously disabled domain.
    pub fn enable(&mut self, domain_code: &str) {
        self.disabled.remove(&domain_code.to_uppercase());
    }

    /// Check if a domain is disabled.
    pub fn is_disabled(&self, domain_code: &str) -> bool {
        self.disabled
            .get(&domain_code.to_uppercase())
            .copied()
            .unwrap_or(false)
    }

    /// Get the processor for a domain code.
    ///
    /// Returns the specific processor if registered, otherwise the default processor.
    pub fn get(&self, domain_code: &str) -> Option<Arc<dyn DomainProcessor>> {
        let code = domain_code.to_uppercase();
        if self.is_disabled(&code) {
            return None;
        }
        self.processors
            .get(&code)
            .cloned()
            .or_else(|| self.default_processor.clone())
    }

    /// Process a domain using the registered processor.
    ///
    /// If the domain is disabled, returns Ok without processing.
    /// If no processor is found, returns Ok without processing.
    pub fn process(
        &self,
        domain: &Domain,
        df: &mut DataFrame,
        ctx: &ProcessingContext,
    ) -> Result<()> {
        let code = domain.code.to_uppercase();
        if self.is_disabled(&code) {
            return Ok(());
        }
        if let Some(processor) = self.get(&code) {
            processor.process(domain, df, ctx)
        } else {
            Ok(())
        }
    }

    /// List all registered domain codes.
    pub fn registered_domains(&self) -> Vec<String> {
        self.processors.keys().cloned().collect()
    }
}

// ============================================================================
// Domain Processor Implementations
// ============================================================================

/// Default processor for domains without specific processing rules.
pub struct DefaultProcessor;

impl DomainProcessor for DefaultProcessor {
    fn process(&self, domain: &Domain, df: &mut DataFrame, ctx: &ProcessingContext) -> Result<()> {
        default::process_default(domain, df, ctx)
    }

    fn domain_code(&self) -> &str {
        "DEFAULT"
    }
}

macro_rules! impl_domain_processor {
    ($name:ident, $code:literal, $func:path) => {
        pub struct $name;

        impl DomainProcessor for $name {
            fn process(
                &self,
                domain: &Domain,
                df: &mut DataFrame,
                ctx: &ProcessingContext,
            ) -> Result<()> {
                $func(domain, df, ctx)
            }

            fn domain_code(&self) -> &str {
                $code
            }
        }
    };
}

impl_domain_processor!(AeProcessor, "AE", ae::process_ae);
impl_domain_processor!(CmProcessor, "CM", cm::process_cm);
impl_domain_processor!(DaProcessor, "DA", da::process_da);
impl_domain_processor!(DmProcessor, "DM", dm::process_dm);
impl_domain_processor!(DsProcessor, "DS", ds::process_ds);
impl_domain_processor!(ExProcessor, "EX", ex::process_ex);
impl_domain_processor!(IeProcessor, "IE", ie::process_ie);
impl_domain_processor!(LbProcessor, "LB", lb::process_lb);
impl_domain_processor!(MhProcessor, "MH", mh::process_mh);
impl_domain_processor!(PeProcessor, "PE", pe::process_pe);
impl_domain_processor!(PrProcessor, "PR", pr::process_pr);
impl_domain_processor!(QsProcessor, "QS", qs::process_qs);
impl_domain_processor!(SeProcessor, "SE", se::process_se);
impl_domain_processor!(TaProcessor, "TA", ta::process_ta);
impl_domain_processor!(TeProcessor, "TE", te::process_te);
impl_domain_processor!(TsProcessor, "TS", ts::process_ts);
impl_domain_processor!(VsProcessor, "VS", vs::process_vs);

/// Build the default registry with all standard SDTM domain processors.
///
/// This registers processors for: AE, CM, DA, DM, DS, EX, IE, LB, MH, PE, PR,
/// QS, SE, TA, TE, TS, VS, and sets DefaultProcessor as the fallback.
pub fn build_default_registry() -> DomainProcessorRegistry {
    let mut registry = DomainProcessorRegistry::new();

    // Register all standard processors
    registry.register(Arc::new(AeProcessor));
    registry.register(Arc::new(CmProcessor));
    registry.register(Arc::new(DaProcessor));
    registry.register(Arc::new(DmProcessor));
    registry.register(Arc::new(DsProcessor));
    registry.register(Arc::new(ExProcessor));
    registry.register(Arc::new(IeProcessor));
    registry.register(Arc::new(LbProcessor));
    registry.register(Arc::new(MhProcessor));
    registry.register(Arc::new(PeProcessor));
    registry.register(Arc::new(PrProcessor));
    registry.register(Arc::new(QsProcessor));
    registry.register(Arc::new(SeProcessor));
    registry.register(Arc::new(TaProcessor));
    registry.register(Arc::new(TeProcessor));
    registry.register(Arc::new(TsProcessor));
    registry.register(Arc::new(VsProcessor));

    // Set default processor for unregistered domains
    registry.set_default(Arc::new(DefaultProcessor));

    registry
}

/// Process a domain using the default registry.
///
/// This is the primary entry point for domain processing. It uses the
/// default registry which includes all standard SDTM domain processors.
pub fn process_domain(domain: &Domain, df: &mut DataFrame, ctx: &ProcessingContext) -> Result<()> {
    // Use a static-like approach: build registry and process
    // For performance, callers should use build_default_registry() and reuse it
    match domain.code.to_uppercase().as_str() {
        "AE" => ae::process_ae(domain, df, ctx),
        "CM" => cm::process_cm(domain, df, ctx),
        "DA" => da::process_da(domain, df, ctx),
        "DM" => dm::process_dm(domain, df, ctx),
        "DS" => ds::process_ds(domain, df, ctx),
        "EX" => ex::process_ex(domain, df, ctx),
        "IE" => ie::process_ie(domain, df, ctx),
        "LB" => lb::process_lb(domain, df, ctx),
        "MH" => mh::process_mh(domain, df, ctx),
        "PE" => pe::process_pe(domain, df, ctx),
        "PR" => pr::process_pr(domain, df, ctx),
        "QS" => qs::process_qs(domain, df, ctx),
        "SE" => se::process_se(domain, df, ctx),
        "TA" => ta::process_ta(domain, df, ctx),
        "TE" => te::process_te(domain, df, ctx),
        "TS" => ts::process_ts(domain, df, ctx),
        "VS" => vs::process_vs(domain, df, ctx),
        _ => default::process_default(domain, df, ctx),
    }
}

/// Process a domain using a provided registry.
///
/// Use this when you need custom processor configuration or want to reuse
/// a registry across multiple domain processing calls.
pub fn process_domain_with_registry(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
    registry: &DomainProcessorRegistry,
) -> Result<()> {
    registry.process(domain, df, ctx)
}
