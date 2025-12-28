use sdtm_model::{Domain, Variable};

/// SDTM variable roles per SDTMIG v3.4 Chapter 2 (Section 2.1).
/// Roles define the type of information conveyed by a variable.
///
/// The order of variants defines the standard column ordering:
/// 1. Identifier - identify study, subject, domain, sequence
/// 2. Topic - focus of the observation
/// 3. Qualifiers (in order): Grouping, Result, Synonym, Record, Variable
/// 4. Rule - Trial Design Model conditions (start, end, branch, loop)
/// 5. Timing - timing of the observation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SdtmRole {
    /// Identifier variables (STUDYID, USUBJID, DOMAIN, --SEQ)
    Identifier,
    /// Topic variables - focus of observation (e.g., lab test name)
    Topic,
    /// Grouping Qualifier - group observations (--CAT, --SCAT)
    GroupingQualifier,
    /// Result Qualifier - describe results (--ORRES, --STRESC, --STRESN)
    ResultQualifier,
    /// Synonym Qualifier - alternative names (--MODIFY, --DECOD)
    SynonymQualifier,
    /// Record Qualifier - attributes of the record as a whole
    RecordQualifier,
    /// Variable Qualifier - modify specific variables (--ORRESU, --DOSU)
    VariableQualifier,
    /// Rule variables - Trial Design conditions (start, end, branch, loop)
    Rule,
    /// Timing variables - timing of observation (--STDTC, --ENDTC, --DY)
    Timing,
}

impl SdtmRole {
    /// Parse a role string from SDTMIG metadata into an SdtmRole.
    /// Returns None for empty or unrecognized role strings.
    fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("IDENTIFIER") {
            Some(SdtmRole::Identifier)
        } else if trimmed.eq_ignore_ascii_case("TOPIC") {
            Some(SdtmRole::Topic)
        } else if trimmed.eq_ignore_ascii_case("GROUPING QUALIFIER") {
            Some(SdtmRole::GroupingQualifier)
        } else if trimmed.eq_ignore_ascii_case("RESULT QUALIFIER") {
            Some(SdtmRole::ResultQualifier)
        } else if trimmed.eq_ignore_ascii_case("SYNONYM QUALIFIER") {
            Some(SdtmRole::SynonymQualifier)
        } else if trimmed.eq_ignore_ascii_case("RECORD QUALIFIER") {
            Some(SdtmRole::RecordQualifier)
        } else if trimmed.eq_ignore_ascii_case("VARIABLE QUALIFIER") {
            Some(SdtmRole::VariableQualifier)
        } else if trimmed.eq_ignore_ascii_case("RULE") {
            Some(SdtmRole::Rule)
        } else if trimmed.eq_ignore_ascii_case("TIMING") {
            Some(SdtmRole::Timing)
        } else {
            None
        }
    }

    /// Returns the sort order for this role (lower = earlier in output).
    /// Per SDTMIG v3.4 Chapter 2: Identifiers, Topic, Qualifiers, Rule, Timing.
    fn sort_order(&self) -> u8 {
        match self {
            SdtmRole::Identifier => 1,
            SdtmRole::Topic => 2,
            SdtmRole::GroupingQualifier => 3,
            SdtmRole::ResultQualifier => 4,
            SdtmRole::SynonymQualifier => 5,
            SdtmRole::RecordQualifier => 6,
            SdtmRole::VariableQualifier => 7,
            SdtmRole::Rule => 8,
            SdtmRole::Timing => 9,
        }
    }
}

/// Get the sort key for a variable based on SDTM role and order.
/// Uses the variable's order field if present, otherwise uses role order * 1000.
/// This ensures variables are sorted by role first, then by their defined order within each role.
fn variable_sort_key(var: &Variable) -> (u8, u32) {
    let role = var
        .role
        .as_ref()
        .and_then(|r| SdtmRole::parse(r))
        .map(|r| r.sort_order())
        .unwrap_or(99); // Unknown roles sort last

    let order = var.order.unwrap_or(999);
    (role, order)
}

/// Order variables by SDTM role per SDTMIG v3.4 Chapter 2.
/// Within each role category, variables are ordered by their defined order field.
///
/// Returns a new Vec with variables sorted by role then order.
pub fn order_variables_by_role(variables: &[Variable]) -> Vec<Variable> {
    let mut sorted: Vec<Variable> = variables.to_vec();
    sorted.sort_by_key(variable_sort_key);
    sorted
}

#[derive(Debug, Clone, Default)]
pub struct StandardColumns {
    pub study_id: Option<String>,
    pub domain: Option<String>,
    pub rdomain: Option<String>,
    pub usubjid: Option<String>,
    pub idvar: Option<String>,
    pub idvarval: Option<String>,
    pub qnam: Option<String>,
    pub qlabel: Option<String>,
    pub qval: Option<String>,
    pub qorig: Option<String>,
    pub qeval: Option<String>,
    pub relid: Option<String>,
    pub reltype: Option<String>,
    pub refid: Option<String>,
    pub spec: Option<String>,
    pub parent: Option<String>,
    pub level: Option<String>,
}

pub fn column_name(domain: &Domain, canonical: &str) -> Option<String> {
    domain
        .variables
        .iter()
        .find(|variable| variable.name.eq_ignore_ascii_case(canonical))
        .map(|variable| variable.name.clone())
}

pub fn standard_columns(domain: &Domain) -> StandardColumns {
    let mut columns = StandardColumns::default();
    for variable in &domain.variables {
        let name = variable.name.as_str();
        if columns.study_id.is_none() && name.eq_ignore_ascii_case("STUDYID") {
            columns.study_id = Some(variable.name.clone());
        } else if columns.domain.is_none() && name.eq_ignore_ascii_case("DOMAIN") {
            columns.domain = Some(variable.name.clone());
        } else if columns.rdomain.is_none() && name.eq_ignore_ascii_case("RDOMAIN") {
            columns.rdomain = Some(variable.name.clone());
        } else if columns.usubjid.is_none() && name.eq_ignore_ascii_case("USUBJID") {
            columns.usubjid = Some(variable.name.clone());
        } else if columns.idvar.is_none() && name.eq_ignore_ascii_case("IDVAR") {
            columns.idvar = Some(variable.name.clone());
        } else if columns.idvarval.is_none() && name.eq_ignore_ascii_case("IDVARVAL") {
            columns.idvarval = Some(variable.name.clone());
        } else if columns.qnam.is_none() && name.eq_ignore_ascii_case("QNAM") {
            columns.qnam = Some(variable.name.clone());
        } else if columns.qlabel.is_none() && name.eq_ignore_ascii_case("QLABEL") {
            columns.qlabel = Some(variable.name.clone());
        } else if columns.qval.is_none() && name.eq_ignore_ascii_case("QVAL") {
            columns.qval = Some(variable.name.clone());
        } else if columns.qorig.is_none() && name.eq_ignore_ascii_case("QORIG") {
            columns.qorig = Some(variable.name.clone());
        } else if columns.qeval.is_none() && name.eq_ignore_ascii_case("QEVAL") {
            columns.qeval = Some(variable.name.clone());
        } else if columns.relid.is_none() && name.eq_ignore_ascii_case("RELID") {
            columns.relid = Some(variable.name.clone());
        } else if columns.reltype.is_none() && name.eq_ignore_ascii_case("RELTYPE") {
            columns.reltype = Some(variable.name.clone());
        } else if columns.refid.is_none() && name.eq_ignore_ascii_case("REFID") {
            columns.refid = Some(variable.name.clone());
        } else if columns.spec.is_none() && name.eq_ignore_ascii_case("SPEC") {
            columns.spec = Some(variable.name.clone());
        } else if columns.parent.is_none() && name.eq_ignore_ascii_case("PARENT") {
            columns.parent = Some(variable.name.clone());
        } else if columns.level.is_none() && name.eq_ignore_ascii_case("LEVEL") {
            columns.level = Some(variable.name.clone());
        }
    }
    columns
}

pub(crate) fn infer_seq_column(domain: &Domain) -> Option<String> {
    let code = domain.code.to_uppercase();
    let expected = format!("{code}SEQ");
    if domain
        .variables
        .iter()
        .any(|var| var.name.eq_ignore_ascii_case(&expected))
    {
        return Some(expected);
    }
    let mut candidates: Vec<&str> = domain
        .variables
        .iter()
        .map(|var| var.name.as_str())
        .filter(|name| ends_with_case_insensitive(name, "SEQ") && !name.eq_ignore_ascii_case("SEQ"))
        .collect();
    candidates.sort_by_key(|name| name.to_ascii_uppercase());
    if let Some(name) = candidates.first() {
        return Some((*name).to_string());
    }
    let mut grp_candidates: Vec<&str> = domain
        .variables
        .iter()
        .map(|var| var.name.as_str())
        .filter(|name| {
            ends_with_case_insensitive(name, "GRPID") && !name.eq_ignore_ascii_case("GRPID")
        })
        .collect();
    grp_candidates.sort_by_key(|name| name.to_ascii_uppercase());
    grp_candidates.first().map(|name| (*name).to_string())
}

pub(crate) fn refid_candidates(domain: &Domain) -> Vec<String> {
    domain
        .variables
        .iter()
        .map(|var| var.name.clone())
        .filter(|name| {
            name.eq_ignore_ascii_case("REFID") || ends_with_case_insensitive(name, "REFID")
        })
        .collect()
}

fn ends_with_case_insensitive(value: &str, suffix: &str) -> bool {
    if value.len() < suffix.len() {
        return false;
    }
    value[value.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}
