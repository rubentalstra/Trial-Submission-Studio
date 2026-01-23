//! SUPP column configuration snapshots.

use rkyv::{Archive, Deserialize, Serialize};

/// Snapshot of SUPP column configuration.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct SuppColumnSnapshot {
    /// Source column name.
    pub column: String,

    /// QNAM - Qualifier Variable Name (max 8 chars, uppercase).
    pub qnam: String,

    /// QLABEL - Qualifier Variable Label (max 40 chars).
    pub qlabel: String,

    /// QORIG - Origin of the data.
    pub qorig: SuppOriginSnapshot,

    /// QEVAL - Evaluator (optional).
    pub qeval: Option<String>,

    /// Action: whether to include in SUPP or skip.
    pub action: SuppActionSnapshot,
}

impl SuppColumnSnapshot {
    /// Create a new SUPP column snapshot with default values.
    pub fn from_column(column: impl Into<String>) -> Self {
        let col = column.into();
        // Auto-generate QNAM from column name (max 8 chars, uppercase)
        let qnam = col
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(8)
            .collect::<String>()
            .to_uppercase();

        Self {
            column: col,
            qnam,
            qlabel: String::new(),
            qorig: SuppOriginSnapshot::default(),
            qeval: None,
            action: SuppActionSnapshot::default(),
        }
    }
}

/// Origin of SUPP qualifier data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub enum SuppOriginSnapshot {
    /// Data from Case Report Form.
    #[default]
    Crf,
    /// Derived from other data.
    Derived,
    /// Sponsor-assigned value.
    Assigned,
}

/// Action for a SUPP column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub enum SuppActionSnapshot {
    /// Column is pending review.
    #[default]
    Pending,
    /// Include in SUPP domain.
    Include,
    /// Skip this column (don't include in output).
    Skip,
}
