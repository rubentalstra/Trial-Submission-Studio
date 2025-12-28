//! Cross-domain validation rules.
//!
//! Per SDTMIG v3.4 Chapter 4 and Chapter 8, certain validations require
//! examining multiple domains together:
//!
//! - **SEQ uniqueness across split datasets**: A domain split across multiple
//!   files (e.g., LBCH, LBHE) must maintain unique --SEQ values per USUBJID
//!   across all splits.
//!
//! - **SUPPQUAL QNAM uniqueness**: QNAM must be unique within
//!   (STUDYID, RDOMAIN, USUBJID, IDVAR, IDVARVAL). See Section 8.4.
//!
//! - **QVAL non-empty**: QVAL cannot be empty when a SUPPQUAL record exists.
//!
//! - **Relationship key integrity**: RELREC, RELSPEC, RELSUB references must
//!   point to valid records in referenced domains.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use polars::prelude::{AnyValue, DataFrame};

use sdtm_ingest::any_to_string;
use sdtm_model::{CaseInsensitiveLookup, ConformanceIssue, ConformanceReport, IssueSeverity};

// Import rule mappings - use correct P21 rules where they exist,
// internal TRANS_* rules for transpiler-specific validations
use crate::rule_mapping::{
    // P21 Rules (from Rules.csv) - use ONLY when validation matches P21 definition
    P21_SUPP_QNAM_DUPLICATE, // SD0086 - SUPPQUAL duplicate records
    // Internal Rules (not in P21 - use TRANS_* prefix)
    TRANS_CO_IDVAR_INTEGRITY, // CO IDVAR/IDVARVAL referential integrity
    TRANS_RELREC_INTEGRITY,   // RELREC referential integrity
    TRANS_RELSPEC_INTEGRITY,  // RELSPEC structure validation
    TRANS_RELSUB_INTEGRITY,   // RELSUB referential integrity
    TRANS_SEQ_CROSS_SPLIT,    // --SEQ collision across split datasets
    TRANS_SUPP_QVAL_EMPTY,    // QVAL empty in SUPPQUAL
    TRANS_SUPP_TIMING_VAR,    // Timing variable in SUPPQUAL
    TRANS_VARIABLE_PREFIX,    // Variable prefix validation for splits
};

/// Input for cross-domain validation.
pub struct CrossDomainValidationInput<'a> {
    /// All domain frames indexed by domain code (uppercase).
    pub frames: &'a BTreeMap<String, &'a DataFrame>,
    /// Base domain codes (not dataset names) for split detection.
    /// Maps dataset name (e.g., "LBCH") to base domain (e.g., "LB").
    pub split_mappings: Option<&'a BTreeMap<String, String>>,
}

/// Result of cross-domain validation.
#[derive(Debug, Default)]
pub struct CrossDomainValidationResult {
    /// Issues found, grouped by domain code.
    pub issues_by_domain: BTreeMap<String, Vec<ConformanceIssue>>,
    /// Summary counts.
    pub seq_violations: u64,
    pub qnam_violations: u64,
    pub qval_violations: u64,
    pub relrec_violations: u64,
    pub prefix_violations: u64,
}

impl CrossDomainValidationResult {
    /// Convert to conformance reports.
    pub fn into_reports(self) -> Vec<ConformanceReport> {
        self.issues_by_domain
            .into_iter()
            .map(|(domain_code, issues)| ConformanceReport {
                domain_code,
                issues,
            })
            .collect()
    }

    /// Merge issues into existing report map.
    pub fn merge_into(self, reports: &mut BTreeMap<String, ConformanceReport>) {
        for (domain_code, issues) in self.issues_by_domain {
            reports
                .entry(domain_code.clone())
                .or_insert_with(|| ConformanceReport {
                    domain_code,
                    issues: Vec::new(),
                })
                .issues
                .extend(issues);
        }
    }

    /// Check if any violations were found.
    pub fn has_issues(&self) -> bool {
        !self.issues_by_domain.is_empty()
    }

    /// Total issue count.
    pub fn total_issues(&self) -> usize {
        self.issues_by_domain
            .values()
            .map(|issues| issues.len())
            .sum()
    }
}

/// Run all cross-domain validations.
pub fn validate_cross_domain(input: CrossDomainValidationInput<'_>) -> CrossDomainValidationResult {
    let mut result = CrossDomainValidationResult::default();

    // 1. Validate SEQ uniqueness across split datasets
    let seq_result = validate_seq_across_splits(&input);
    result.seq_violations = seq_result.violation_count;
    for (domain, issues) in seq_result.issues {
        result
            .issues_by_domain
            .entry(domain)
            .or_default()
            .extend(issues);
    }

    // 2. Validate SUPPQUAL QNAM uniqueness
    let qnam_result = validate_supp_qnam_uniqueness(input.frames);
    result.qnam_violations = qnam_result.violation_count;
    for (domain, issues) in qnam_result.issues {
        result
            .issues_by_domain
            .entry(domain)
            .or_default()
            .extend(issues);
    }

    // 3. Validate QVAL non-empty
    let qval_result = validate_supp_qval_non_empty(input.frames);
    result.qval_violations = qval_result.violation_count;
    for (domain, issues) in qval_result.issues {
        result
            .issues_by_domain
            .entry(domain)
            .or_default()
            .extend(issues);
    }

    // 4. Validate relationship key integrity
    let rel_result = validate_relationship_integrity(input.frames);
    result.relrec_violations = rel_result.violation_count;
    for (domain, issues) in rel_result.issues {
        result
            .issues_by_domain
            .entry(domain)
            .or_default()
            .extend(issues);
    }

    // 5. Validate variable prefixes follow base domain code for split datasets
    let prefix_result = validate_variable_prefixes(&input);
    result.prefix_violations = prefix_result.violation_count;
    for (domain, issues) in prefix_result.issues {
        result
            .issues_by_domain
            .entry(domain)
            .or_default()
            .extend(issues);
    }

    // 6. Validate no timing variables in SUPPQUAL
    let timing_result = validate_supp_timing_variables(input.frames);
    for (domain, issues) in timing_result.issues {
        result
            .issues_by_domain
            .entry(domain)
            .or_default()
            .extend(issues);
    }

    result
}

// ============================================================================
// Variable Prefix Validation for Split Datasets
// ============================================================================

struct PrefixValidationResult {
    issues: BTreeMap<String, Vec<ConformanceIssue>>,
    violation_count: u64,
}

/// Validate that variable prefixes follow the base DOMAIN code for split datasets.
///
/// Per SDTMIG v3.4 Section 4.1.7: When a domain is split across multiple datasets
/// (e.g., LB split into LBCH, LBHE), variable prefixes must use the base domain
/// code (LB), not the dataset name (LBCH, LBHE).
///
/// Valid: LBSEQ, LBDTC, LBORRES in dataset LBCH
/// Invalid: LBCHSEQ, LBCHDTC (would use dataset name as prefix)
fn validate_variable_prefixes(input: &CrossDomainValidationInput<'_>) -> PrefixValidationResult {
    let mut issues: BTreeMap<String, Vec<ConformanceIssue>> = BTreeMap::new();
    let mut violation_count = 0u64;

    // Get split mappings to identify split datasets
    let Some(split_mappings) = input.split_mappings else {
        return PrefixValidationResult {
            issues,
            violation_count,
        };
    };

    // Check each dataset for incorrect variable prefixes
    for (dataset_name, df) in input.frames {
        // Get the base domain for this dataset
        let base_domain = split_mappings
            .get(dataset_name)
            .cloned()
            .unwrap_or_else(|| infer_base_domain(dataset_name));

        // Skip if not a split dataset (dataset_name == base_domain)
        if dataset_name.eq_ignore_ascii_case(&base_domain) {
            continue;
        }

        // Get column names
        let columns = df.get_column_names_owned();

        // Check for columns that incorrectly use the dataset name as prefix
        let mut invalid_columns: Vec<String> = Vec::new();

        for col in &columns {
            let col_upper = col.to_uppercase();

            // Skip standard identifier columns that don't use domain prefix
            if matches!(
                col_upper.as_str(),
                "STUDYID" | "DOMAIN" | "USUBJID" | "SUBJID" | "VISIT" | "VISITNUM" | "EPOCH"
            ) {
                continue;
            }

            // Check if column uses the full dataset name as prefix
            // E.g., for dataset LBCH, check if column starts with "LBCH"
            if col_upper.starts_with(&dataset_name.to_uppercase())
                && col_upper.len() > dataset_name.len()
            {
                // This column incorrectly uses dataset name as prefix
                // It should use base domain (e.g., LB instead of LBCH)
                invalid_columns.push(col.to_string());
            }
        }

        if !invalid_columns.is_empty() {
            violation_count += invalid_columns.len() as u64;

            let sample_count = invalid_columns.len().min(5);
            let samples: Vec<String> = invalid_columns.iter().take(sample_count).cloned().collect();

            issues
                .entry(dataset_name.clone())
                .or_default()
                .push(ConformanceIssue {
                    code: TRANS_VARIABLE_PREFIX.to_string(),
                    message: format!(
                        "Split dataset {} contains {} variable(s) using dataset name as prefix instead of base domain {}. \
                         Variables should use prefix '{}', not '{}'. Invalid columns: {}",
                        dataset_name,
                        invalid_columns.len(),
                        base_domain,
                        base_domain,
                        dataset_name,
                        samples.join(", ")
                    ),
                    severity: IssueSeverity::Error,
                    variable: None,
                    count: Some(invalid_columns.len() as u64),
                    rule_id: Some(TRANS_VARIABLE_PREFIX.to_string()),
                    category: Some("Naming".to_string()),
                    codelist_code: None,
                    ct_source: None,
                });
        }
    }

    PrefixValidationResult {
        issues,
        violation_count,
    }
}

// ============================================================================
// SEQ Uniqueness Across Split Datasets
// ============================================================================

struct SeqValidationResult {
    issues: BTreeMap<String, Vec<ConformanceIssue>>,
    violation_count: u64,
}

/// Validate that --SEQ values are unique per USUBJID across split datasets.
///
/// Per SDTMIG v3.4 Section 4.1.5: "--SEQ is unique for each record within a
/// domain and USUBJID." When a domain is split (e.g., LB into LBCH, LBHE),
/// SEQ values must remain unique across all splits.
fn validate_seq_across_splits(input: &CrossDomainValidationInput<'_>) -> SeqValidationResult {
    let mut issues: BTreeMap<String, Vec<ConformanceIssue>> = BTreeMap::new();
    let mut violation_count = 0u64;

    // Group frames by base domain
    let split_groups = group_by_base_domain(input.frames, input.split_mappings);

    for (base_domain, dataset_names) in split_groups {
        if dataset_names.len() <= 1 {
            // Not a split domain
            continue;
        }

        // Determine the SEQ column name (--SEQ pattern)
        let seq_column = format!("{}SEQ", base_domain);

        // Collect all (USUBJID, SEQ) pairs across splits
        let mut seen: HashMap<(String, String), Vec<String>> = HashMap::new();

        for dataset_name in &dataset_names {
            let Some(df) = input.frames.get(dataset_name) else {
                continue;
            };

            let lookup = CaseInsensitiveLookup::new(df.get_column_names_owned());
            let usubjid_col = match lookup.get("USUBJID") {
                Some(col) => col,
                None => continue,
            };
            let seq_col = match lookup.get(&seq_column) {
                Some(col) => col,
                None => continue,
            };

            let usubjid_series = match df.column(usubjid_col) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let seq_series = match df.column(seq_col) {
                Ok(s) => s,
                Err(_) => continue,
            };

            for idx in 0..df.height() {
                let usubjid = any_to_string(usubjid_series.get(idx).unwrap_or(AnyValue::Null));
                let seq = any_to_string(seq_series.get(idx).unwrap_or(AnyValue::Null));

                if usubjid.trim().is_empty() || seq.trim().is_empty() {
                    continue;
                }

                let key = (usubjid.trim().to_string(), seq.trim().to_string());
                seen.entry(key).or_default().push(dataset_name.clone());
            }
        }

        // Find duplicates (same USUBJID+SEQ appearing in multiple datasets)
        let mut duplicates: Vec<(String, String, Vec<String>)> = Vec::new();
        for ((usubjid, seq), datasets) in &seen {
            if datasets.len() > 1 {
                duplicates.push((usubjid.clone(), seq.clone(), datasets.clone()));
                violation_count += 1;
            }
        }

        if !duplicates.is_empty() {
            // Report on the base domain
            let sample_count = duplicates.len().min(5);
            let samples: Vec<String> = duplicates
                .iter()
                .take(sample_count)
                .map(|(subj, seq, dsets)| format!("{}:{} in {}", subj, seq, dsets.join(",")))
                .collect();

            issues.entry(base_domain.clone()).or_default().push(ConformanceIssue {
                code: TRANS_SEQ_CROSS_SPLIT.to_string(),
                message: format!(
                    "{}SEQ values are not unique across split datasets ({}). {} duplicate(s) found. Samples: {}",
                    base_domain,
                    dataset_names.join(", "),
                    duplicates.len(),
                    samples.join("; ")
                ),
                severity: IssueSeverity::Error,
                variable: Some(seq_column),
                count: Some(duplicates.len() as u64),
                rule_id: Some(TRANS_SEQ_CROSS_SPLIT.to_string()),
                category: Some("Identifier".to_string()),
                codelist_code: None,
                ct_source: None,
            });
        }
    }

    SeqValidationResult {
        issues,
        violation_count,
    }
}

/// Group datasets by base domain code.
fn group_by_base_domain(
    frames: &BTreeMap<String, &DataFrame>,
    split_mappings: Option<&BTreeMap<String, String>>,
) -> BTreeMap<String, Vec<String>> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for dataset_name in frames.keys() {
        let base = if let Some(mappings) = split_mappings {
            mappings
                .get(dataset_name)
                .cloned()
                .unwrap_or_else(|| infer_base_domain(dataset_name))
        } else {
            infer_base_domain(dataset_name)
        };
        groups.entry(base).or_default().push(dataset_name.clone());
    }

    groups
}

/// Infer base domain from dataset name.
/// E.g., "LBCH" -> "LB", "QSFT" -> "QS", "DM" -> "DM"
fn infer_base_domain(dataset_name: &str) -> String {
    let name = dataset_name.to_uppercase();

    // SUPPXX datasets -> base is the XX part
    if name.starts_with("SUPP") && name.len() > 4 {
        return name[4..].to_string();
    }

    // Special relationship datasets - check these BEFORE 2-letter domain check
    // because RELREC starts with "RE" which is a valid 2-letter domain code
    if name.starts_with("RELREC")
        || name.starts_with("RELSPEC")
        || name.starts_with("RELSUB")
        || name.starts_with("SUPPQUAL")
    {
        return name;
    }

    // Standard 2-letter domains that may have suffixes
    const TWO_LETTER_DOMAINS: &[&str] = &[
        "AE", "AG", "BE", "BS", "CE", "CM", "CO", "CP", "CV", "DA", "DD", "DM", "DS", "DV", "EC",
        "EG", "EX", "FA", "FT", "GF", "HO", "IE", "IS", "LB", "MB", "MH", "MI", "MK", "ML", "MS",
        "NV", "OE", "OI", "PC", "PE", "PP", "PR", "QS", "RE", "RP", "RS", "SC", "SE", "SM", "SR",
        "SS", "SU", "SV", "TA", "TD", "TE", "TI", "TM", "TR", "TS", "TU", "TV", "UR", "VS",
    ];

    // Check if starts with a known 2-letter domain
    if name.len() >= 2 {
        let prefix = &name[..2];
        if TWO_LETTER_DOMAINS.contains(&prefix) {
            return prefix.to_string();
        }
    }

    // Default: return as-is
    name
}

// ============================================================================
// SUPPQUAL QNAM Uniqueness
// ============================================================================

struct QnamValidationResult {
    issues: BTreeMap<String, Vec<ConformanceIssue>>,
    violation_count: u64,
}

/// Validate QNAM uniqueness within SUPPQUAL datasets.
///
/// Per SDTMIG v3.4 Section 8.4: QNAM must be unique within the combination of
/// (STUDYID, RDOMAIN, USUBJID, IDVAR, IDVARVAL).
fn validate_supp_qnam_uniqueness(frames: &BTreeMap<String, &DataFrame>) -> QnamValidationResult {
    let mut issues: BTreeMap<String, Vec<ConformanceIssue>> = BTreeMap::new();
    let mut violation_count = 0u64;

    for (domain_code, df) in frames {
        // Only check SUPP datasets
        if !domain_code.starts_with("SUPP") {
            continue;
        }

        let lookup = CaseInsensitiveLookup::new(df.get_column_names_owned());

        // Get required columns
        let studyid_col = lookup.get("STUDYID");
        let rdomain_col = lookup.get("RDOMAIN");
        let usubjid_col = lookup.get("USUBJID");
        let idvar_col = lookup.get("IDVAR");
        let idvarval_col = lookup.get("IDVARVAL");
        let qnam_col = match lookup.get("QNAM") {
            Some(col) => col,
            None => continue,
        };

        // Build key -> count map
        let mut seen: HashMap<String, u64> = HashMap::new();

        for idx in 0..df.height() {
            let studyid = studyid_col
                .and_then(|col| df.column(col).ok())
                .map(|s| any_to_string(s.get(idx).unwrap_or(AnyValue::Null)))
                .unwrap_or_default();
            let rdomain = rdomain_col
                .and_then(|col| df.column(col).ok())
                .map(|s| any_to_string(s.get(idx).unwrap_or(AnyValue::Null)))
                .unwrap_or_default();
            let usubjid = usubjid_col
                .and_then(|col| df.column(col).ok())
                .map(|s| any_to_string(s.get(idx).unwrap_or(AnyValue::Null)))
                .unwrap_or_default();
            let idvar = idvar_col
                .and_then(|col| df.column(col).ok())
                .map(|s| any_to_string(s.get(idx).unwrap_or(AnyValue::Null)))
                .unwrap_or_default();
            let idvarval = idvarval_col
                .and_then(|col| df.column(col).ok())
                .map(|s| any_to_string(s.get(idx).unwrap_or(AnyValue::Null)))
                .unwrap_or_default();
            let qnam = df
                .column(qnam_col)
                .ok()
                .map(|s| any_to_string(s.get(idx).unwrap_or(AnyValue::Null)))
                .unwrap_or_default();

            // Build composite key
            let key = format!(
                "{}|{}|{}|{}|{}|{}",
                studyid.trim(),
                rdomain.trim(),
                usubjid.trim(),
                idvar.trim(),
                idvarval.trim(),
                qnam.trim()
            );

            *seen.entry(key).or_insert(0) += 1;
        }

        // Find duplicates
        let duplicates: Vec<(&String, &u64)> =
            seen.iter().filter(|(_, count)| **count > 1).collect();

        if !duplicates.is_empty() {
            let total_dups: u64 = duplicates.iter().map(|(_, c)| *c - 1).sum();
            violation_count += total_dups;

            let sample_count = duplicates.len().min(5);
            let samples: Vec<String> = duplicates
                .iter()
                .take(sample_count)
                .map(|(key, count)| format!("{} ({}x)", key, count))
                .collect();

            issues.entry(domain_code.clone()).or_default().push(ConformanceIssue {
                code: P21_SUPP_QNAM_DUPLICATE.to_string(),
                message: format!(
                    "QNAM is not unique within (STUDYID, RDOMAIN, USUBJID, IDVAR, IDVARVAL). {} duplicate key(s) found. Samples: {}",
                    duplicates.len(),
                    samples.join("; ")
                ),
                severity: IssueSeverity::Error,
                variable: Some("QNAM".to_string()),
                count: Some(total_dups),
                rule_id: Some(P21_SUPP_QNAM_DUPLICATE.to_string()),
                category: Some("Uniqueness".to_string()),
                codelist_code: None,
                ct_source: None,
            });
        }
    }

    QnamValidationResult {
        issues,
        violation_count,
    }
}

// ============================================================================
// QVAL Non-Empty Validation
// ============================================================================

struct QvalValidationResult {
    issues: BTreeMap<String, Vec<ConformanceIssue>>,
    violation_count: u64,
}

/// Validate that QVAL is not empty in SUPPQUAL records.
///
/// Per SDTMIG v3.4 Section 8.4: QVAL must contain the actual value being
/// recorded. An empty QVAL makes the SUPPQUAL record meaningless.
fn validate_supp_qval_non_empty(frames: &BTreeMap<String, &DataFrame>) -> QvalValidationResult {
    let mut issues: BTreeMap<String, Vec<ConformanceIssue>> = BTreeMap::new();
    let mut violation_count = 0u64;

    for (domain_code, df) in frames {
        // Only check SUPP datasets
        if !domain_code.starts_with("SUPP") {
            continue;
        }

        let lookup = CaseInsensitiveLookup::new(df.get_column_names_owned());
        let qval_col = match lookup.get("QVAL") {
            Some(col) => col,
            None => continue,
        };

        let qval_series = match df.column(qval_col) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let mut empty_count = 0u64;
        for idx in 0..df.height() {
            let value = any_to_string(qval_series.get(idx).unwrap_or(AnyValue::Null));
            if value.trim().is_empty() {
                empty_count += 1;
            }
        }

        if empty_count > 0 {
            violation_count += empty_count;

            issues
                .entry(domain_code.clone())
                .or_default()
                .push(ConformanceIssue {
                    code: TRANS_SUPP_QVAL_EMPTY.to_string(),
                    message: format!(
                        "QVAL contains {} empty value(s). SUPPQUAL records require non-empty QVAL.",
                        empty_count
                    ),
                    severity: IssueSeverity::Error,
                    variable: Some("QVAL".to_string()),
                    count: Some(empty_count),
                    rule_id: Some(TRANS_SUPP_QVAL_EMPTY.to_string()),
                    category: Some("Completeness".to_string()),
                    codelist_code: None,
                    ct_source: None,
                });
        }
    }

    QvalValidationResult {
        issues,
        violation_count,
    }
}

// ============================================================================
// SUPPQUAL Timing Variable Validation
// ============================================================================

struct TimingValidationResult {
    issues: BTreeMap<String, Vec<ConformanceIssue>>,
}

/// Validate that SUPPQUAL datasets don't contain timing variables.
///
/// Per SDTMIG v3.4 Section 8.4: Timing variables should be included in the
/// parent domain, not as supplemental qualifiers. Common timing variable
/// suffixes include:
/// - --DTC (datetime)
/// - --STDTC, --ENDTC (start/end datetime)
/// - --DY, --STDY, --ENDY (study day)
/// - --DUR (duration)
/// - --TPT (timepoint)
fn validate_supp_timing_variables(frames: &BTreeMap<String, &DataFrame>) -> TimingValidationResult {
    let mut issues: BTreeMap<String, Vec<ConformanceIssue>> = BTreeMap::new();

    // Timing variable suffixes that should not be in SUPPQUAL
    const TIMING_SUFFIXES: &[&str] = &[
        "DTC", "STDTC", "ENDTC", "DY", "STDY", "ENDY", "DUR", "TPT", "TPTNUM", "ELTM", "TPTREF",
    ];

    for (domain_code, df) in frames {
        // Only check SUPP datasets
        if !domain_code.starts_with("SUPP") {
            continue;
        }

        let lookup = CaseInsensitiveLookup::new(df.get_column_names_owned());
        let qnam_col = match lookup.get("QNAM") {
            Some(col) => col,
            None => continue,
        };

        let qnam_series = match df.column(qnam_col) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Collect unique QNAMs that look like timing variables
        let mut timing_qnams: BTreeSet<String> = BTreeSet::new();

        for idx in 0..df.height() {
            let qnam = any_to_string(qnam_series.get(idx).unwrap_or(AnyValue::Null));
            let qnam_upper = qnam.trim().to_uppercase();

            if qnam_upper.is_empty() {
                continue;
            }

            // Check if QNAM ends with any timing suffix
            for suffix in TIMING_SUFFIXES {
                if qnam_upper.ends_with(suffix) {
                    timing_qnams.insert(qnam.trim().to_string());
                    break;
                }
            }
        }

        if !timing_qnams.is_empty() {
            let timing_list: Vec<String> = timing_qnams.into_iter().collect();
            issues
                .entry(domain_code.clone())
                .or_default()
                .push(ConformanceIssue {
                    code: TRANS_SUPP_TIMING_VAR.to_string(),
                    message: format!(
                        "SUPPQUAL contains timing variable(s): {}. Timing variables should be in parent domain.",
                        timing_list.join(", ")
                    ),
                    severity: IssueSeverity::Warning,
                    variable: Some("QNAM".to_string()),
                    count: Some(timing_list.len() as u64),
                    rule_id: Some(TRANS_SUPP_TIMING_VAR.to_string()),
                    category: Some("Structure".to_string()),
                    codelist_code: None,
                    ct_source: None,
                });
        }
    }

    TimingValidationResult { issues }
}

// ============================================================================
// Relationship Key Integrity
// ============================================================================

struct RelationshipValidationResult {
    issues: BTreeMap<String, Vec<ConformanceIssue>>,
    violation_count: u64,
}

/// Validate relationship dataset key integrity.
///
/// Per SDTMIG v3.4 Chapter 8:
/// - RELREC must reference valid records in the specified domains
/// - RELSPEC must reference valid specimen identifiers
/// - RELSUB must reference valid subject identifiers
fn validate_relationship_integrity(
    frames: &BTreeMap<String, &DataFrame>,
) -> RelationshipValidationResult {
    let mut issues: BTreeMap<String, Vec<ConformanceIssue>> = BTreeMap::new();
    let mut violation_count = 0u64;

    // Validate RELREC
    if let Some(relrec_df) = frames.get("RELREC") {
        let result = validate_relrec_integrity(relrec_df, frames);
        violation_count += result.violation_count;
        for (domain, domain_issues) in result.issues {
            issues.entry(domain).or_default().extend(domain_issues);
        }
    }

    // Validate RELSPEC (if present)
    if let Some(relspec_df) = frames.get("RELSPEC") {
        let result = validate_relspec_integrity(relspec_df, frames);
        violation_count += result.violation_count;
        for (domain, domain_issues) in result.issues {
            issues.entry(domain).or_default().extend(domain_issues);
        }
    }

    // Validate RELSUB (if present)
    if let Some(relsub_df) = frames.get("RELSUB") {
        let result = validate_relsub_integrity(relsub_df, frames);
        violation_count += result.violation_count;
        for (domain, domain_issues) in result.issues {
            issues.entry(domain).or_default().extend(domain_issues);
        }
    }

    // Validate CO (Comments) IDVAR/IDVARVAL references (if present)
    if let Some(co_df) = frames.get("CO") {
        let result = validate_co_idvar_integrity(co_df, frames);
        violation_count += result.violation_count;
        for (domain, domain_issues) in result.issues {
            issues.entry(domain).or_default().extend(domain_issues);
        }
    }

    RelationshipValidationResult {
        issues,
        violation_count,
    }
}

struct IntegrityResult {
    issues: BTreeMap<String, Vec<ConformanceIssue>>,
    violation_count: u64,
}

/// Validate RELREC references point to valid records.
fn validate_relrec_integrity(
    relrec_df: &DataFrame,
    frames: &BTreeMap<String, &DataFrame>,
) -> IntegrityResult {
    let mut issues: BTreeMap<String, Vec<ConformanceIssue>> = BTreeMap::new();
    let mut violation_count = 0u64;

    let lookup = CaseInsensitiveLookup::new(relrec_df.get_column_names_owned());

    // Get RELREC columns
    let rdomain_col = lookup.get("RDOMAIN");
    // Note: IDVAR/IDVARVAL would be used for more detailed integrity checks
    let _idvar_col = lookup.get("IDVAR");
    let _idvarval_col = lookup.get("IDVARVAL");
    let usubjid_col = lookup.get("USUBJID");

    // Build index of valid keys per domain
    let mut domain_keys: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (domain_code, df) in frames {
        if domain_code == "RELREC" || domain_code.starts_with("SUPP") {
            continue;
        }
        let df_lookup = CaseInsensitiveLookup::new(df.get_column_names_owned());

        // Index by USUBJID for basic integrity check
        if let Some(usubjid) = df_lookup.get("USUBJID")
            && let Ok(series) = df.column(usubjid)
        {
            let keys: BTreeSet<String> = (0..df.height())
                .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                .filter(|v| !v.trim().is_empty())
                .collect();
            domain_keys.insert(domain_code.to_uppercase(), keys);
        }
    }

    // Check each RELREC record
    let mut invalid_refs = 0u64;
    let mut invalid_samples: Vec<String> = Vec::new();

    for idx in 0..relrec_df.height() {
        let rdomain = rdomain_col
            .and_then(|col| relrec_df.column(col).ok())
            .map(|s| any_to_string(s.get(idx).unwrap_or(AnyValue::Null)))
            .unwrap_or_default()
            .trim()
            .to_uppercase();

        if rdomain.is_empty() {
            continue;
        }

        // Check if referenced domain exists
        if !domain_keys.contains_key(&rdomain) && !frames.contains_key(&rdomain) {
            invalid_refs += 1;
            if invalid_samples.len() < 5 {
                invalid_samples.push(format!("RDOMAIN={} (domain not found)", rdomain));
            }
            continue;
        }

        // If USUBJID-based check
        if let Some(usubjid) = usubjid_col.and_then(|col| relrec_df.column(col).ok()) {
            let subj = any_to_string(usubjid.get(idx).unwrap_or(AnyValue::Null));
            if !subj.trim().is_empty()
                && let Some(valid_keys) = domain_keys.get(&rdomain)
                && !valid_keys.contains(subj.trim())
            {
                invalid_refs += 1;
                if invalid_samples.len() < 5 {
                    invalid_samples
                        .push(format!("RDOMAIN={}, USUBJID={} (not found)", rdomain, subj));
                }
            }
        }
    }

    if invalid_refs > 0 {
        violation_count += invalid_refs;
        issues
            .entry("RELREC".to_string())
            .or_default()
            .push(ConformanceIssue {
                code: TRANS_RELREC_INTEGRITY.to_string(),
                message: format!(
                    "RELREC contains {} reference(s) to non-existent records. Samples: {}",
                    invalid_refs,
                    invalid_samples.join("; ")
                ),
                severity: IssueSeverity::Error,
                variable: Some("RDOMAIN".to_string()),
                count: Some(invalid_refs),
                rule_id: Some(TRANS_RELREC_INTEGRITY.to_string()),
                category: Some("Referential Integrity".to_string()),
                codelist_code: None,
                ct_source: None,
            });
    }

    IntegrityResult {
        issues,
        violation_count,
    }
}

/// Validate RELSPEC references point to valid specimens.
fn validate_relspec_integrity(
    relspec_df: &DataFrame,
    _frames: &BTreeMap<String, &DataFrame>,
) -> IntegrityResult {
    let mut issues: BTreeMap<String, Vec<ConformanceIssue>> = BTreeMap::new();
    let violation_count = 0u64;

    let lookup = CaseInsensitiveLookup::new(relspec_df.get_column_names_owned());

    // Check that required columns exist
    let required = ["STUDYID", "USUBJID", "SPEC", "SPTYPE"];
    let mut missing: Vec<&str> = Vec::new();
    for col in required {
        if lookup.get(col).is_none() {
            missing.push(col);
        }
    }

    if !missing.is_empty() {
        issues
            .entry("RELSPEC".to_string())
            .or_default()
            .push(ConformanceIssue {
                code: TRANS_RELSPEC_INTEGRITY.to_string(),
                message: format!(
                    "RELSPEC is missing required columns: {}",
                    missing.join(", ")
                ),
                severity: IssueSeverity::Error,
                variable: None,
                count: Some(missing.len() as u64),
                rule_id: Some(TRANS_RELSPEC_INTEGRITY.to_string()),
                category: Some("Structure".to_string()),
                codelist_code: None,
                ct_source: None,
            });
    }

    // Note: Full specimen ID validation would require checking against BS/parent domains
    // This is a structural check only

    IntegrityResult {
        issues,
        violation_count,
    }
}

/// Validate RELSUB references point to valid subjects.
fn validate_relsub_integrity(
    relsub_df: &DataFrame,
    frames: &BTreeMap<String, &DataFrame>,
) -> IntegrityResult {
    let mut issues: BTreeMap<String, Vec<ConformanceIssue>> = BTreeMap::new();
    let mut violation_count = 0u64;

    let lookup = CaseInsensitiveLookup::new(relsub_df.get_column_names_owned());

    // Get RELSUB columns
    let usubjid_col = match lookup.get("USUBJID") {
        Some(col) => col,
        None => {
            issues
                .entry("RELSUB".to_string())
                .or_default()
                .push(ConformanceIssue {
                    code: TRANS_RELSUB_INTEGRITY.to_string(),
                    message: "RELSUB is missing required USUBJID column".to_string(),
                    severity: IssueSeverity::Error,
                    variable: Some("USUBJID".to_string()),
                    count: Some(1),
                    rule_id: Some(TRANS_RELSUB_INTEGRITY.to_string()),
                    category: Some("Structure".to_string()),
                    codelist_code: None,
                    ct_source: None,
                });
            return IntegrityResult {
                issues,
                violation_count: 1,
            };
        }
    };

    // Build set of valid USUBJIDs from DM
    let valid_subjects: BTreeSet<String> = if let Some(dm_df) = frames.get("DM") {
        let dm_lookup = CaseInsensitiveLookup::new(dm_df.get_column_names_owned());
        if let Some(dm_usubjid) = dm_lookup.get("USUBJID") {
            if let Ok(series) = dm_df.column(dm_usubjid) {
                (0..dm_df.height())
                    .map(|idx| {
                        any_to_string(series.get(idx).unwrap_or(AnyValue::Null))
                            .trim()
                            .to_string()
                    })
                    .filter(|v| !v.is_empty())
                    .collect()
            } else {
                BTreeSet::new()
            }
        } else {
            BTreeSet::new()
        }
    } else {
        // No DM dataset - cannot validate subjects
        return IntegrityResult {
            issues,
            violation_count: 0,
        };
    };

    // Check each RELSUB record
    let usubjid_series = match relsub_df.column(usubjid_col) {
        Ok(s) => s,
        Err(_) => {
            return IntegrityResult {
                issues,
                violation_count: 0,
            };
        }
    };

    let mut invalid_subjects = 0u64;
    let mut invalid_samples: Vec<String> = Vec::new();

    for idx in 0..relsub_df.height() {
        let usubjid = any_to_string(usubjid_series.get(idx).unwrap_or(AnyValue::Null));
        let trimmed = usubjid.trim();

        if trimmed.is_empty() {
            continue;
        }

        if !valid_subjects.contains(trimmed) {
            invalid_subjects += 1;
            if invalid_samples.len() < 5 {
                invalid_samples.push(trimmed.to_string());
            }
        }
    }

    if invalid_subjects > 0 {
        violation_count += invalid_subjects;
        issues
            .entry("RELSUB".to_string())
            .or_default()
            .push(ConformanceIssue {
                code: TRANS_RELSUB_INTEGRITY.to_string(),
                message: format!(
                    "RELSUB contains {} reference(s) to non-existent subjects. Samples: {}",
                    invalid_subjects,
                    invalid_samples.join(", ")
                ),
                severity: IssueSeverity::Error,
                variable: Some("USUBJID".to_string()),
                count: Some(invalid_subjects),
                rule_id: Some(TRANS_RELSUB_INTEGRITY.to_string()),
                category: Some("Referential Integrity".to_string()),
                codelist_code: None,
                ct_source: None,
            });
    }

    IntegrityResult {
        issues,
        violation_count,
    }
}

/// Validate CO (Comments) IDVAR/IDVARVAL references point to valid records.
///
/// Per SDTMIG v3.4 Section 8.5, the CO domain uses:
/// - RDOMAIN: The domain code being referenced (e.g., "AE", "CM")
/// - IDVAR: The identifying variable in the referenced domain (usually --SEQ)
/// - IDVARVAL: The value of IDVAR that identifies the specific record
///
/// This validation checks that RDOMAIN/IDVAR/IDVARVAL combinations reference
/// records that actually exist in the referenced domains.
fn validate_co_idvar_integrity(
    co_df: &DataFrame,
    frames: &BTreeMap<String, &DataFrame>,
) -> IntegrityResult {
    let mut issues: BTreeMap<String, Vec<ConformanceIssue>> = BTreeMap::new();
    let mut violation_count = 0u64;

    let lookup = CaseInsensitiveLookup::new(co_df.get_column_names_owned());

    // Get CO columns
    let rdomain_col = match lookup.get("RDOMAIN") {
        Some(col) => col,
        None => {
            // RDOMAIN is optional in CO - if not present, skip validation
            return IntegrityResult {
                issues,
                violation_count: 0,
            };
        }
    };

    let idvar_col = match lookup.get("IDVAR") {
        Some(col) => col,
        None => {
            return IntegrityResult {
                issues,
                violation_count: 0,
            };
        }
    };

    let idvarval_col = match lookup.get("IDVARVAL") {
        Some(col) => col,
        None => {
            return IntegrityResult {
                issues,
                violation_count: 0,
            };
        }
    };

    // Get series
    let rdomain_series = match co_df.column(rdomain_col) {
        Ok(s) => s,
        Err(_) => {
            return IntegrityResult {
                issues,
                violation_count: 0,
            };
        }
    };
    let idvar_series = match co_df.column(idvar_col) {
        Ok(s) => s,
        Err(_) => {
            return IntegrityResult {
                issues,
                violation_count: 0,
            };
        }
    };
    let idvarval_series = match co_df.column(idvarval_col) {
        Ok(s) => s,
        Err(_) => {
            return IntegrityResult {
                issues,
                violation_count: 0,
            };
        }
    };

    // Build lookup of valid values by (domain, idvar) -> set of idvarval values
    let mut valid_refs: HashMap<(String, String), BTreeSet<String>> = HashMap::new();

    for (domain_code, domain_df) in frames {
        let domain_lookup = CaseInsensitiveLookup::new(domain_df.get_column_names_owned());

        // Look for --SEQ variables (most common IDVAR)
        for col_name in domain_df.get_column_names_owned() {
            let upper = col_name.to_uppercase();
            if upper.ends_with("SEQ") {
                if let Ok(series) = domain_df.column(&col_name) {
                    let values: BTreeSet<String> = (0..domain_df.height())
                        .map(|idx| {
                            any_to_string(series.get(idx).unwrap_or(AnyValue::Null))
                                .trim()
                                .to_string()
                        })
                        .filter(|v| !v.is_empty())
                        .collect();
                    let key = (domain_code.clone(), upper);
                    valid_refs.insert(key, values);
                }
            }
        }

        // Also check for USUBJID as potential IDVAR
        if let Some(usubjid_col) = domain_lookup.get("USUBJID") {
            if let Ok(series) = domain_df.column(usubjid_col) {
                let values: BTreeSet<String> = (0..domain_df.height())
                    .map(|idx| {
                        any_to_string(series.get(idx).unwrap_or(AnyValue::Null))
                            .trim()
                            .to_string()
                    })
                    .filter(|v| !v.is_empty())
                    .collect();
                let key = (domain_code.clone(), "USUBJID".to_string());
                valid_refs.insert(key, values);
            }
        }
    }

    // Check each CO record
    let mut invalid_refs = 0u64;
    let mut invalid_samples: Vec<String> = Vec::new();

    for idx in 0..co_df.height() {
        let rdomain = any_to_string(rdomain_series.get(idx).unwrap_or(AnyValue::Null));
        let idvar = any_to_string(idvar_series.get(idx).unwrap_or(AnyValue::Null));
        let idvarval = any_to_string(idvarval_series.get(idx).unwrap_or(AnyValue::Null));

        let rdomain_trimmed = rdomain.trim().to_uppercase();
        let idvar_trimmed = idvar.trim().to_uppercase();
        let idvarval_trimmed = idvarval.trim();

        // Skip if any key field is empty
        if rdomain_trimmed.is_empty() || idvar_trimmed.is_empty() || idvarval_trimmed.is_empty() {
            continue;
        }

        // Check if referenced domain exists and has the referenced value
        let key = (rdomain_trimmed.clone(), idvar_trimmed.clone());
        let is_valid = valid_refs
            .get(&key)
            .map(|values| values.contains(idvarval_trimmed))
            .unwrap_or(false);

        if !is_valid {
            invalid_refs += 1;
            if invalid_samples.len() < 5 {
                invalid_samples.push(format!(
                    "{}:{}.{}={}",
                    rdomain_trimmed, idvar_trimmed, idvarval_trimmed, idvarval_trimmed
                ));
            }
        }
    }

    if invalid_refs > 0 {
        violation_count += invalid_refs;
        issues
            .entry("CO".to_string())
            .or_default()
            .push(ConformanceIssue {
                code: TRANS_CO_IDVAR_INTEGRITY.to_string(),
                message: format!(
                    "CO contains {} reference(s) to non-existent records. Samples: {}",
                    invalid_refs,
                    invalid_samples.join(", ")
                ),
                severity: IssueSeverity::Error,
                variable: Some("IDVARVAL".to_string()),
                count: Some(invalid_refs),
                rule_id: Some(TRANS_CO_IDVAR_INTEGRITY.to_string()),
                category: Some("Referential Integrity".to_string()),
                codelist_code: None,
                ct_source: None,
            });
    }

    IntegrityResult {
        issues,
        violation_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::{Column, NamedFrom, Series};

    fn make_df(columns: Vec<(&str, Vec<&str>)>) -> DataFrame {
        let cols: Vec<Column> = columns
            .into_iter()
            .map(|(name, values)| {
                Series::new(
                    name.into(),
                    values.into_iter().map(String::from).collect::<Vec<_>>(),
                )
                .into()
            })
            .collect();
        DataFrame::new(cols).unwrap()
    }

    #[test]
    fn test_infer_base_domain() {
        assert_eq!(infer_base_domain("LB"), "LB");
        assert_eq!(infer_base_domain("LBCH"), "LB");
        assert_eq!(infer_base_domain("LBHE"), "LB");
        assert_eq!(infer_base_domain("QS"), "QS");
        assert_eq!(infer_base_domain("QSFT"), "QS");
        assert_eq!(infer_base_domain("DM"), "DM");
        assert_eq!(infer_base_domain("SUPPDM"), "DM");
        assert_eq!(infer_base_domain("SUPPLB"), "LB");
        assert_eq!(infer_base_domain("RELREC"), "RELREC");
    }

    #[test]
    fn test_validate_supp_qval_non_empty() {
        let df = make_df(vec![
            ("QNAM", vec!["TEST1", "TEST2", "TEST3"]),
            ("QVAL", vec!["value1", "", "value3"]),
        ]);

        let mut frames = BTreeMap::new();
        frames.insert("SUPPDM".to_string(), &df);

        let result = validate_supp_qval_non_empty(&frames);
        assert_eq!(result.violation_count, 1);
        assert!(result.issues.contains_key("SUPPDM"));
    }

    #[test]
    fn test_validate_supp_qnam_uniqueness() {
        let df = make_df(vec![
            ("STUDYID", vec!["STUDY1", "STUDY1", "STUDY1"]),
            ("RDOMAIN", vec!["DM", "DM", "DM"]),
            ("USUBJID", vec!["SUBJ1", "SUBJ1", "SUBJ2"]),
            ("IDVAR", vec!["", "", ""]),
            ("IDVARVAL", vec!["", "", ""]),
            ("QNAM", vec!["AGE", "AGE", "AGE"]), // Duplicate for SUBJ1
            ("QVAL", vec!["30", "31", "25"]),
        ]);

        let mut frames = BTreeMap::new();
        frames.insert("SUPPDM".to_string(), &df);

        let result = validate_supp_qnam_uniqueness(&frames);
        assert_eq!(result.violation_count, 1); // One duplicate (SUBJ1 has 2 AGE records)
        assert!(result.issues.contains_key("SUPPDM"));
    }
}
