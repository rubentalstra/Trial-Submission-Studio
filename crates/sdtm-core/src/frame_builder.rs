use anyhow::{Context, Result};
use polars::prelude::{Column, DataFrame, NamedFrom, Series};

use sdtm_ingest::CsvTable;

use crate::frame::DomainFrame;

pub fn build_domain_frame(table: &CsvTable, domain_code: &str) -> Result<DomainFrame> {
    let mut columns: Vec<Column> = Vec::with_capacity(table.headers.len());
    for (col_idx, header) in table.headers.iter().enumerate() {
        let mut values = Vec::with_capacity(table.rows.len());
        for row in &table.rows {
            values.push(row.get(col_idx).cloned().unwrap_or_default());
        }
        columns.push(Series::new(header.as_str().into(), values).into());
    }
    let data = DataFrame::new(columns).context("build dataframe")?;
    Ok(DomainFrame {
        domain_code: domain_code.to_string(),
        data,
    })
}
