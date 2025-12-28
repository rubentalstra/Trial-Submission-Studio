use std::collections::BTreeMap;

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
pub enum SdtmRole {
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
    pub fn parse(s: &str) -> Option<Self> {
        let normalized = s.trim().to_uppercase();
        match normalized.as_str() {
            "IDENTIFIER" => Some(SdtmRole::Identifier),
            "TOPIC" => Some(SdtmRole::Topic),
            "GROUPING QUALIFIER" => Some(SdtmRole::GroupingQualifier),
            "RESULT QUALIFIER" => Some(SdtmRole::ResultQualifier),
            "SYNONYM QUALIFIER" => Some(SdtmRole::SynonymQualifier),
            "RECORD QUALIFIER" => Some(SdtmRole::RecordQualifier),
            "VARIABLE QUALIFIER" => Some(SdtmRole::VariableQualifier),
            "RULE" => Some(SdtmRole::Rule),
            "TIMING" => Some(SdtmRole::Timing),
            _ => None,
        }
    }

    /// Returns the sort order for this role (lower = earlier in output).
    /// Per SDTMIG v3.4 Chapter 2: Identifiers, Topic, Qualifiers, Rule, Timing.
    pub fn sort_order(&self) -> u8 {
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

    /// Returns the role category name as it appears in SDTMIG.
    pub fn as_str(&self) -> &'static str {
        match self {
            SdtmRole::Identifier => "Identifier",
            SdtmRole::Topic => "Topic",
            SdtmRole::GroupingQualifier => "Grouping Qualifier",
            SdtmRole::ResultQualifier => "Result Qualifier",
            SdtmRole::SynonymQualifier => "Synonym Qualifier",
            SdtmRole::RecordQualifier => "Record Qualifier",
            SdtmRole::VariableQualifier => "Variable Qualifier",
            SdtmRole::Rule => "Rule",
            SdtmRole::Timing => "Timing",
        }
    }

    /// Returns true if this is any type of Qualifier role.
    pub fn is_qualifier(&self) -> bool {
        matches!(
            self,
            SdtmRole::GroupingQualifier
                | SdtmRole::ResultQualifier
                | SdtmRole::SynonymQualifier
                | SdtmRole::RecordQualifier
                | SdtmRole::VariableQualifier
        )
    }
}

impl std::fmt::Display for SdtmRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Get the sort key for a variable based on SDTM role and order.
/// Uses the variable's order field if present, otherwise uses role order * 1000.
/// This ensures variables are sorted by role first, then by their defined order within each role.
pub fn variable_sort_key(var: &Variable) -> (u8, u32) {
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

/// Result of validating column order.
#[derive(Debug, Clone)]
pub struct ColumnOrderValidation {
    /// True if columns are in correct order
    pub is_valid: bool,
    /// Description of any ordering violations
    pub violations: Vec<String>,
    /// Suggested correct order of column names
    pub suggested_order: Vec<String>,
}

/// Validate that column names are ordered according to SDTM role rules.
/// Returns validation result with any violations found.
pub fn validate_column_order(column_names: &[String], domain: &Domain) -> ColumnOrderValidation {
    // Build a map of column name -> variable for role lookup
    let var_map: BTreeMap<String, &Variable> = domain
        .variables
        .iter()
        .map(|v| (v.name.to_uppercase(), v))
        .collect();

    // Get the expected order from domain variables
    let ordered_vars = order_variables_by_role(&domain.variables);
    let expected_order: Vec<String> = ordered_vars.iter().map(|v| v.name.clone()).collect();

    // Filter expected order to only include columns that exist in input
    let input_upper: std::collections::HashSet<String> =
        column_names.iter().map(|s| s.to_uppercase()).collect();
    let suggested_order: Vec<String> = expected_order
        .into_iter()
        .filter(|name| input_upper.contains(&name.to_uppercase()))
        .collect();

    // Check for violations: compare role categories in sequence
    let mut violations = Vec::new();
    let mut prev_role_order: u8 = 0;
    let mut prev_var_name = String::new();

    for col in column_names {
        let upper = col.to_uppercase();
        if let Some(var) = var_map.get(&upper) {
            let role_order = var
                .role
                .as_ref()
                .and_then(|r| SdtmRole::parse(r))
                .map(|r| r.sort_order())
                .unwrap_or(99);

            if role_order < prev_role_order {
                let current_role = var.role.as_deref().unwrap_or("Unknown");
                violations.push(format!(
                    "{} ({}) appears after {} but should come before",
                    col, current_role, prev_var_name
                ));
            }
            prev_role_order = role_order;
            prev_var_name = col.clone();
        }
    }

    ColumnOrderValidation {
        is_valid: violations.is_empty(),
        violations,
        suggested_order,
    }
}

/// Reorder DataFrame columns according to SDTM role order.
/// Returns the list of column names in correct order.
/// Columns not in the domain definition are placed at the end in their original order.
pub fn reorder_columns_by_role(column_names: &[String], domain: &Domain) -> Vec<String> {
    // Build a map of column name (uppercase) -> original name
    let original_names: BTreeMap<String, String> = column_names
        .iter()
        .map(|s| (s.to_uppercase(), s.clone()))
        .collect();

    // Get the expected order from domain variables
    let ordered_vars = order_variables_by_role(&domain.variables);

    // Collect columns in order, preserving original casing
    let mut result: Vec<String> = Vec::new();
    let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();

    for var in &ordered_vars {
        let upper = var.name.to_uppercase();
        if let Some(original) = original_names.get(&upper) {
            result.push(original.clone());
            used.insert(upper);
        }
    }

    // Add any remaining columns not in the domain definition
    for col in column_names {
        if !used.contains(&col.to_uppercase()) {
            result.push(col.clone());
        }
    }

    result
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
    let target = canonical.to_uppercase();
    domain
        .variables
        .iter()
        .find(|variable| variable.name.to_uppercase() == target)
        .map(|variable| variable.name.clone())
}

pub fn standard_columns(domain: &Domain) -> StandardColumns {
    StandardColumns {
        study_id: column_name(domain, "STUDYID"),
        domain: column_name(domain, "DOMAIN"),
        rdomain: column_name(domain, "RDOMAIN"),
        usubjid: column_name(domain, "USUBJID"),
        idvar: column_name(domain, "IDVAR"),
        idvarval: column_name(domain, "IDVARVAL"),
        qnam: column_name(domain, "QNAM"),
        qlabel: column_name(domain, "QLABEL"),
        qval: column_name(domain, "QVAL"),
        qorig: column_name(domain, "QORIG"),
        qeval: column_name(domain, "QEVAL"),
        relid: column_name(domain, "RELID"),
        reltype: column_name(domain, "RELTYPE"),
        refid: column_name(domain, "REFID"),
        spec: column_name(domain, "SPEC"),
        parent: column_name(domain, "PARENT"),
        level: column_name(domain, "LEVEL"),
    }
}

pub fn infer_seq_column(domain: &Domain) -> Option<String> {
    let code = domain.code.to_uppercase();
    let expected = format!("{code}SEQ");
    if domain
        .variables
        .iter()
        .any(|var| var.name.eq_ignore_ascii_case(&expected))
    {
        return Some(expected);
    }
    let mut candidates: Vec<String> = domain
        .variables
        .iter()
        .map(|var| var.name.clone())
        .filter(|name| {
            let upper = name.to_uppercase();
            upper.ends_with("SEQ") && upper != "SEQ"
        })
        .collect();
    candidates.sort_by_key(|a| a.to_uppercase());
    if let Some(name) = candidates.first() {
        return Some(name.clone());
    }
    let mut grp_candidates: Vec<String> = domain
        .variables
        .iter()
        .map(|var| var.name.clone())
        .filter(|name| {
            let upper = name.to_uppercase();
            upper.ends_with("GRPID") && upper != "GRPID"
        })
        .collect();
    grp_candidates.sort_by_key(|a| a.to_uppercase());
    grp_candidates.first().cloned()
}

pub fn refid_candidates(domain: &Domain) -> Vec<String> {
    domain
        .variables
        .iter()
        .map(|var| var.name.clone())
        .filter(|name| {
            let upper = name.to_uppercase();
            upper == "REFID" || upper.ends_with("REFID")
        })
        .collect()
}
