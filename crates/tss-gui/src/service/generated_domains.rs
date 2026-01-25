//! Generated domain creation service.
//!
//! This service generates DataFrames for special-purpose and relationship domains:
//! - **CO (Comments)**: Free-text comments linked to subjects/records
//! - **RELREC (Related Records)**: Links records within/across domains
//! - **RELSPEC (Related Specimens)**: Specimen parent/child hierarchy
//! - **RELSUB (Related Subjects)**: Subject relationships (bidirectional)
//!
//! # CDISC Compliance
//!
//! Generated DataFrames conform to SDTM-IG v3.4 structure:
//! - All required variables are included
//! - Variable order matches the standard
//! - Auto-generated values (STUDYID, DOMAIN, --SEQ) are populated

use std::collections::HashMap;

use polars::prelude::{DataFrame, NamedFrom, Series};
use tss_standards::SdtmDomain;
use tss_standards::sdtm::get_reciprocal_srel;

use crate::state::{
    CommentEntry, GeneratedDomainEntry, GeneratedDomainState, GeneratedDomainType, RelrecEntry,
    RelspecEntry, RelsubEntry,
};

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Errors that can occur during domain generation.
#[derive(Debug, thiserror::Error)]
pub enum GenerationError {
    /// Missing required field.
    #[error("Missing required field: {field} for {domain}")]
    MissingField {
        domain: &'static str,
        field: &'static str,
    },

    /// Invalid entry for domain type.
    #[error("Invalid entry type for {domain} domain")]
    InvalidEntryType { domain: &'static str },

    /// DataFrame construction failed.
    #[error("Failed to build DataFrame: {0}")]
    DataFrameError(String),

    /// Domain definition not found.
    #[error("Domain definition not found: {0}")]
    DomainNotFound(String),
}

// =============================================================================
// MAIN GENERATION FUNCTION
// =============================================================================

/// Generate a relationship domain from entries.
///
/// This is the main entry point for domain generation. It:
/// 1. Validates entries match the domain type
/// 2. Generates the appropriate DataFrame
/// 3. Returns a complete `GeneratedDomainState`
pub fn generate_relationship_domain(
    domain_type: GeneratedDomainType,
    study_id: &str,
    entries: Vec<GeneratedDomainEntry>,
    definition: SdtmDomain,
) -> Result<GeneratedDomainState, GenerationError> {
    let df = match domain_type {
        GeneratedDomainType::Comments => generate_co_dataframe(study_id, &entries)?,
        GeneratedDomainType::RelatedRecords => generate_relrec_dataframe(study_id, &entries)?,
        GeneratedDomainType::RelatedSpecimens => generate_relspec_dataframe(study_id, &entries)?,
        GeneratedDomainType::RelatedSubjects => generate_relsub_dataframe(study_id, &entries)?,
    };

    Ok(GeneratedDomainState::new(
        domain_type,
        df,
        entries,
        definition,
    ))
}

// =============================================================================
// CO (COMMENTS) GENERATION
// =============================================================================

/// Generate CO (Comments) domain DataFrame.
///
/// Per SDTM-IG v3.4 Section 5.1, CO variables include:
/// - STUDYID, DOMAIN, USUBJID, COSEQ (identifiers)
/// - RDOMAIN, IDVAR, IDVARVAL (linking to other records)
/// - COREF, CODTC, COVAL, COEVAL
fn generate_co_dataframe(
    study_id: &str,
    entries: &[GeneratedDomainEntry],
) -> Result<DataFrame, GenerationError> {
    // Extract comment entries
    let comments: Vec<&CommentEntry> = entries
        .iter()
        .filter_map(|e| match e {
            GeneratedDomainEntry::Comment(c) => Some(c),
            _ => None,
        })
        .collect();

    if comments.is_empty() {
        // Return empty DataFrame with correct schema
        return empty_co_dataframe();
    }

    // Build column vectors
    let mut studyid_vec = Vec::with_capacity(comments.len());
    let mut domain_vec = Vec::with_capacity(comments.len());
    let mut usubjid_vec = Vec::with_capacity(comments.len());
    let mut coseq_vec = Vec::with_capacity(comments.len());
    let mut rdomain_vec = Vec::with_capacity(comments.len());
    let mut idvar_vec = Vec::with_capacity(comments.len());
    let mut idvarval_vec = Vec::with_capacity(comments.len());
    let mut coref_vec = Vec::with_capacity(comments.len());
    let mut codtc_vec = Vec::with_capacity(comments.len());
    let mut coval_vec = Vec::with_capacity(comments.len());
    let mut coeval_vec = Vec::with_capacity(comments.len());

    // Track sequence numbers per subject
    let mut seq_counters: HashMap<&str, i32> = HashMap::new();

    for entry in &comments {
        let seq = seq_counters.entry(&entry.usubjid).or_insert(0);
        *seq += 1;

        studyid_vec.push(study_id.to_string());
        domain_vec.push("CO".to_string());
        usubjid_vec.push(entry.usubjid.clone());
        coseq_vec.push(*seq);
        rdomain_vec.push(entry.rdomain.clone().unwrap_or_default());
        idvar_vec.push(entry.idvar.clone().unwrap_or_default());
        idvarval_vec.push(entry.idvarval.clone().unwrap_or_default());
        coref_vec.push(entry.coref.clone().unwrap_or_default());
        codtc_vec.push(entry.codtc.clone().unwrap_or_default());

        // Handle COVAL overflow (max 200 chars per SDTM-IG)
        // For now, we truncate. Full support would split to COVAL1, COVAL2, etc.
        let coval = if entry.comment.len() > 200 {
            entry.comment[..200].to_string()
        } else {
            entry.comment.clone()
        };
        coval_vec.push(coval);

        coeval_vec.push(entry.coeval.clone().unwrap_or_default());
    }

    DataFrame::new(vec![
        Series::new("STUDYID".into(), studyid_vec).into(),
        Series::new("DOMAIN".into(), domain_vec).into(),
        Series::new("USUBJID".into(), usubjid_vec).into(),
        Series::new("COSEQ".into(), coseq_vec).into(),
        Series::new("RDOMAIN".into(), rdomain_vec).into(),
        Series::new("IDVAR".into(), idvar_vec).into(),
        Series::new("IDVARVAL".into(), idvarval_vec).into(),
        Series::new("COREF".into(), coref_vec).into(),
        Series::new("CODTC".into(), codtc_vec).into(),
        Series::new("COVAL".into(), coval_vec).into(),
        Series::new("COEVAL".into(), coeval_vec).into(),
    ])
    .map_err(|e| GenerationError::DataFrameError(e.to_string()))
}

fn empty_co_dataframe() -> Result<DataFrame, GenerationError> {
    DataFrame::new(vec![
        Series::new("STUDYID".into(), Vec::<String>::new()).into(),
        Series::new("DOMAIN".into(), Vec::<String>::new()).into(),
        Series::new("USUBJID".into(), Vec::<String>::new()).into(),
        Series::new("COSEQ".into(), Vec::<i32>::new()).into(),
        Series::new("RDOMAIN".into(), Vec::<String>::new()).into(),
        Series::new("IDVAR".into(), Vec::<String>::new()).into(),
        Series::new("IDVARVAL".into(), Vec::<String>::new()).into(),
        Series::new("COREF".into(), Vec::<String>::new()).into(),
        Series::new("CODTC".into(), Vec::<String>::new()).into(),
        Series::new("COVAL".into(), Vec::<String>::new()).into(),
        Series::new("COEVAL".into(), Vec::<String>::new()).into(),
    ])
    .map_err(|e| GenerationError::DataFrameError(e.to_string()))
}

// =============================================================================
// RELREC (RELATED RECORDS) GENERATION
// =============================================================================

/// Generate RELREC (Related Records) domain DataFrame.
///
/// Per SDTM-IG v3.4 Section 8.2, RELREC variables include:
/// - STUDYID, RDOMAIN, USUBJID (identifiers)
/// - IDVAR, IDVARVAL (record identification)
/// - RELTYPE, RELID (relationship description)
fn generate_relrec_dataframe(
    study_id: &str,
    entries: &[GeneratedDomainEntry],
) -> Result<DataFrame, GenerationError> {
    let relrecs: Vec<&RelrecEntry> = entries
        .iter()
        .filter_map(|e| match e {
            GeneratedDomainEntry::RelatedRecord(r) => Some(r),
            _ => None,
        })
        .collect();

    if relrecs.is_empty() {
        return empty_relrec_dataframe();
    }

    let mut studyid_vec = Vec::with_capacity(relrecs.len());
    let mut rdomain_vec = Vec::with_capacity(relrecs.len());
    let mut usubjid_vec = Vec::with_capacity(relrecs.len());
    let mut idvar_vec = Vec::with_capacity(relrecs.len());
    let mut idvarval_vec = Vec::with_capacity(relrecs.len());
    let mut reltype_vec = Vec::with_capacity(relrecs.len());
    let mut relid_vec = Vec::with_capacity(relrecs.len());

    for entry in &relrecs {
        studyid_vec.push(study_id.to_string());
        rdomain_vec.push(entry.rdomain.clone());
        usubjid_vec.push(entry.usubjid.clone().unwrap_or_default());
        idvar_vec.push(entry.idvar.clone());
        idvarval_vec.push(entry.idvarval.clone().unwrap_or_default());
        reltype_vec.push(
            entry
                .reltype
                .map(|rt| rt.code().to_string())
                .unwrap_or_default(),
        );
        relid_vec.push(entry.relid.clone());
    }

    DataFrame::new(vec![
        Series::new("STUDYID".into(), studyid_vec).into(),
        Series::new("RDOMAIN".into(), rdomain_vec).into(),
        Series::new("USUBJID".into(), usubjid_vec).into(),
        Series::new("IDVAR".into(), idvar_vec).into(),
        Series::new("IDVARVAL".into(), idvarval_vec).into(),
        Series::new("RELTYPE".into(), reltype_vec).into(),
        Series::new("RELID".into(), relid_vec).into(),
    ])
    .map_err(|e| GenerationError::DataFrameError(e.to_string()))
}

fn empty_relrec_dataframe() -> Result<DataFrame, GenerationError> {
    DataFrame::new(vec![
        Series::new("STUDYID".into(), Vec::<String>::new()).into(),
        Series::new("RDOMAIN".into(), Vec::<String>::new()).into(),
        Series::new("USUBJID".into(), Vec::<String>::new()).into(),
        Series::new("IDVAR".into(), Vec::<String>::new()).into(),
        Series::new("IDVARVAL".into(), Vec::<String>::new()).into(),
        Series::new("RELTYPE".into(), Vec::<String>::new()).into(),
        Series::new("RELID".into(), Vec::<String>::new()).into(),
    ])
    .map_err(|e| GenerationError::DataFrameError(e.to_string()))
}

// =============================================================================
// RELSPEC (RELATED SPECIMENS) GENERATION
// =============================================================================

/// Generate RELSPEC (Related Specimens) domain DataFrame.
///
/// Per SDTM-IG v3.4 Section 8.8, RELSPEC variables include:
/// - STUDYID, DOMAIN, USUBJID (identifiers)
/// - REFID (specimen identifier)
/// - SPEC (specimen type)
/// - PARENT (parent specimen REFID)
/// - LEVEL (generation number, auto-calculated)
fn generate_relspec_dataframe(
    study_id: &str,
    entries: &[GeneratedDomainEntry],
) -> Result<DataFrame, GenerationError> {
    let relspecs: Vec<&RelspecEntry> = entries
        .iter()
        .filter_map(|e| match e {
            GeneratedDomainEntry::RelatedSpecimen(r) => Some(r),
            _ => None,
        })
        .collect();

    if relspecs.is_empty() {
        return empty_relspec_dataframe();
    }

    // Build parent -> children map for LEVEL calculation
    let mut parent_map: HashMap<(&str, &str), Option<&str>> = HashMap::new();
    for entry in &relspecs {
        parent_map.insert(
            (entry.usubjid.as_str(), entry.refid.as_str()),
            entry.parent.as_deref(),
        );
    }

    // Calculate LEVEL for each specimen
    fn calculate_level(
        usubjid: &str,
        refid: &str,
        parent_map: &HashMap<(&str, &str), Option<&str>>,
        visited: &mut Vec<String>,
    ) -> i32 {
        if visited.contains(&refid.to_string()) {
            // Circular reference - return 1 as fallback
            return 1;
        }
        visited.push(refid.to_string());

        match parent_map.get(&(usubjid, refid)) {
            Some(Some(parent)) => 1 + calculate_level(usubjid, parent, parent_map, visited),
            _ => 1, // No parent = collected sample = LEVEL 1
        }
    }

    let mut studyid_vec = Vec::with_capacity(relspecs.len());
    let mut domain_vec = Vec::with_capacity(relspecs.len());
    let mut usubjid_vec = Vec::with_capacity(relspecs.len());
    let mut refid_vec = Vec::with_capacity(relspecs.len());
    let mut spec_vec = Vec::with_capacity(relspecs.len());
    let mut parent_vec = Vec::with_capacity(relspecs.len());
    let mut level_vec = Vec::with_capacity(relspecs.len());

    for entry in &relspecs {
        let mut visited = Vec::new();
        let level = calculate_level(&entry.usubjid, &entry.refid, &parent_map, &mut visited);

        studyid_vec.push(study_id.to_string());
        domain_vec.push("RELSPEC".to_string());
        usubjid_vec.push(entry.usubjid.clone());
        refid_vec.push(entry.refid.clone());
        spec_vec.push(entry.spec.clone().unwrap_or_default());
        parent_vec.push(entry.parent.clone().unwrap_or_default());
        level_vec.push(level);
    }

    DataFrame::new(vec![
        Series::new("STUDYID".into(), studyid_vec).into(),
        Series::new("DOMAIN".into(), domain_vec).into(),
        Series::new("USUBJID".into(), usubjid_vec).into(),
        Series::new("REFID".into(), refid_vec).into(),
        Series::new("SPEC".into(), spec_vec).into(),
        Series::new("PARENT".into(), parent_vec).into(),
        Series::new("LEVEL".into(), level_vec).into(),
    ])
    .map_err(|e| GenerationError::DataFrameError(e.to_string()))
}

fn empty_relspec_dataframe() -> Result<DataFrame, GenerationError> {
    DataFrame::new(vec![
        Series::new("STUDYID".into(), Vec::<String>::new()).into(),
        Series::new("DOMAIN".into(), Vec::<String>::new()).into(),
        Series::new("USUBJID".into(), Vec::<String>::new()).into(),
        Series::new("REFID".into(), Vec::<String>::new()).into(),
        Series::new("SPEC".into(), Vec::<String>::new()).into(),
        Series::new("PARENT".into(), Vec::<String>::new()).into(),
        Series::new("LEVEL".into(), Vec::<i32>::new()).into(),
    ])
    .map_err(|e| GenerationError::DataFrameError(e.to_string()))
}

// =============================================================================
// RELSUB (RELATED SUBJECTS) GENERATION
// =============================================================================

/// Generate RELSUB (Related Subjects) domain DataFrame.
///
/// Per SDTM-IG v3.4 Section 8.7:
/// - RELSUB relationships MUST be bidirectional
/// - If A→B exists with SREL=X, then B→A must exist with reciprocal SREL
///
/// This function automatically generates reciprocal relationships.
fn generate_relsub_dataframe(
    study_id: &str,
    entries: &[GeneratedDomainEntry],
) -> Result<DataFrame, GenerationError> {
    let relsubs: Vec<&RelsubEntry> = entries
        .iter()
        .filter_map(|e| match e {
            GeneratedDomainEntry::RelatedSubject(r) => Some(r),
            _ => None,
        })
        .collect();

    if relsubs.is_empty() {
        return empty_relsub_dataframe();
    }

    // Build complete relationship set (including auto-generated reciprocals)
    let mut all_relationships: Vec<(String, String, String)> = Vec::new();
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

    for entry in &relsubs {
        let key = (entry.usubjid.clone(), entry.rsubjid.clone());
        let reverse_key = (entry.rsubjid.clone(), entry.usubjid.clone());

        // Add the original relationship if not seen
        if !seen.contains(&key) {
            all_relationships.push((
                entry.usubjid.clone(),
                entry.rsubjid.clone(),
                entry.srel.clone(),
            ));
            seen.insert(key);
        }

        // Auto-generate reciprocal if not already present
        if !seen.contains(&reverse_key) {
            if let Some(reciprocal_srel) = get_reciprocal_srel(&entry.srel) {
                all_relationships.push((
                    entry.rsubjid.clone(),
                    entry.usubjid.clone(),
                    reciprocal_srel.to_string(),
                ));
                seen.insert(reverse_key);
            }
        }
    }

    let mut studyid_vec = Vec::with_capacity(all_relationships.len());
    let mut domain_vec = Vec::with_capacity(all_relationships.len());
    let mut usubjid_vec = Vec::with_capacity(all_relationships.len());
    let mut rsubjid_vec = Vec::with_capacity(all_relationships.len());
    let mut srel_vec = Vec::with_capacity(all_relationships.len());

    for (usubjid, rsubjid, srel) in &all_relationships {
        studyid_vec.push(study_id.to_string());
        domain_vec.push("RELSUB".to_string());
        usubjid_vec.push(usubjid.clone());
        rsubjid_vec.push(rsubjid.clone());
        srel_vec.push(srel.clone());
    }

    DataFrame::new(vec![
        Series::new("STUDYID".into(), studyid_vec).into(),
        Series::new("DOMAIN".into(), domain_vec).into(),
        Series::new("USUBJID".into(), usubjid_vec).into(),
        Series::new("RSUBJID".into(), rsubjid_vec).into(),
        Series::new("SREL".into(), srel_vec).into(),
    ])
    .map_err(|e| GenerationError::DataFrameError(e.to_string()))
}

fn empty_relsub_dataframe() -> Result<DataFrame, GenerationError> {
    DataFrame::new(vec![
        Series::new("STUDYID".into(), Vec::<String>::new()).into(),
        Series::new("DOMAIN".into(), Vec::<String>::new()).into(),
        Series::new("USUBJID".into(), Vec::<String>::new()).into(),
        Series::new("RSUBJID".into(), Vec::<String>::new()).into(),
        Series::new("SREL".into(), Vec::<String>::new()).into(),
    ])
    .map_err(|e| GenerationError::DataFrameError(e.to_string()))
}

// =============================================================================
// HELPER: GET DOMAIN DEFINITION
// =============================================================================

/// Get the SDTM domain definition for a generated domain type.
///
/// This loads the definition from the standards registry.
pub fn get_domain_definition(
    domain_type: GeneratedDomainType,
    registry: &tss_standards::StandardsRegistry,
) -> Option<SdtmDomain> {
    registry.find_sdtm_domain(domain_type.code()).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_empty_co() {
        let df = generate_co_dataframe("TEST-001", &[]).unwrap();
        assert_eq!(df.height(), 0);
        assert!(df.column("STUDYID").is_ok());
        assert!(df.column("COVAL").is_ok());
    }

    #[test]
    fn test_generate_co_with_entries() {
        let entries = vec![GeneratedDomainEntry::Comment(CommentEntry::standalone(
            "SUBJ-001",
            "Test comment",
        ))];

        let df = generate_co_dataframe("TEST-001", &entries).unwrap();
        assert_eq!(df.height(), 1);
    }

    #[test]
    fn test_relsub_auto_reciprocal() {
        let entries = vec![GeneratedDomainEntry::RelatedSubject(RelsubEntry::new(
            "SUBJ-001",
            "SUBJ-002",
            "MOTHER, BIOLOGICAL",
        ))];

        let df = generate_relsub_dataframe("TEST-001", &entries).unwrap();

        // Should have 2 rows: original + auto-generated reciprocal
        assert_eq!(df.height(), 2);
    }

    #[test]
    fn test_relsub_symmetric_no_duplicate() {
        let entries = vec![GeneratedDomainEntry::RelatedSubject(RelsubEntry::new(
            "SUBJ-001",
            "SUBJ-002",
            "TWIN, DIZYGOTIC",
        ))];

        let df = generate_relsub_dataframe("TEST-001", &entries).unwrap();

        // Symmetric relationship should still create reciprocal
        assert_eq!(df.height(), 2);
    }

    #[test]
    fn test_relspec_level_calculation() {
        let entries = vec![
            // Collected sample (LEVEL 1)
            GeneratedDomainEntry::RelatedSpecimen(RelspecEntry::collected(
                "SUBJ-001",
                "SPC-001",
                Some("BLOOD".to_string()),
            )),
            // Derived from SPC-001 (LEVEL 2)
            GeneratedDomainEntry::RelatedSpecimen(RelspecEntry::derived(
                "SUBJ-001",
                "SPC-001-A",
                "SPC-001",
                Some("SERUM".to_string()),
            )),
        ];

        let df = generate_relspec_dataframe("TEST-001", &entries).unwrap();
        assert_eq!(df.height(), 2);

        // Check LEVEL values
        let level_col = df.column("LEVEL").unwrap();
        let levels: Vec<i32> = level_col.i32().unwrap().into_iter().flatten().collect();

        // First entry (collected) should be LEVEL 1
        // Second entry (derived) should be LEVEL 2
        assert!(levels.contains(&1));
        assert!(levels.contains(&2));
    }
}
