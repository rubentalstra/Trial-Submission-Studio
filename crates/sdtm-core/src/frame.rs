use polars::prelude::DataFrame;

#[derive(Debug, Clone)]
pub struct DomainFrame {
    pub domain_code: String,
    pub data: DataFrame,
}

impl DomainFrame {
    pub fn record_count(&self) -> usize {
        self.data.height()
    }
}
