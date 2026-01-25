//! Relationship and special-purpose domain entry types.
//!
//! These types represent user-provided data for generating domains:
//! - **CO (Comments)**: Special-purpose domain for free-text comments
//! - **RELREC (Related Records)**: Links records within/across domains
//! - **RELSPEC (Related Specimens)**: Tracks specimen parent/child hierarchy
//! - **RELSUB (Related Subjects)**: Tracks relationships between study subjects
//!
//! # CDISC Domain Classifications
//!
//! Per SDTM-IG v3.4:
//! - CO is a **Special-Purpose** domain (Section 5.1)
//! - RELREC, RELSPEC, RELSUB are **Relationship** domains (Sections 8.2, 8.7, 8.8)

// =============================================================================
// CO (COMMENTS) - SPECIAL-PURPOSE DOMAIN
// =============================================================================

/// Entry for CO (Comments) domain.
///
/// Per SDTM-IG v3.4 Section 5.1:
/// - Comments may be standalone, linked to a domain, or linked to specific records
/// - COVAL max 200 chars; overflow to COVAL1-COVALn handled by generation service
#[derive(Debug, Clone, PartialEq)]
pub struct CommentEntry {
    /// Subject identifier (USUBJID).
    pub usubjid: String,

    /// Comment text (COVAL).
    /// Text over 200 characters will be split into COVAL, COVAL1, etc.
    pub comment: String,

    /// Related domain (RDOMAIN).
    /// Null for standalone comments collected on general comment pages.
    pub rdomain: Option<String>,

    /// Identifying variable name (IDVAR).
    /// e.g., "AESEQ", "CMGRPID". Used when linking to specific records.
    pub idvar: Option<String>,

    /// Identifying variable value (IDVARVAL).
    /// Value of the variable named in IDVAR.
    pub idvarval: Option<String>,

    /// Comment reference (COREF).
    /// Sponsor-defined reference (e.g., CRF page number, module name).
    pub coref: Option<String>,

    /// Date/time of comment (CODTC).
    /// ISO 8601 format. Should be null for child records of other domains.
    pub codtc: Option<String>,

    /// Evaluator (COEVAL).
    /// Role of the person who provided the comment (e.g., "INVESTIGATOR").
    pub coeval: Option<String>,
}

impl CommentEntry {
    /// Create a new standalone comment (not linked to any domain).
    pub fn standalone(usubjid: impl Into<String>, comment: impl Into<String>) -> Self {
        Self {
            usubjid: usubjid.into(),
            comment: comment.into(),
            rdomain: None,
            idvar: None,
            idvarval: None,
            coref: None,
            codtc: None,
            coeval: None,
        }
    }

    /// Create a comment linked to a domain (but not specific records).
    pub fn for_domain(
        usubjid: impl Into<String>,
        comment: impl Into<String>,
        rdomain: impl Into<String>,
    ) -> Self {
        Self {
            usubjid: usubjid.into(),
            comment: comment.into(),
            rdomain: Some(rdomain.into()),
            idvar: None,
            idvarval: None,
            coref: None,
            codtc: None,
            coeval: None,
        }
    }

    /// Create a comment linked to specific record(s).
    pub fn for_record(
        usubjid: impl Into<String>,
        comment: impl Into<String>,
        rdomain: impl Into<String>,
        idvar: impl Into<String>,
        idvarval: impl Into<String>,
    ) -> Self {
        Self {
            usubjid: usubjid.into(),
            comment: comment.into(),
            rdomain: Some(rdomain.into()),
            idvar: Some(idvar.into()),
            idvarval: Some(idvarval.into()),
            coref: None,
            codtc: None,
            coeval: None,
        }
    }

    /// Set the comment reference.
    pub fn with_coref(mut self, coref: impl Into<String>) -> Self {
        self.coref = Some(coref.into());
        self
    }

    /// Set the comment date/time.
    pub fn with_codtc(mut self, codtc: impl Into<String>) -> Self {
        self.codtc = Some(codtc.into());
        self
    }

    /// Set the evaluator.
    pub fn with_coeval(mut self, coeval: impl Into<String>) -> Self {
        self.coeval = Some(coeval.into());
        self
    }
}

// =============================================================================
// RELREC (RELATED RECORDS) - RELATIONSHIP DOMAIN
// =============================================================================

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

// =============================================================================
// RELSPEC (RELATED SPECIMENS) - RELATIONSHIP DOMAIN
// =============================================================================

/// Entry for RELSPEC (Related Specimens) domain.
///
/// Per SDTM-IG v3.4 Section 8.8:
/// - Tracks specimen parent/child hierarchy
/// - LEVEL=1 for collected samples (PARENT is null)
/// - LEVEL is calculated automatically from hierarchy depth
#[derive(Debug, Clone, PartialEq)]
pub struct RelspecEntry {
    /// Subject identifier (USUBJID).
    pub usubjid: String,

    /// Specimen identifier (REFID).
    /// Unique within USUBJID.
    pub refid: String,

    /// Specimen type (SPEC).
    /// e.g., "SERUM", "PLASMA", "URINE", "TISSUE".
    pub spec: Option<String>,

    /// Parent specimen identifier (PARENT).
    /// REFID of the parent specimen. Null for collected samples (LEVEL=1).
    pub parent: Option<String>,
}

impl RelspecEntry {
    /// Create a collected specimen entry (LEVEL=1, no parent).
    pub fn collected(
        usubjid: impl Into<String>,
        refid: impl Into<String>,
        spec: Option<String>,
    ) -> Self {
        Self {
            usubjid: usubjid.into(),
            refid: refid.into(),
            spec,
            parent: None,
        }
    }

    /// Create a derived specimen entry (has parent).
    pub fn derived(
        usubjid: impl Into<String>,
        refid: impl Into<String>,
        parent: impl Into<String>,
        spec: Option<String>,
    ) -> Self {
        Self {
            usubjid: usubjid.into(),
            refid: refid.into(),
            spec,
            parent: Some(parent.into()),
        }
    }
}

// =============================================================================
// RELSUB (RELATED SUBJECTS) - RELATIONSHIP DOMAIN
// =============================================================================

/// Entry for RELSUB (Related Subjects) domain.
///
/// Per SDTM-IG v3.4 Section 8.7:
/// - Represents relationships between study subjects
/// - **MUST be bidirectional**: if A to B exists, B to A must also exist
/// - The generation service handles auto-creating reciprocal relationships
#[derive(Debug, Clone, PartialEq)]
pub struct RelsubEntry {
    /// Subject identifier (USUBJID).
    pub usubjid: String,

    /// Related subject identifier (RSUBJID).
    /// Must be a USUBJID present in the DM domain.
    pub rsubjid: String,

    /// Subject relationship (SREL).
    /// From RELSUB codelist (e.g., "MOTHER, BIOLOGICAL", "TWIN, DIZYGOTIC").
    pub srel: String,
}

impl RelsubEntry {
    /// Create a new subject relationship entry.
    ///
    /// Note: The generation service will automatically create the reciprocal
    /// relationship (e.g., if you create MOTHER to CHILD, CHILD to MOTHER is auto-added).
    pub fn new(
        usubjid: impl Into<String>,
        rsubjid: impl Into<String>,
        srel: impl Into<String>,
    ) -> Self {
        Self {
            usubjid: usubjid.into(),
            rsubjid: rsubjid.into(),
            srel: srel.into(),
        }
    }
}

// =============================================================================
// UNIFIED ENTRY ENUM
// =============================================================================

/// A relationship or special-purpose domain entry.
///
/// This enum allows storing entries for different generated domains
/// in a single collection when needed.
#[derive(Debug, Clone, PartialEq)]
pub enum GeneratedDomainEntry {
    /// CO (Comments) - Special-Purpose domain.
    Comment(CommentEntry),
    /// RELREC (Related Records) - Relationship domain.
    RelatedRecord(RelrecEntry),
    /// RELSPEC (Related Specimens) - Relationship domain.
    RelatedSpecimen(RelspecEntry),
    /// RELSUB (Related Subjects) - Relationship domain.
    RelatedSubject(RelsubEntry),
}

impl From<CommentEntry> for GeneratedDomainEntry {
    fn from(entry: CommentEntry) -> Self {
        Self::Comment(entry)
    }
}

impl From<RelrecEntry> for GeneratedDomainEntry {
    fn from(entry: RelrecEntry) -> Self {
        Self::RelatedRecord(entry)
    }
}

impl From<RelspecEntry> for GeneratedDomainEntry {
    fn from(entry: RelspecEntry) -> Self {
        Self::RelatedSpecimen(entry)
    }
}

impl From<RelsubEntry> for GeneratedDomainEntry {
    fn from(entry: RelsubEntry) -> Self {
        Self::RelatedSubject(entry)
    }
}

// =============================================================================
// GENERATED DOMAIN TYPE
// =============================================================================

/// Type of generated domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneratedDomainType {
    /// CO (Comments) - Special-Purpose domain.
    Comments,
    /// RELREC (Related Records) - Relationship domain.
    RelatedRecords,
    /// RELSPEC (Related Specimens) - Relationship domain.
    RelatedSpecimens,
    /// RELSUB (Related Subjects) - Relationship domain.
    RelatedSubjects,
}

impl GeneratedDomainType {
    /// Get the CDISC domain code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Comments => "CO",
            Self::RelatedRecords => "RELREC",
            Self::RelatedSpecimens => "RELSPEC",
            Self::RelatedSubjects => "RELSUB",
        }
    }

    /// Get the domain class per SDTM-IG.
    pub fn domain_class(&self) -> &'static str {
        match self {
            Self::Comments => "Special-Purpose",
            Self::RelatedRecords | Self::RelatedSpecimens | Self::RelatedSubjects => "Relationship",
        }
    }

    /// Get the human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Comments => "Comments",
            Self::RelatedRecords => "Related Records",
            Self::RelatedSpecimens => "Related Specimens",
            Self::RelatedSubjects => "Related Subjects",
        }
    }

    /// Get the SDTM-IG section reference.
    pub fn ig_section(&self) -> &'static str {
        match self {
            Self::Comments => "5.1",
            Self::RelatedRecords => "8.2",
            Self::RelatedSpecimens => "8.8",
            Self::RelatedSubjects => "8.7",
        }
    }

    /// All generated domain types.
    pub const ALL: [GeneratedDomainType; 4] = [
        Self::Comments,
        Self::RelatedRecords,
        Self::RelatedSpecimens,
        Self::RelatedSubjects,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_entry_standalone() {
        let entry = CommentEntry::standalone("STUDY-001", "General comment");
        assert_eq!(entry.usubjid, "STUDY-001");
        assert_eq!(entry.comment, "General comment");
        assert!(entry.rdomain.is_none());
        assert!(entry.idvar.is_none());
    }

    #[test]
    fn test_comment_entry_for_record() {
        let entry = CommentEntry::for_record("STUDY-001", "AE comment", "AE", "AESEQ", "5");
        assert_eq!(entry.rdomain, Some("AE".to_string()));
        assert_eq!(entry.idvar, Some("AESEQ".to_string()));
        assert_eq!(entry.idvarval, Some("5".to_string()));
    }

    #[test]
    fn test_relrec_entry_record() {
        let entry = RelrecEntry::record("REL1", "STUDY-001", "AE", "AESEQ", "5");
        assert_eq!(entry.relid, "REL1");
        assert!(entry.usubjid.is_some());
        assert!(entry.reltype.is_none());
    }

    #[test]
    fn test_relrec_entry_dataset() {
        let entry = RelrecEntry::dataset("REL1", "TU", "TULNKID", RelrecRelType::One);
        assert!(entry.usubjid.is_none());
        assert!(entry.idvarval.is_none());
        assert_eq!(entry.reltype, Some(RelrecRelType::One));
    }

    #[test]
    fn test_relspec_entry_collected() {
        let entry = RelspecEntry::collected("STUDY-001", "SPC-001", Some("TISSUE".to_string()));
        assert!(entry.parent.is_none());
    }

    #[test]
    fn test_relspec_entry_derived() {
        let entry =
            RelspecEntry::derived("STUDY-001", "SPC-001-A", "SPC-001", Some("DNA".to_string()));
        assert_eq!(entry.parent, Some("SPC-001".to_string()));
    }

    #[test]
    fn test_relsub_entry() {
        let entry = RelsubEntry::new("STUDY-001", "STUDY-002", "MOTHER, BIOLOGICAL");
        assert_eq!(entry.usubjid, "STUDY-001");
        assert_eq!(entry.rsubjid, "STUDY-002");
        assert_eq!(entry.srel, "MOTHER, BIOLOGICAL");
    }

    #[test]
    fn test_generated_domain_type_codes() {
        assert_eq!(GeneratedDomainType::Comments.code(), "CO");
        assert_eq!(GeneratedDomainType::RelatedRecords.code(), "RELREC");
        assert_eq!(GeneratedDomainType::RelatedSpecimens.code(), "RELSPEC");
        assert_eq!(GeneratedDomainType::RelatedSubjects.code(), "RELSUB");
    }

    #[test]
    fn test_generated_domain_type_classes() {
        assert_eq!(
            GeneratedDomainType::Comments.domain_class(),
            "Special-Purpose"
        );
        assert_eq!(
            GeneratedDomainType::RelatedRecords.domain_class(),
            "Relationship"
        );
        assert_eq!(
            GeneratedDomainType::RelatedSpecimens.domain_class(),
            "Relationship"
        );
        assert_eq!(
            GeneratedDomainType::RelatedSubjects.domain_class(),
            "Relationship"
        );
    }
}
