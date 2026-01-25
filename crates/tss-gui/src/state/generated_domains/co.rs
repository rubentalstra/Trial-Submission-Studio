//! CO (Comments) domain entry type.
//!
//! CO is a **Special-Purpose** domain per SDTM-IG v3.4 Section 5.1.
//! It captures free-text comments that can be linked to records in other domains.

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

    /// Evaluator role (COEVAL).
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

    /// Set the evaluator role.
    pub fn with_coeval(mut self, coeval: impl Into<String>) -> Self {
        self.coeval = Some(coeval.into());
        self
    }
}
