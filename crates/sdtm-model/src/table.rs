#![deny(unsafe_code)]

use std::collections::BTreeMap;

use crate::{DomainCode, RowId, VarName};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum CellValue {
    Text(String),
    Missing,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Row {
    pub id: RowId,
    pub cells: BTreeMap<VarName, CellValue>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Table {
    pub domain: DomainCode,
    pub columns: Vec<VarName>,
    pub rows: Vec<Row>,
}

impl Table {
    pub fn new(domain: DomainCode, columns: Vec<VarName>) -> Self {
        Self {
            domain,
            columns,
            rows: Vec::new(),
        }
    }

    pub fn push_row(&mut self, row: Row) {
        self.rows.push(row);
    }
}
