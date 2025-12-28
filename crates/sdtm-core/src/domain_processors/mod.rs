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

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

/// Process a domain using the standard SDTM processor match.
pub fn process_domain(domain: &Domain, df: &mut DataFrame, ctx: &ProcessingContext) -> Result<()> {
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
