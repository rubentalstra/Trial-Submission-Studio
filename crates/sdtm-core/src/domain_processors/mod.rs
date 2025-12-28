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
mod qs;
mod se;
mod ta;
mod te;
mod ts;
mod vs;

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

/// Process a domain using the standard SDTM processor match.
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
