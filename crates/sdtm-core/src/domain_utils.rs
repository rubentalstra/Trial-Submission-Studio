use std::collections::BTreeMap;

use sdtm_model::Domain;

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

pub fn column_map(domain: &Domain) -> BTreeMap<String, String> {
    domain
        .variables
        .iter()
        .map(|variable| (variable.name.to_uppercase(), variable.name.clone()))
        .collect()
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
    candidates.sort_by(|a, b| a.to_uppercase().cmp(&b.to_uppercase()));
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
    grp_candidates.sort_by(|a, b| a.to_uppercase().cmp(&b.to_uppercase()));
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
