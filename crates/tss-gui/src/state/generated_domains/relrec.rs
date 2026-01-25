//! RELREC (Related Records) domain entry type.
//!
//! RELREC is a **Relationship** domain per SDTM-IG v3.4 Section 8.2.
//! It links records across or within domains using a relationship identifier.

/// Entry for RELREC (Related Records) domain.
///
/// Per SDTM-IG v3.4 Section 8.2:
/// - Links records within/across domains using RELID groups
/// - Each record in a relationship group has the same RELID
#[derive(Debug, Clone, PartialEq)]
pub struct RelrecEntry {
    /// Relationship identifier (RELID).
    /// Groups related records together. All records with same RELID are related.
    pub relid: String,

    /// Subject identifier (USUBJID).
    /// Null for dataset-level relationships.
    pub usubjid: Option<String>,

    /// Related domain (RDOMAIN).
    /// Domain code of the record being linked.
    pub rdomain: String,

    /// Identifying variable name (IDVAR).
    /// e.g., "AESEQ", "CMGRPID", "LBSEQ".
    pub idvar: String,

    /// Identifying variable value (IDVARVAL).
    /// Value of the variable named in IDVAR. Null for dataset-level relationships.
    pub idvarval: Option<String>,

    /// Relationship type (RELTYPE).
    /// "ONE" or "MANY" - used for dataset-level relationships.
    pub reltype: Option<RelrecRelType>,
}

/// RELTYPE values for RELREC domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelrecRelType {
    /// Single record in the relationship.
    One,
    /// Multiple records in the relationship.
    Many,
}

impl RelrecRelType {
    /// Get CDISC code value.
    pub fn code(&self) -> &'static str {
        match self {
            Self::One => "ONE",
            Self::Many => "MANY",
        }
    }
}

impl RelrecEntry {
    /// Create a record-level relationship entry.
    pub fn record(
        relid: impl Into<String>,
        usubjid: impl Into<String>,
        rdomain: impl Into<String>,
        idvar: impl Into<String>,
        idvarval: impl Into<String>,
    ) -> Self {
        Self {
            relid: relid.into(),
            usubjid: Some(usubjid.into()),
            rdomain: rdomain.into(),
            idvar: idvar.into(),
            idvarval: Some(idvarval.into()),
            reltype: None,
        }
    }

    /// Create a dataset-level relationship entry.
    pub fn dataset(
        relid: impl Into<String>,
        rdomain: impl Into<String>,
        idvar: impl Into<String>,
        reltype: RelrecRelType,
    ) -> Self {
        Self {
            relid: relid.into(),
            usubjid: None,
            rdomain: rdomain.into(),
            idvar: idvar.into(),
            idvarval: None,
            reltype: Some(reltype),
        }
    }
}
