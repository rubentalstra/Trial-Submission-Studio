//! Cross-domain reference validation.
//!
//! Validates referential integrity across SDTM domains:
//! - All USUBJIDs in non-DM domains must exist in DM
//! - Parent record references (--SPID) must exist in parent domain
//! - CO/RELREC RDOMAIN references valid domains
//! - RELSUB RSUBJID exists in DM and relationships are bidirectional
//! - RELSPEC PARENT references valid REFID within subject
//!
//! These checks ensure data consistency across the submission package.

use polars::prelude::DataFrame;
use std::collections::{HashMap, HashSet};

use super::super::column_reader::ColumnReader;
use super::super::issue::Issue;

/// Extract all USUBJIDs from the DM domain.
///
/// Returns a set of unique, non-empty USUBJID values.
pub fn extract_dm_subjects(dm_df: &DataFrame) -> HashSet<String> {
    let reader = ColumnReader::new(dm_df);
    let mut subjects = HashSet::new();

    if let Some(values) = reader.values("USUBJID") {
        for (_, value) in values {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                subjects.insert(trimmed.to_string());
            }
        }
    }

    subjects
}

/// Check that all USUBJIDs in a domain exist in the DM domain.
///
/// # Arguments
/// * `domain_name` - Name of the domain being checked (e.g., "AE", "LB")
/// * `df` - DataFrame of the domain to check
/// * `dm_subjects` - Set of valid USUBJIDs from DM domain
///
/// # Returns
/// A vector of issues (empty if all USUBJIDs are valid).
pub fn check_usubjid_in_dm(
    domain_name: &str,
    df: &DataFrame,
    dm_subjects: &HashSet<String>,
) -> Vec<Issue> {
    let reader = ColumnReader::new(df);

    let Some(values) = reader.values("USUBJID") else {
        // No USUBJID column - nothing to check
        return vec![];
    };

    let mut missing_values = Vec::new();
    let mut missing_count = 0u64;

    for (_, usubjid) in values {
        let trimmed = usubjid.trim();
        // Skip empty values (handled by other validation)
        if !trimmed.is_empty() && !dm_subjects.contains(trimmed) {
            missing_count += 1;
            // Collect up to 5 sample values
            if missing_values.len() < 5 && !missing_values.contains(&trimmed.to_string()) {
                missing_values.push(trimmed.to_string());
            }
        }
    }

    if missing_count > 0 {
        vec![Issue::UsubjidNotInDm {
            domain: domain_name.to_string(),
            missing_count,
            samples: missing_values,
        }]
    } else {
        vec![]
    }
}

// =============================================================================
// RDOMAIN VALIDATION (CO, RELREC)
// =============================================================================

/// Check that RDOMAIN values reference valid domains in the submission.
///
/// # Arguments
/// * `domain_name` - Name of the domain being checked (e.g., "CO", "RELREC")
/// * `df` - DataFrame to check
/// * `valid_domains` - Set of valid domain codes in the submission
///
/// # Returns
/// A vector of issues (empty if all RDOMAIN values are valid).
pub fn check_rdomain_valid(
    domain_name: &str,
    df: &DataFrame,
    valid_domains: &HashSet<String>,
) -> Vec<Issue> {
    let reader = ColumnReader::new(df);

    let Some(values) = reader.values("RDOMAIN") else {
        return vec![];
    };

    let mut invalid_values = Vec::new();
    let mut invalid_count = 0u64;

    for (_, rdomain) in values {
        let trimmed = rdomain.trim().to_uppercase();
        // Skip empty values (RDOMAIN can be null for standalone comments)
        if !trimmed.is_empty() && !valid_domains.contains(&trimmed) {
            invalid_count += 1;
            if invalid_values.len() < 5 && !invalid_values.contains(&trimmed) {
                invalid_values.push(trimmed);
            }
        }
    }

    if invalid_count > 0 {
        vec![Issue::InvalidRdomain {
            domain: domain_name.to_string(),
            invalid_count,
            samples: invalid_values,
        }]
    } else {
        vec![]
    }
}

// =============================================================================
// RELSUB VALIDATION
// =============================================================================

/// Check RELSUB domain for valid RSUBJID and bidirectional relationships.
///
/// Per SDTM-IG 8.7:
/// - RSUBJID must be a USUBJID value present in DM
/// - Relationships must be bidirectional (A→B requires B→A)
///
/// # Arguments
/// * `relsub_df` - RELSUB DataFrame
/// * `dm_subjects` - Set of valid USUBJIDs from DM
///
/// # Returns
/// A vector of issues.
pub fn check_relsub(relsub_df: &DataFrame, dm_subjects: &HashSet<String>) -> Vec<Issue> {
    let mut issues = Vec::new();
    let reader = ColumnReader::new(relsub_df);

    // Check RSUBJID exists in DM
    if let Some(rsubjid_values) = reader.values("RSUBJID") {
        let mut missing_values = Vec::new();
        let mut missing_count = 0u64;

        for (_, rsubjid) in rsubjid_values {
            let trimmed = rsubjid.trim();
            if !trimmed.is_empty() && !dm_subjects.contains(trimmed) {
                missing_count += 1;
                if missing_values.len() < 5 && !missing_values.contains(&trimmed.to_string()) {
                    missing_values.push(trimmed.to_string());
                }
            }
        }

        if missing_count > 0 {
            issues.push(Issue::RelsubNotInDm {
                missing_count,
                samples: missing_values,
            });
        }
    }

    // Check bidirectional relationships
    // Collect all (USUBJID, RSUBJID) pairs
    let usubjid_values = reader.values("USUBJID");
    let rsubjid_values = reader.values("RSUBJID");

    if let (Some(usubjids), Some(rsubjids)) = (usubjid_values, rsubjid_values) {
        let mut relationships: HashSet<(String, String)> = HashSet::new();

        for ((_, usubjid), (_, rsubjid)) in usubjids.zip(rsubjids) {
            let u = usubjid.trim().to_string();
            let r = rsubjid.trim().to_string();
            if !u.is_empty() && !r.is_empty() {
                relationships.insert((u, r));
            }
        }

        // Check for missing reciprocal relationships
        let mut missing_reciprocal = Vec::new();
        let mut missing_count = 0u64;

        for (usubjid, rsubjid) in &relationships {
            // The reciprocal would be (rsubjid, usubjid)
            if !relationships.contains(&(rsubjid.clone(), usubjid.clone())) {
                missing_count += 1;
                if missing_reciprocal.len() < 5 {
                    missing_reciprocal.push(format!("{}→{}", usubjid, rsubjid));
                }
            }
        }

        if missing_count > 0 {
            issues.push(Issue::RelsubNotBidirectional {
                missing_count,
                samples: missing_reciprocal,
            });
        }
    }

    issues
}

// =============================================================================
// RELSPEC VALIDATION
// =============================================================================

/// Check RELSPEC PARENT references valid REFID within each subject.
///
/// Per SDTM-IG 8.8:
/// - PARENT must reference an existing REFID for the same USUBJID
/// - LEVEL=1 specimens should have null PARENT
///
/// # Arguments
/// * `relspec_df` - RELSPEC DataFrame
///
/// # Returns
/// A vector of issues.
pub fn check_relspec(relspec_df: &DataFrame) -> Vec<Issue> {
    let reader = ColumnReader::new(relspec_df);

    // Build a map of USUBJID -> Set of REFIDs
    let mut refids_by_subject: HashMap<String, HashSet<String>> = HashMap::new();

    let usubjid_col = reader.values("USUBJID");
    let refid_col = reader.values("REFID");

    if let (Some(usubjids), Some(refids)) = (usubjid_col, refid_col) {
        for ((_, usubjid), (_, refid)) in usubjids.zip(refids) {
            let u = usubjid.trim().to_string();
            let r = refid.trim().to_string();
            if !u.is_empty() && !r.is_empty() {
                refids_by_subject.entry(u).or_default().insert(r);
            }
        }
    }

    // Check PARENT references
    let usubjid_col = reader.values("USUBJID");
    let parent_col = reader.values("PARENT");

    let Some(parents) = parent_col else {
        return vec![];
    };

    let mut invalid_values = Vec::new();
    let mut invalid_count = 0u64;

    if let Some(usubjids) = usubjid_col {
        for ((_, usubjid), (_, parent)) in usubjids.zip(parents) {
            let u = usubjid.trim();
            let p = parent.trim();

            // Skip empty PARENT (valid for LEVEL=1 collected specimens)
            if p.is_empty() {
                continue;
            }

            // Check if PARENT exists in this subject's REFIDs
            let valid_refids = refids_by_subject.get(u);
            let parent_exists = valid_refids.is_some_and(|refs| refs.contains(p));

            if !parent_exists {
                invalid_count += 1;
                if invalid_values.len() < 5 {
                    invalid_values.push(format!("{}:{}", u, p));
                }
            }
        }
    }

    if invalid_count > 0 {
        vec![Issue::RelspecInvalidParent {
            invalid_count,
            samples: invalid_values,
        }]
    } else {
        vec![]
    }
}

// =============================================================================
// RELREC VALIDATION
// =============================================================================

/// Context for RELREC validation containing domain data.
pub struct RelrecContext<'a> {
    /// Map of domain code -> (DataFrame, key_variable -> Set of key values)
    pub domains: HashMap<String, (&'a DataFrame, HashMap<String, HashSet<String>>)>,
}

impl<'a> RelrecContext<'a> {
    /// Build context from a list of (domain_name, DataFrame) pairs.
    pub fn new(domain_list: &[(&str, &'a DataFrame)]) -> Self {
        let mut domains = HashMap::new();

        for (name, df) in domain_list {
            let reader = ColumnReader::new(df);
            let mut key_values: HashMap<String, HashSet<String>> = HashMap::new();

            // Extract common key variables: --SEQ, --GRPID, --REFID, --LNKID
            let domain_prefix = name.to_uppercase();
            let seq_var = format!("{}SEQ", domain_prefix);
            let grpid_var = format!("{}GRPID", domain_prefix);
            let refid_var = format!("{}REFID", domain_prefix);
            let lnkid_var = format!("{}LNKID", domain_prefix);

            for var in [&seq_var, &grpid_var, &refid_var, &lnkid_var] {
                if let Some(values) = reader.values(var) {
                    let mut set = HashSet::new();
                    for (_, val) in values {
                        let trimmed = val.trim();
                        if !trimmed.is_empty() {
                            set.insert(trimmed.to_string());
                        }
                    }
                    if !set.is_empty() {
                        key_values.insert(var.clone(), set);
                    }
                }
            }

            // Also check generic variable names (VISITNUM, etc.)
            for var in ["VISITNUM"] {
                if let Some(values) = reader.values(var) {
                    let mut set = HashSet::new();
                    for (_, val) in values {
                        let trimmed = val.trim();
                        if !trimmed.is_empty() {
                            set.insert(trimmed.to_string());
                        }
                    }
                    if !set.is_empty() {
                        key_values.insert(var.to_string(), set);
                    }
                }
            }

            domains.insert(name.to_uppercase(), (*df, key_values));
        }

        Self { domains }
    }

    /// Check if a reference exists in the context.
    pub fn reference_exists(&self, rdomain: &str, idvar: &str, idvarval: &str) -> bool {
        let domain_upper = rdomain.to_uppercase();
        let Some((_, key_values)) = self.domains.get(&domain_upper) else {
            return false;
        };

        // Check if the IDVAR exists and contains the IDVARVAL
        key_values
            .get(idvar)
            .is_some_and(|values| values.contains(idvarval))
    }
}

/// Check RELREC references point to existing records.
///
/// Per SDTM-IG 8.2:
/// - RDOMAIN + IDVAR + IDVARVAL should reference existing records
/// - For dataset relationships, USUBJID and IDVARVAL are null
///
/// # Arguments
/// * `relrec_df` - RELREC DataFrame
/// * `context` - Context containing all domain data
///
/// # Returns
/// A vector of issues grouped by RDOMAIN.
pub fn check_relrec(relrec_df: &DataFrame, context: &RelrecContext) -> Vec<Issue> {
    let reader = ColumnReader::new(relrec_df);

    let rdomain_col = reader.values("RDOMAIN");
    let idvar_col = reader.values("IDVAR");
    let idvarval_col = reader.values("IDVARVAL");
    let usubjid_col = reader.values("USUBJID");

    let Some(rdomains) = rdomain_col else {
        return vec![];
    };
    let Some(idvars) = idvar_col else {
        return vec![];
    };
    let Some(idvarvals) = idvarval_col else {
        return vec![];
    };

    // Group invalid references by RDOMAIN
    let mut invalid_by_domain: HashMap<String, (u64, Vec<String>)> = HashMap::new();

    // Handle case where USUBJID might be null (dataset-level relationships)
    let usubjids: Vec<String> = usubjid_col
        .map(|col| col.map(|(_, v)| v.to_string()).collect())
        .unwrap_or_else(|| vec!["".to_string(); rdomains.size_hint().0]);

    for (((_, rdomain), (_, idvar)), ((_, idvarval), usubjid)) in
        rdomains.zip(idvars).zip(idvarvals.zip(usubjids.iter()))
    {
        let rdomain = rdomain.trim();
        let idvar = idvar.trim();
        let idvarval = idvarval.trim();
        let usubjid = usubjid.trim();

        // Skip dataset-level relationships (USUBJID and IDVARVAL are null)
        if usubjid.is_empty() && idvarval.is_empty() {
            continue;
        }

        // Skip empty values
        if rdomain.is_empty() || idvar.is_empty() || idvarval.is_empty() {
            continue;
        }

        // Check if reference exists
        if !context.reference_exists(rdomain, idvar, idvarval) {
            let entry = invalid_by_domain
                .entry(rdomain.to_string())
                .or_insert((0, Vec::new()));
            entry.0 += 1;
            if entry.1.len() < 5 {
                entry.1.push(format!("{}={}", idvar, idvarval));
            }
        }
    }

    // Convert to issues
    invalid_by_domain
        .into_iter()
        .map(
            |(rdomain, (count, samples))| Issue::RelrecInvalidReference {
                rdomain,
                invalid_count: count,
                samples,
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn dm_df() -> DataFrame {
        df! {
            "USUBJID" => &["STUDY-001", "STUDY-002", "STUDY-003"],
            "AGE" => &[25, 30, 35],
        }
        .unwrap()
    }

    fn ae_df_valid() -> DataFrame {
        df! {
            "USUBJID" => &["STUDY-001", "STUDY-002", "STUDY-001"],
            "AETERM" => &["HEADACHE", "NAUSEA", "FATIGUE"],
        }
        .unwrap()
    }

    fn ae_df_invalid() -> DataFrame {
        df! {
            "USUBJID" => &["STUDY-001", "STUDY-999", "STUDY-888"],
            "AETERM" => &["HEADACHE", "NAUSEA", "FATIGUE"],
        }
        .unwrap()
    }

    #[test]
    fn test_extract_dm_subjects() {
        let dm = dm_df();
        let subjects = extract_dm_subjects(&dm);

        assert_eq!(subjects.len(), 3);
        assert!(subjects.contains("STUDY-001"));
        assert!(subjects.contains("STUDY-002"));
        assert!(subjects.contains("STUDY-003"));
    }

    #[test]
    fn test_check_usubjid_all_valid() {
        let dm = dm_df();
        let ae = ae_df_valid();
        let dm_subjects = extract_dm_subjects(&dm);

        let issues = check_usubjid_in_dm("AE", &ae, &dm_subjects);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_check_usubjid_some_missing() {
        let dm = dm_df();
        let ae = ae_df_invalid();
        let dm_subjects = extract_dm_subjects(&dm);

        let issues = check_usubjid_in_dm("AE", &ae, &dm_subjects);
        assert_eq!(issues.len(), 1);

        match &issues[0] {
            Issue::UsubjidNotInDm {
                domain,
                missing_count,
                samples,
            } => {
                assert_eq!(domain, "AE");
                assert_eq!(*missing_count, 2);
                assert!(samples.contains(&"STUDY-999".to_string()));
                assert!(samples.contains(&"STUDY-888".to_string()));
            }
            _ => panic!("Expected UsubjidNotInDm issue"),
        }
    }

    #[test]
    fn test_check_usubjid_no_usubjid_column() {
        let dm = dm_df();
        let lb = df! {
            "LBTEST" => &["ALT", "AST"],
        }
        .unwrap();
        let dm_subjects = extract_dm_subjects(&dm);

        let issues = check_usubjid_in_dm("LB", &lb, &dm_subjects);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_extract_dm_subjects_empty() {
        let dm = DataFrame::empty();
        let subjects = extract_dm_subjects(&dm);
        assert!(subjects.is_empty());
    }

    #[test]
    fn test_check_usubjid_empty_subjects() {
        let dm_subjects = HashSet::new();
        let ae = ae_df_valid();

        // All USUBJIDs should be flagged as missing
        let issues = check_usubjid_in_dm("AE", &ae, &dm_subjects);
        assert_eq!(issues.len(), 1);

        match &issues[0] {
            Issue::UsubjidNotInDm { missing_count, .. } => {
                assert_eq!(*missing_count, 3); // All 3 rows have missing USUBJIDs
            }
            _ => panic!("Expected UsubjidNotInDm issue"),
        }
    }

    // =========================================================================
    // RDOMAIN VALIDATION TESTS
    // =========================================================================

    #[test]
    fn test_check_rdomain_valid_all_valid() {
        let co = df! {
            "USUBJID" => &["STUDY-001", "STUDY-002"],
            "RDOMAIN" => &["AE", "LB"],
            "COVAL" => &["Comment 1", "Comment 2"],
        }
        .unwrap();

        let valid_domains: HashSet<String> = ["DM", "AE", "LB", "CO"]
            .into_iter()
            .map(String::from)
            .collect();

        let issues = check_rdomain_valid("CO", &co, &valid_domains);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_check_rdomain_valid_invalid_references() {
        let co = df! {
            "USUBJID" => &["STUDY-001", "STUDY-002", "STUDY-003"],
            "RDOMAIN" => &["AE", "XX", "YY"],
            "COVAL" => &["Comment 1", "Comment 2", "Comment 3"],
        }
        .unwrap();

        let valid_domains: HashSet<String> =
            ["DM", "AE", "LB"].into_iter().map(String::from).collect();

        let issues = check_rdomain_valid("CO", &co, &valid_domains);
        assert_eq!(issues.len(), 1);

        match &issues[0] {
            Issue::InvalidRdomain {
                domain,
                invalid_count,
                samples,
            } => {
                assert_eq!(domain, "CO");
                assert_eq!(*invalid_count, 2);
                assert!(samples.contains(&"XX".to_string()));
                assert!(samples.contains(&"YY".to_string()));
            }
            _ => panic!("Expected InvalidRdomain issue"),
        }
    }

    #[test]
    fn test_check_rdomain_valid_empty_rdomain_allowed() {
        // Empty RDOMAIN is valid for standalone comments
        let co = df! {
            "USUBJID" => &["STUDY-001"],
            "RDOMAIN" => &[""],
            "COVAL" => &["General study comment"],
        }
        .unwrap();

        let valid_domains: HashSet<String> = ["DM", "AE"].into_iter().map(String::from).collect();

        let issues = check_rdomain_valid("CO", &co, &valid_domains);
        assert!(issues.is_empty());
    }

    // =========================================================================
    // RELSUB VALIDATION TESTS
    // =========================================================================

    #[test]
    fn test_check_relsub_all_valid() {
        // Bidirectional twin relationship
        let relsub = df! {
            "USUBJID" => &["STUDY-001", "STUDY-002"],
            "RSUBJID" => &["STUDY-002", "STUDY-001"],
            "SREL" => &["TWIN", "TWIN"],
        }
        .unwrap();

        let dm_subjects: HashSet<String> = ["STUDY-001", "STUDY-002", "STUDY-003"]
            .into_iter()
            .map(String::from)
            .collect();

        let issues = check_relsub(&relsub, &dm_subjects);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_check_relsub_rsubjid_not_in_dm() {
        let relsub = df! {
            "USUBJID" => &["STUDY-001", "STUDY-002"],
            "RSUBJID" => &["STUDY-999", "STUDY-001"],
            "SREL" => &["TWIN", "TWIN"],
        }
        .unwrap();

        let dm_subjects: HashSet<String> = ["STUDY-001", "STUDY-002"]
            .into_iter()
            .map(String::from)
            .collect();

        let issues = check_relsub(&relsub, &dm_subjects);

        // Should have RelsubNotInDm issue
        let rsubjid_issue = issues
            .iter()
            .find(|i| matches!(i, Issue::RelsubNotInDm { .. }));
        assert!(rsubjid_issue.is_some());

        match rsubjid_issue.unwrap() {
            Issue::RelsubNotInDm {
                missing_count,
                samples,
            } => {
                assert_eq!(*missing_count, 1);
                assert!(samples.contains(&"STUDY-999".to_string()));
            }
            _ => panic!("Expected RelsubNotInDm issue"),
        }
    }

    #[test]
    fn test_check_relsub_not_bidirectional() {
        // Unidirectional relationship - missing reciprocal
        let relsub = df! {
            "USUBJID" => &["STUDY-001"],
            "RSUBJID" => &["STUDY-002"],
            "SREL" => &["PARENT"],
        }
        .unwrap();

        let dm_subjects: HashSet<String> = ["STUDY-001", "STUDY-002"]
            .into_iter()
            .map(String::from)
            .collect();

        let issues = check_relsub(&relsub, &dm_subjects);

        let bidirectional_issue = issues
            .iter()
            .find(|i| matches!(i, Issue::RelsubNotBidirectional { .. }));
        assert!(bidirectional_issue.is_some());

        match bidirectional_issue.unwrap() {
            Issue::RelsubNotBidirectional {
                missing_count,
                samples,
            } => {
                assert_eq!(*missing_count, 1);
                assert!(samples.contains(&"STUDY-001→STUDY-002".to_string()));
            }
            _ => panic!("Expected RelsubNotBidirectional issue"),
        }
    }

    // =========================================================================
    // RELSPEC VALIDATION TESTS
    // =========================================================================

    #[test]
    fn test_check_relspec_valid_parent_chain() {
        // Parent specimen → Child specimen
        let relspec = df! {
            "USUBJID" => &["STUDY-001", "STUDY-001"],
            "REFID" => &["SPEC-001", "SPEC-002"],
            "PARENT" => &["", "SPEC-001"],  // SPEC-002 derived from SPEC-001
            "LEVEL" => &[1i32, 2i32],
        }
        .unwrap();

        let issues = check_relspec(&relspec);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_check_relspec_invalid_parent() {
        // PARENT references non-existent REFID
        let relspec = df! {
            "USUBJID" => &["STUDY-001", "STUDY-001"],
            "REFID" => &["SPEC-001", "SPEC-002"],
            "PARENT" => &["", "SPEC-999"],  // SPEC-999 doesn't exist
            "LEVEL" => &[1i32, 2i32],
        }
        .unwrap();

        let issues = check_relspec(&relspec);
        assert_eq!(issues.len(), 1);

        match &issues[0] {
            Issue::RelspecInvalidParent {
                invalid_count,
                samples,
            } => {
                assert_eq!(*invalid_count, 1);
                assert!(samples.contains(&"STUDY-001:SPEC-999".to_string()));
            }
            _ => panic!("Expected RelspecInvalidParent issue"),
        }
    }

    #[test]
    fn test_check_relspec_parent_from_different_subject() {
        // PARENT should be within same subject
        let relspec = df! {
            "USUBJID" => &["STUDY-001", "STUDY-002"],
            "REFID" => &["SPEC-001", "SPEC-002"],
            "PARENT" => &["", "SPEC-001"],  // SPEC-001 is from STUDY-001, not STUDY-002
            "LEVEL" => &[1i32, 2i32],
        }
        .unwrap();

        let issues = check_relspec(&relspec);
        assert_eq!(issues.len(), 1); // Should flag cross-subject reference
    }

    // =========================================================================
    // RELREC VALIDATION TESTS
    // =========================================================================

    #[test]
    fn test_check_relrec_valid_references() {
        let relrec = df! {
            "STUDYID" => &["STUDY", "STUDY"],
            "USUBJID" => &["STUDY-001", "STUDY-001"],
            "RDOMAIN" => &["AE", "CM"],
            "IDVAR" => &["AESEQ", "CMSEQ"],
            "IDVARVAL" => &["1", "1"],
            "RELID" => &["REL1", "REL1"],
        }
        .unwrap();

        let ae = df! {
            "USUBJID" => &["STUDY-001"],
            "AESEQ" => &["1"],
            "AETERM" => &["HEADACHE"],
        }
        .unwrap();

        let cm = df! {
            "USUBJID" => &["STUDY-001"],
            "CMSEQ" => &["1"],
            "CMTRT" => &["ASPIRIN"],
        }
        .unwrap();

        let domains: Vec<(&str, &DataFrame)> = vec![("AE", &ae), ("CM", &cm)];
        let context = RelrecContext::new(&domains);

        let issues = check_relrec(&relrec, &context);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_check_relrec_invalid_reference() {
        let relrec = df! {
            "STUDYID" => &["STUDY"],
            "USUBJID" => &["STUDY-001"],
            "RDOMAIN" => &["AE"],
            "IDVAR" => &["AESEQ"],
            "IDVARVAL" => &["999"],  // Doesn't exist
            "RELID" => &["REL1"],
        }
        .unwrap();

        let ae = df! {
            "USUBJID" => &["STUDY-001"],
            "AESEQ" => &["1"],
            "AETERM" => &["HEADACHE"],
        }
        .unwrap();

        let domains: Vec<(&str, &DataFrame)> = vec![("AE", &ae)];
        let context = RelrecContext::new(&domains);

        let issues = check_relrec(&relrec, &context);
        assert_eq!(issues.len(), 1);

        match &issues[0] {
            Issue::RelrecInvalidReference {
                rdomain,
                invalid_count,
                samples,
            } => {
                assert_eq!(rdomain, "AE");
                assert_eq!(*invalid_count, 1);
                assert!(samples.contains(&"AESEQ=999".to_string()));
            }
            _ => panic!("Expected RelrecInvalidReference issue"),
        }
    }

    #[test]
    fn test_check_relrec_dataset_level_relationship() {
        // Dataset-level relationships have null USUBJID and IDVARVAL
        let relrec = df! {
            "STUDYID" => &["STUDY"],
            "USUBJID" => &[""],  // Null for dataset-level
            "RDOMAIN" => &["SUPPAE"],
            "IDVAR" => &["QNAM"],
            "IDVARVAL" => &[""],  // Null for dataset-level
            "RELID" => &["REL1"],
        }
        .unwrap();

        let domains: Vec<(&str, &DataFrame)> = vec![];
        let context = RelrecContext::new(&domains);

        let issues = check_relrec(&relrec, &context);
        assert!(issues.is_empty()); // Should skip dataset-level relationships
    }
}
