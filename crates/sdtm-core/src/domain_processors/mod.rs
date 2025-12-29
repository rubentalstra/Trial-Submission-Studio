//! Domain-specific processing logic for SDTM datasets.
//!
//! This module provides specialized processors for each SDTM domain, implementing
//! domain-specific business rules, derivations, and controlled terminology
//! normalization per SDTMIG v3.4.
//!
//! # Architecture
//!
//! Each domain processor implements the [`DomainProcessor`] trait, which defines
//! a common interface for domain-specific transformations:
//!
//! - Variable derivations (e.g., --DY from --DTC)
//! - Result standardization (ORRES → STRESC, STRESN)
//! - Unit normalization via CT
//! - Test code/name resolution
//! - Date validation and formatting
//!
//! Processors are registered in the [`ProcessorRegistry`] and can be looked up
//! by domain code. The [`default_registry()`] function returns a pre-configured
//! registry with all standard SDTM domain processors.
//!
//! # Usage
//!
//! ```ignore
//! use sdtm_core::domain_processors::{default_registry, DomainProcessor};
//!
//! let registry = default_registry();
//! let processor = registry.get("DM");
//! processor.process(&domain, &mut df, &context)?;
//! ```
//!
//! # Supported Domains
//!
//! | Domain | Description | Key Operations |
//! |--------|-------------|----------------|
//! | AE | Adverse Events | Severity, causality, outcome CT |
//! | CM | Concomitant Medications | Dose frequency, route CT |
//! | DA | Drug Accountability | Status, test code CT |
//! | DM | Demographics | Race, sex, ethnicity CT |
//! | DS | Disposition | Disposition event CT |
//! | EX | Exposure | Dose form, route, frequency CT |
//! | IE | Inclusion/Exclusion | Category, result CT |
//! | LB | Laboratory Results | Test, specimen, method CT |
//! | MH | Medical History | Category, relative timing CT |
//! | PE | Physical Examination | Status, location CT |
//! | PR | Procedures | Category, route CT |
//! | QS | Questionnaires | Test, category CT |
//! | SE | Subject Elements | Epoch, element CT |
//! | TA | Trial Arms | Arm code, type CT |
//! | TE | Trial Elements | Element CT |
//! | TS | Trial Summary | Parameter code, value CT |
//! | VS | Vital Signs | Test, position, location CT |

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
mod operations;
mod pe;
mod pr;
mod processor_trait;
mod qs;
mod se;
mod ta;
mod te;
mod ts;
mod vs;

pub use processor_trait::{DomainProcessor, ProcessorRegistry, default_registry};

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

/// Process a domain using the standard SDTM processor match.
///
/// Dispatches to the appropriate domain-specific processor based on
/// the domain code. Falls back to the default processor for unknown domains.
pub(crate) fn process_domain(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    match domain.code.to_uppercase().as_str() {
        "AE" => ae::process_ae(domain, df, context),
        "CM" => cm::process_cm(domain, df, context),
        "DA" => da::process_da(domain, df, context),
        "DM" => dm::process_dm(domain, df, context),
        "DS" => ds::process_ds(domain, df, context),
        "EX" => ex::process_ex(domain, df, context),
        "IE" => ie::process_ie(domain, df, context),
        "LB" => lb::process_lb(domain, df, context),
        "MH" => mh::process_mh(domain, df, context),
        "PE" => pe::process_pe(domain, df, context),
        "PR" => pr::process_pr(domain, df, context),
        "QS" => qs::process_qs(domain, df, context),
        "SE" => se::process_se(domain, df, context),
        "TA" => ta::process_ta(domain, df, context),
        "TE" => te::process_te(domain, df, context),
        "TS" => ts::process_ts(domain, df, context),
        "VS" => vs::process_vs(domain, df, context),
        _ => default::process_default(domain, df, context),
    }
}
