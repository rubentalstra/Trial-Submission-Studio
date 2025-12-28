use std::collections::BTreeMap;

use anyhow::Result;
use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};

use sdtm_model::{Domain, VariableType};

use crate::data_utils::column_value_string;
use crate::domain_utils::{StandardColumns, refid_candidates, standard_columns};
use crate::frame::DomainFrame;
use sdtm_ingest::{any_to_string, parse_f64};

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

struct RelspecSource<'a> {
    df: &'a DataFrame,
    study_id: &'a str,
    relspec_domain: &'a Domain,
    usubjid_col: &'a str,
    refid_cols: &'a [String],
    spec_col: Option<&'a str>,
    parent_col: Option<&'a str>,
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

    let domain_map = build_domain_map(domains);
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
        for idx in 0..frame.data.height() {
            let usubjid = column_value_string(&frame.data, &usubjid_col, idx)
                .trim()
                .to_string();
            let idvarval = column_value_string(&frame.data, &link.name, idx)
                .trim()
                .to_string();
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

    let data = build_domain_frame(relrec_domain, &records)?;
    Ok(Some(DomainFrame {
        domain_code: relrec_domain.code.clone(),
        data,
        meta: None,
    }))
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
    let domain_map = build_domain_map(domains);
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
    let domain_map = build_domain_map(domains);
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
            Some(name) => name.clone(),
            None => continue,
        };
        let refid_cols = find_refid_columns(domain_def, &frame.data);
        if refid_cols.is_empty() {
            continue;
        }
        let spec_col = domain_columns.spec.clone();
        let parent_col = domain_columns.parent.clone();
        collect_relspec_records(
            &mut records,
            &RelspecSource {
                df: &frame.data,
                study_id,
                relspec_domain,
                usubjid_col: &usubjid_col,
                refid_cols: &refid_cols,
                spec_col: spec_col.as_deref(),
                parent_col: parent_col.as_deref(),
            },
        );
    }
    if records.is_empty() {
        return Ok(None);
    }
    let relspec_columns = standard_columns(relspec_domain);
    let records: Vec<BTreeMap<String, String>> = records
        .into_values()
        .map(|record| record.into_map(&relspec_columns))
        .collect();
    let data = build_domain_frame(relspec_domain, &records)?;
    Ok(Some(DomainFrame {
        domain_code: relspec_domain.code.clone(),
        data,
        meta: None,
    }))
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
        if !required.iter().all(|name| lookup.contains_key(name)) {
            continue;
        }
        for idx in 0..frame.data.height() {
            if !row_has_required(&frame.data, &lookup, &required, idx) {
                continue;
            }
            if !row_has_subject_reference(&frame.data, &lookup, idx) {
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
    let data = build_domain_frame(relsub_domain, &records)?;
    Ok(Some(DomainFrame {
        domain_code: relsub_domain.code.clone(),
        data,
        meta: None,
    }))
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

fn build_domain_frame(domain: &Domain, records: &[BTreeMap<String, String>]) -> Result<DataFrame> {
    let mut columns: Vec<Column> = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        match variable.data_type {
            VariableType::Num => {
                let mut values: Vec<Option<f64>> = Vec::with_capacity(records.len());
                for record in records {
                    let raw = record.get(&variable.name).map(|v| v.trim()).unwrap_or("");
                    values.push(parse_f64(raw));
                }
                columns.push(Series::new(variable.name.as_str().into(), values).into());
            }
            VariableType::Char => {
                let mut values: Vec<String> = Vec::with_capacity(records.len());
                for record in records {
                    values.push(record.get(&variable.name).cloned().unwrap_or_default());
                }
                columns.push(Series::new(variable.name.as_str().into(), values).into());
            }
        }
    }
    let data = DataFrame::new(columns)?;
    Ok(data)
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

fn row_has_required(
    df: &DataFrame,
    lookup: &BTreeMap<String, String>,
    required: &[String],
    idx: usize,
) -> bool {
    required.iter().all(|name| {
        resolve_column(lookup, name)
            .map(|col| !column_value_string(df, col, idx).trim().is_empty())
            .unwrap_or(false)
    })
}

fn row_has_subject_reference(
    df: &DataFrame,
    lookup: &BTreeMap<String, String>,
    idx: usize,
) -> bool {
    let usubjid = resolve_column(lookup, "USUBJID")
        .map(|col| column_value_string(df, col, idx))
        .unwrap_or_default();
    let poolid = resolve_column(lookup, "POOLID")
        .map(|col| column_value_string(df, col, idx))
        .unwrap_or_default();
    !(usubjid.trim().is_empty() && poolid.trim().is_empty())
}

#[derive(Debug, Clone)]
struct RelspecRecord {
    study_id: String,
    usubjid: String,
    refid: String,
    spec: String,
    parent: String,
    level: String,
    columns: BTreeMap<String, String>,
}

impl RelspecRecord {
    fn new(study_id: &str, usubjid: &str, refid: &str, relspec_domain: &Domain) -> Self {
        let mut columns = BTreeMap::new();
        for variable in &relspec_domain.variables {
            columns.insert(variable.name.clone(), String::new());
        }
        Self {
            study_id: study_id.to_string(),
            usubjid: usubjid.to_string(),
            refid: refid.to_string(),
            spec: String::new(),
            parent: String::new(),
            level: "1".to_string(),
            columns,
        }
    }

    fn into_map(mut self, columns: &StandardColumns) -> BTreeMap<String, String> {
        if let Some(name) = columns.study_id.as_ref() {
            self.columns.insert(name.clone(), self.study_id.clone());
        }
        if let Some(name) = columns.usubjid.as_ref() {
            self.columns.insert(name.clone(), self.usubjid.clone());
        }
        if let Some(name) = columns.refid.as_ref() {
            self.columns.insert(name.clone(), self.refid.clone());
        }
        if let Some(name) = columns.spec.as_ref() {
            self.columns.insert(name.clone(), self.spec.clone());
        }
        if let Some(name) = columns.parent.as_ref() {
            self.columns.insert(name.clone(), self.parent.clone());
        }
        if let Some(name) = columns.level.as_ref() {
            self.columns.insert(name.clone(), self.level.clone());
        }
        self.columns
    }
}

fn collect_relspec_records(
    records: &mut BTreeMap<(String, String), RelspecRecord>,
    source: &RelspecSource<'_>,
) {
    if source.df.height() == 0 {
        return;
    }
    if source.df.column(source.usubjid_col).is_err() {
        return;
    }
    for refid_col in source.refid_cols {
        for idx in 0..source.df.height() {
            let usubjid = column_value_string(source.df, source.usubjid_col, idx)
                .trim()
                .to_string();
            let refid = column_value_string(source.df, refid_col, idx)
                .trim()
                .to_string();
            if usubjid.is_empty() || refid.is_empty() {
                continue;
            }
            let key = (usubjid.clone(), refid.clone());
            let entry = records.entry(key).or_insert_with(|| {
                RelspecRecord::new(source.study_id, &usubjid, &refid, source.relspec_domain)
            });
            if entry.spec.is_empty()
                && let Some(spec_col) = source.spec_col
            {
                let spec = column_value_string(source.df, spec_col, idx)
                    .trim()
                    .to_string();
                if !spec.is_empty() {
                    entry.spec = spec;
                }
            }
            if entry.parent.is_empty()
                && let Some(parent_col) = source.parent_col
            {
                let parent = column_value_string(source.df, parent_col, idx)
                    .trim()
                    .to_string();
                if !parent.is_empty() {
                    entry.parent = parent;
                }
            }
        }
    }
}

fn find_refid_columns(domain: &Domain, df: &DataFrame) -> Vec<String> {
    refid_candidates(domain)
        .into_iter()
        .filter(|name| df.column(name.as_str()).is_ok())
        .collect()
}

fn build_domain_map(domains: &[Domain]) -> BTreeMap<String, &Domain> {
    let mut map = BTreeMap::new();
    for domain in domains {
        map.insert(domain.code.to_uppercase(), domain);
    }
    map
}
