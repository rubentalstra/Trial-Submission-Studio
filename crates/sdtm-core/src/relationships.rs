use std::collections::BTreeMap;

use anyhow::Result;
use polars::prelude::{AnyValue, DataFrame};

use sdtm_model::Domain;

use crate::data_utils::column_value_string;
use crate::domain_sets::domain_map_by_code;
use crate::domain_utils::{StandardColumns, refid_candidates, standard_columns};
use crate::frame::DomainFrame;
use crate::frame_builder::build_domain_frame_from_records;
use sdtm_ingest::any_to_string;

/// Configuration options for relationship generation.
#[derive(Debug, Clone, Default)]
pub struct RelationshipConfig {
    /// If true, skip automatic RELREC generation.
    /// Per SDTMIG 8.2, RELREC should only be generated from explicit relationship keys.
    pub disable_auto_relrec: bool,
    /// If true, include GRPID in RELREC generation (not recommended per SDTMIG).
    /// Per SDTMIG 8.1, --GRPID is for grouping records within a domain,
    /// not for cross-domain relationships.
    pub include_grpid_in_relrec: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LinkKind {
    /// --LNKID: Explicit cross-domain link identifier
    LnkId,
    /// --LNKGRP: Cross-domain link group identifier
    LnkGrp,
    /// --GRPID: Within-domain grouping (should NOT be used for RELREC per SDTMIG 8.1)
    GrpId,
}

impl LinkKind {
    fn suffix(self) -> &'static str {
        match self {
            LinkKind::LnkId => "LNKID",
            LinkKind::LnkGrp => "LNKGRP",
            LinkKind::GrpId => "GRPID",
        }
    }
}

#[derive(Debug, Clone)]
struct LinkIdentifier {
    name: String,
    kind: LinkKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RelrecKey {
    kind: LinkKind,
    usubjid: String,
    idvarval: String,
}

#[derive(Debug, Clone)]
struct RelrecMember {
    domain_code: String,
    usubjid: String,
    idvar: String,
    idvarval: String,
}

struct RelrecRecordInput<'a> {
    rdomain: &'a str,
    usubjid: &'a str,
    idvar: &'a str,
    idvarval: &'a str,
    relid: &'a str,
    reltype: Option<&'a str>,
}

/// Build RELREC dataset from domain frames.
///
/// Per SDTMIG v3.4 Section 8.2:
/// - RELREC represents relationships between records across domains
/// - Only explicit relationship keys (--LNKID, --LNKGRP) should be used
/// - --GRPID is for within-domain grouping and should NOT be used for RELREC
///
/// Per SDTMIG v3.4 Section 8.3:
/// - RELTYPE should only be set for dataset-level relationships
/// - For record-level links (with IDVAR/IDVARVAL), RELTYPE should be blank
///
/// # Arguments
/// * `domain_frames` - The domain frames to process for relationships
/// * `domains` - The domain definitions from standards
/// * `relrec_domain` - The RELREC domain definition
/// * `study_id` - The study identifier
/// * `config` - Configuration options for RELREC generation
///
/// # Configuration
/// * `disable_auto_relrec` - If true, skip RELREC generation entirely
/// * `include_grpid_in_relrec` - If true, include GRPID (not recommended per SDTMIG)
pub fn build_relrec(
    domain_frames: &[DomainFrame],
    domains: &[Domain],
    relrec_domain: &Domain,
    study_id: &str,
    config: &RelationshipConfig,
) -> Result<Option<DomainFrame>> {
    // Check if auto RELREC is disabled
    if config.disable_auto_relrec {
        return Ok(None);
    }

    let domain_map = domain_map_by_code(domains);
    let mut groups: BTreeMap<RelrecKey, Vec<RelrecMember>> = BTreeMap::new();
    for frame in domain_frames {
        if frame.data.height() == 0 {
            continue;
        }
        // Per SDTMIG 8.5: CO (Comments) domain uses RDOMAIN/IDVAR/IDVARVAL to link
        // comments to records in other domains. This is its own linking mechanism,
        // so we should not generate RELREC entries for CO domain.
        if frame.domain_code.eq_ignore_ascii_case("CO") {
            continue;
        }
        let domain_def = match domain_map.get(&frame.domain_code.to_uppercase()) {
            Some(domain) => domain,
            None => {
                return Err(anyhow::anyhow!(
                    "missing standards metadata for domain {}",
                    frame.domain_code
                ));
            }
        };
        let domain_columns = standard_columns(domain_def);
        let usubjid_col = match domain_columns.usubjid.as_ref() {
            Some(name) => name.clone(),
            None => continue,
        };
        if frame.data.column(&usubjid_col).is_err() {
            continue;
        }
        let Some(link) = infer_link_idvar_for_relrec(domain_def, &frame.data, config) else {
            continue;
        };
        let Some(usubjid_vals) = column_trimmed_values(&frame.data, &usubjid_col) else {
            continue;
        };
        let Some(idvar_vals) = column_trimmed_values(&frame.data, &link.name) else {
            continue;
        };
        for idx in 0..frame.data.height() {
            let usubjid = usubjid_vals[idx].clone();
            let idvarval = idvar_vals[idx].clone();
            if idvarval.is_empty() {
                continue;
            }
            let key = RelrecKey {
                kind: link.kind,
                usubjid: usubjid.clone(),
                idvarval: idvarval.clone(),
            };
            groups.entry(key).or_default().push(RelrecMember {
                domain_code: frame.domain_code.to_uppercase(),
                usubjid,
                idvar: link.name.clone(),
                idvarval,
            });
        }
    }

    let mut records: Vec<BTreeMap<String, String>> = Vec::new();
    let mut rel_counter = 0usize;
    for (_key, members) in groups {
        let mut domain_counts: BTreeMap<String, usize> = BTreeMap::new();
        for member in &members {
            *domain_counts.entry(member.domain_code.clone()).or_insert(0) += 1;
        }
        if domain_counts.len() < 2 {
            continue;
        }
        rel_counter += 1;
        let relid = format!("REL{:05}", rel_counter);
        for member in members {
            // Per SDTMIG 8.3: RELTYPE is only used for dataset-level relationships
            // (where IDVAR and IDVARVAL are empty). For record-level relationships,
            // RELTYPE should be blank.
            // Since we have IDVAR/IDVARVAL populated (record-level), leave RELTYPE blank.
            let reltype: Option<&str> = None;
            records.push(relrec_record(
                relrec_domain,
                study_id,
                RelrecRecordInput {
                    rdomain: &member.domain_code,
                    usubjid: &member.usubjid,
                    idvar: &member.idvar,
                    idvarval: &member.idvarval,
                    relid: &relid,
                    reltype,
                },
            ));
        }
    }

    if records.is_empty() {
        return Ok(None);
    }

    let data = build_domain_frame_from_records(relrec_domain, &records)?;
    Ok(Some(DomainFrame::new(relrec_domain.code.clone(), data)))
}

/// Infer link identifier variable for RELREC generation.
///
/// Per SDTMIG 8.1: --GRPID is for within-domain grouping only.
/// Only --LNKID and --LNKGRP should be used for cross-domain RELREC.
fn infer_link_idvar_for_relrec(
    domain: &Domain,
    df: &DataFrame,
    config: &RelationshipConfig,
) -> Option<LinkIdentifier> {
    // First, try explicit cross-domain link identifiers (LNKID, LNKGRP)
    for kind in [LinkKind::LnkId, LinkKind::LnkGrp] {
        if let Some(name) = find_suffix_column(domain, df, kind.suffix()) {
            return Some(LinkIdentifier { name, kind });
        }
    }

    // Only include GRPID if explicitly configured (not recommended per SDTMIG)
    if config.include_grpid_in_relrec
        && let Some(name) = find_suffix_column(domain, df, LinkKind::GrpId.suffix())
    {
        return Some(LinkIdentifier {
            name,
            kind: LinkKind::GrpId,
        });
    }

    None
}

fn find_suffix_column(domain: &Domain, df: &DataFrame, suffix: &str) -> Option<String> {
    let mut candidates: Vec<String> = domain
        .variables
        .iter()
        .map(|var| var.name.clone())
        .filter(|name| name.to_uppercase().ends_with(suffix))
        .filter(|name| df.column(name).is_ok())
        .collect();
    candidates.sort_by_key(|a| a.to_uppercase());
    candidates
        .into_iter()
        .find(|name| column_has_values(df, name))
}

fn column_trimmed_values(df: &DataFrame, name: &str) -> Option<Vec<String>> {
    let Ok(series) = df.column(name) else {
        return None;
    };
    let mut values = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        values.push(value.trim().to_string());
    }
    Some(values)
}

fn column_has_values(df: &DataFrame, name: &str) -> bool {
    let Ok(series) = df.column(name) else {
        return false;
    };
    for idx in 0..df.height() {
        let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        if !value.trim().is_empty() {
            return true;
        }
    }
    false
}

pub fn build_relationship_frames(
    domain_frames: &[DomainFrame],
    domains: &[Domain],
    study_id: &str,
) -> Result<Vec<DomainFrame>> {
    let domain_map = domain_map_by_code(domains);
    let mut frames = Vec::new();
    let config = RelationshipConfig::default();
    if let Some(relrec_domain) = domain_map.get("RELREC")
        && let Some(frame) = build_relrec(domain_frames, domains, relrec_domain, study_id, &config)?
    {
        frames.push(frame);
    }
    if let Some(relspec_domain) = domain_map.get("RELSPEC")
        && let Some(frame) = build_relspec(domain_frames, domains, relspec_domain, study_id)?
    {
        frames.push(frame);
    }
    if let Some(relsub_domain) = domain_map.get("RELSUB")
        && let Some(frame) = build_relsub(domain_frames, relsub_domain, study_id)?
    {
        frames.push(frame);
    }
    Ok(frames)
}

pub fn build_relspec(
    domain_frames: &[DomainFrame],
    domains: &[Domain],
    relspec_domain: &Domain,
    study_id: &str,
) -> Result<Option<DomainFrame>> {
    let domain_map = domain_map_by_code(domains);
    let mut records: BTreeMap<(String, String), RelspecRecord> = BTreeMap::new();
    for frame in domain_frames {
        let domain_def = match domain_map.get(&frame.domain_code.to_uppercase()) {
            Some(domain) => domain,
            None => {
                return Err(anyhow::anyhow!(
                    "missing standards metadata for domain {}",
                    frame.domain_code
                ));
            }
        };
        let domain_columns = standard_columns(domain_def);
        let usubjid_col = match domain_columns.usubjid.as_ref() {
            Some(name) => name,
            None => continue,
        };
        if frame.data.column(usubjid_col).is_err() {
            continue;
        }
        let Some(usubjid_vals) = column_trimmed_values(&frame.data, usubjid_col) else {
            continue;
        };
        let refid_cols = find_refid_columns(domain_def, &frame.data);
        if refid_cols.is_empty() {
            continue;
        }
        let spec_vals = domain_columns
            .spec
            .as_ref()
            .and_then(|name| column_trimmed_values(&frame.data, name));
        let parent_vals = domain_columns
            .parent
            .as_ref()
            .and_then(|name| column_trimmed_values(&frame.data, name));
        for refid_col in refid_cols {
            let Some(refid_vals) = column_trimmed_values(&frame.data, &refid_col) else {
                continue;
            };
            for idx in 0..frame.data.height() {
                let usubjid = usubjid_vals[idx].clone();
                let refid = refid_vals[idx].clone();
                if usubjid.is_empty() || refid.is_empty() {
                    continue;
                }
                let entry = records
                    .entry((usubjid.clone(), refid.clone()))
                    .or_insert_with(|| RelspecRecord::new(study_id, &usubjid, &refid));
                if entry.spec.is_empty()
                    && let Some(spec_vals) = spec_vals.as_ref()
                {
                    let spec = spec_vals[idx].clone();
                    if !spec.is_empty() {
                        entry.spec = spec;
                    }
                }
                if entry.parent.is_empty()
                    && let Some(parent_vals) = parent_vals.as_ref()
                {
                    let parent = parent_vals[idx].clone();
                    if !parent.is_empty() {
                        entry.parent = parent;
                    }
                }
            }
        }
    }
    if records.is_empty() {
        return Ok(None);
    }
    let relspec_columns = standard_columns(relspec_domain);
    let records: Vec<BTreeMap<String, String>> = records
        .into_values()
        .map(|record| record.into_map(&relspec_columns))
        .collect();
    let data = build_domain_frame_from_records(relspec_domain, &records)?;
    Ok(Some(DomainFrame::new(relspec_domain.code.clone(), data)))
}

pub fn build_relsub(
    domain_frames: &[DomainFrame],
    relsub_domain: &Domain,
    study_id: &str,
) -> Result<Option<DomainFrame>> {
    let required = required_data_columns(relsub_domain);
    let mut records: Vec<BTreeMap<String, String>> = Vec::new();
    for frame in domain_frames {
        if frame.data.height() == 0 {
            continue;
        }
        let lookup = frame_column_lookup(&frame.data);
        let required_cols: Vec<String> = required
            .iter()
            .filter_map(|name| lookup.get(name).cloned())
            .collect();
        if required_cols.len() != required.len() {
            continue;
        }
        let required_values: Vec<Vec<String>> = required_cols
            .iter()
            .map(|name| column_trimmed_values(&frame.data, name).unwrap_or_default())
            .collect();
        let usubjid_values = resolve_column(&lookup, "USUBJID")
            .and_then(|name| column_trimmed_values(&frame.data, name))
            .unwrap_or_else(|| vec![String::new(); frame.data.height()]);
        let poolid_values = resolve_column(&lookup, "POOLID")
            .and_then(|name| column_trimmed_values(&frame.data, name))
            .unwrap_or_else(|| vec![String::new(); frame.data.height()]);
        for idx in 0..frame.data.height() {
            let missing_required = required_values.iter().any(|values| values[idx].is_empty());
            if missing_required {
                continue;
            }
            if usubjid_values[idx].is_empty() && poolid_values[idx].is_empty() {
                continue;
            }
            let mut record = BTreeMap::new();
            for variable in &relsub_domain.variables {
                let value = resolve_column(&lookup, &variable.name)
                    .map(|name| column_value_string(&frame.data, name, idx))
                    .unwrap_or_default();
                let final_value =
                    if variable.name.eq_ignore_ascii_case("STUDYID") && value.trim().is_empty() {
                        study_id.to_string()
                    } else {
                        value
                    };
                record.insert(variable.name.clone(), final_value);
            }
            records.push(record);
        }
    }
    if records.is_empty() {
        return Ok(None);
    }
    let data = build_domain_frame_from_records(relsub_domain, &records)?;
    Ok(Some(DomainFrame::new(relsub_domain.code.clone(), data)))
}

fn relrec_record(
    relrec_domain: &Domain,
    study_id: &str,
    input: RelrecRecordInput<'_>,
) -> BTreeMap<String, String> {
    let mut record = BTreeMap::new();
    let columns = standard_columns(relrec_domain);
    if let Some(name) = columns.study_id {
        record.insert(name, study_id.to_string());
    }
    if let Some(name) = columns.rdomain {
        record.insert(name, input.rdomain.to_string());
    }
    if let Some(name) = columns.usubjid {
        record.insert(name, input.usubjid.to_string());
    }
    if let Some(name) = columns.idvar {
        record.insert(name, input.idvar.to_string());
    }
    if let Some(name) = columns.idvarval {
        record.insert(name, input.idvarval.to_string());
    }
    if let Some(name) = columns.reltype {
        record.insert(name, input.reltype.unwrap_or("").to_string());
    }
    if let Some(name) = columns.relid {
        record.insert(name, input.relid.to_string());
    }
    record
}

fn required_data_columns(domain: &Domain) -> Vec<String> {
    domain
        .variables
        .iter()
        .filter(|var| {
            var.core
                .as_deref()
                .map(|core| core.eq_ignore_ascii_case("Req"))
                .unwrap_or(false)
        })
        .map(|var| var.name.to_uppercase())
        .filter(|name| name != "STUDYID")
        .collect()
}

fn frame_column_lookup(df: &DataFrame) -> BTreeMap<String, String> {
    df.get_column_names_owned()
        .into_iter()
        .map(|name| (name.to_uppercase(), name.to_string()))
        .collect()
}

fn resolve_column<'a>(lookup: &'a BTreeMap<String, String>, name: &str) -> Option<&'a String> {
    lookup.get(&name.to_uppercase())
}

#[derive(Debug, Clone)]
struct RelspecRecord {
    study_id: String,
    usubjid: String,
    refid: String,
    spec: String,
    parent: String,
    level: String,
}

impl RelspecRecord {
    fn new(study_id: &str, usubjid: &str, refid: &str) -> Self {
        Self {
            study_id: study_id.to_string(),
            usubjid: usubjid.to_string(),
            refid: refid.to_string(),
            spec: String::new(),
            parent: String::new(),
            level: "1".to_string(),
        }
    }

    fn into_map(self, columns: &StandardColumns) -> BTreeMap<String, String> {
        let mut record = BTreeMap::new();
        if let Some(name) = columns.study_id.as_ref() {
            record.insert(name.clone(), self.study_id);
        }
        if let Some(name) = columns.usubjid.as_ref() {
            record.insert(name.clone(), self.usubjid);
        }
        if let Some(name) = columns.refid.as_ref() {
            record.insert(name.clone(), self.refid);
        }
        if let Some(name) = columns.spec.as_ref() {
            record.insert(name.clone(), self.spec);
        }
        if let Some(name) = columns.parent.as_ref() {
            record.insert(name.clone(), self.parent);
        }
        if let Some(name) = columns.level.as_ref() {
            record.insert(name.clone(), self.level);
        }
        record
    }
}

fn find_refid_columns(domain: &Domain, df: &DataFrame) -> Vec<String> {
    refid_candidates(domain)
        .into_iter()
        .filter(|name| df.column(name.as_str()).is_ok())
        .collect()
}
