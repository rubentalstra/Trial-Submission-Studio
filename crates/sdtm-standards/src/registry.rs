#![deny(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use crate::csv::ct::{CtIndex, parse_ct_csv};
use crate::csv::datasets::{DatasetMeta, parse_datasets_csv};
use crate::csv::pinnacle21_rules::{P21RulesIndex, parse_pinnacle21_rules_csv};
use crate::csv::variables::{VariableKey, VariableMeta, parse_variables_csv};
use crate::error::StandardsError;
use crate::hash::sha256_hex;
use crate::manifest::{Manifest, ManifestFile};

const REQUIRED_ROLES: &[&str] = &[
    "sdtm_datasets",
    "sdtm_variables",
    "sdtmig_datasets",
    "sdtmig_variables",
    "pinnacle21_rules",
    "define_xsl_2_1",
    "define_xsl_2_0",
];

const CT_ROLES: &[&str] = &["ct_sdtm", "ct_define_xml"];

const ALLOWED_KINDS: &[&str] = &["csv", "json", "toml", "xsd", "xsl", "pdf", "other"];

#[derive(Debug, Clone, serde::Serialize)]
pub struct Conflict {
    pub kind: String,
    pub domain: String,
    pub var: Option<String>,
    pub field: String,
    pub sdtm: Option<String>,
    pub sdtmig: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VerifySummary {
    pub standards_dir: PathBuf,
    pub manifest_pins: crate::manifest::Pins,
    pub file_count: usize,
    pub domain_count_sdtm: usize,
    pub domain_count_sdtmig: usize,
    pub variable_count_sdtm: usize,
    pub variable_count_sdtmig: usize,
    pub codelist_count: usize,
    pub conflict_count: usize,
}

#[derive(Debug, Clone)]
pub struct StandardsRegistry {
    pub manifest: Manifest,
    pub files: Vec<ManifestFile>,
    pub sdtm_datasets: Vec<DatasetMeta>,
    pub sdtmig_datasets: Vec<DatasetMeta>,
    pub sdtm_variables: Vec<VariableMeta>,
    pub sdtmig_variables: Vec<VariableMeta>,
    pub ct_sdtm: CtIndex,
    pub p21_rules: P21RulesIndex,
    pub conflicts: Vec<Conflict>,
    pub datasets_by_domain: BTreeMap<String, DatasetMeta>,
    pub variables_by_domain: BTreeMap<String, Vec<VariableMeta>>,
}

impl StandardsRegistry {
    pub fn verify_and_load(standards_dir: &Path) -> Result<(Self, VerifySummary), StandardsError> {
        let manifest = load_manifest(&standards_dir.join("manifest.toml"))?;

        validate_manifest(&manifest, standards_dir)?;

        let mut files = manifest.files.clone();
        files.sort_by(|a, b| a.path.cmp(&b.path));

        for file in &files {
            verify_file(standards_dir, file)?;
        }

        let sdtm_datasets = parse_datasets_csv(
            &resolve_role_path(standards_dir, &files, "sdtm_datasets")?,
            "sdtm",
        )?;
        let sdtmig_datasets = parse_datasets_csv(
            &resolve_role_path(standards_dir, &files, "sdtmig_datasets")?,
            "sdtmig",
        )?;

        let sdtm_variables = parse_variables_csv(
            &resolve_role_path(standards_dir, &files, "sdtm_variables")?,
            "sdtm",
        )?;
        let sdtmig_variables = parse_variables_csv(
            &resolve_role_path(standards_dir, &files, "sdtmig_variables")?,
            "sdtmig",
        )?;

        let ct_sdtm = parse_ct_csv(&resolve_role_path_any(standards_dir, &files, CT_ROLES)?)?;

        let p21_rules = parse_pinnacle21_rules_csv(&resolve_role_path(
            standards_dir,
            &files,
            "pinnacle21_rules",
        )?)?;

        let conflicts = detect_conflicts(
            &sdtm_datasets,
            &sdtmig_datasets,
            &sdtm_variables,
            &sdtmig_variables,
        );

        let datasets_by_domain = build_datasets_by_domain(&sdtm_datasets, &sdtmig_datasets);
        let variables_by_domain = build_variables_by_domain(&sdtm_variables, &sdtmig_variables);

        let summary = VerifySummary {
            standards_dir: standards_dir.to_path_buf(),
            manifest_pins: manifest.pins.clone(),
            file_count: files.len(),
            domain_count_sdtm: sdtm_datasets.len(),
            domain_count_sdtmig: sdtmig_datasets.len(),
            variable_count_sdtm: sdtm_variables.len(),
            variable_count_sdtmig: sdtmig_variables.len(),
            codelist_count: ct_sdtm.codelists.len(),
            conflict_count: conflicts.len(),
        };

        Ok((
            Self {
                manifest,
                files,
                sdtm_datasets,
                sdtmig_datasets,
                sdtm_variables,
                sdtmig_variables,
                ct_sdtm,
                p21_rules,
                conflicts,
                datasets_by_domain,
                variables_by_domain,
            },
            summary,
        ))
    }
}

fn load_manifest(path: &Path) -> Result<Manifest, StandardsError> {
    let contents = std::fs::read_to_string(path).map_err(|e| StandardsError::io(path, e))?;
    toml::from_str(&contents).map_err(|e| StandardsError::Toml {
        path: path.to_path_buf(),
        source: e,
    })
}

fn validate_manifest(manifest: &Manifest, standards_dir: &Path) -> Result<(), StandardsError> {
    if manifest.manifest.schema != "cdisc-transpiler.standards-manifest" {
        return Err(StandardsError::InvalidManifest {
            message: format!("unsupported schema: {}", manifest.manifest.schema),
        });
    }
    if manifest.manifest.schema_version != 1 {
        return Err(StandardsError::InvalidManifest {
            message: format!(
                "unsupported schema_version: {}",
                manifest.manifest.schema_version
            ),
        });
    }

    let mut roles: BTreeSet<&str> = BTreeSet::new();
    let mut ct_present = false;
    let mut manifest_paths: BTreeSet<PathBuf> = BTreeSet::new();

    for file in &manifest.files {
        if roles.contains(file.role.as_str()) {
            return Err(StandardsError::DuplicateRole {
                role: file.role.clone(),
            });
        }
        roles.insert(file.role.as_str());

        if CT_ROLES.contains(&file.role.as_str()) {
            ct_present = true;
        }

        if !ALLOWED_KINDS.contains(&file.kind.as_str()) {
            return Err(StandardsError::InvalidManifest {
                message: format!("unsupported kind '{}' for {}", file.kind, file.path),
            });
        }

        validate_sha(&file.sha256, &file.path)?;

        let path = validate_path(&file.path)?;
        manifest_paths.insert(path);
    }

    for role in REQUIRED_ROLES {
        if !roles.contains(role) {
            return Err(StandardsError::MissingRole {
                role: role.to_string(),
            });
        }
    }

    if !ct_present {
        return Err(StandardsError::MissingCtRole {
            roles: CT_ROLES.join(", "),
        });
    }

    let actual_files = list_files_under(standards_dir)?;
    let manifest_paths: BTreeSet<PathBuf> = manifest_paths
        .into_iter()
        .map(|p| normalize_path(&p))
        .collect();

    for path in actual_files {
        if path == PathBuf::from("manifest.toml") {
            continue;
        }
        let normalized = normalize_path(&path);
        if !manifest_paths.contains(&normalized) {
            return Err(StandardsError::UnexpectedFile {
                path: standards_dir.join(path),
            });
        }
    }

    Ok(())
}

fn verify_file(standards_dir: &Path, file: &ManifestFile) -> Result<(), StandardsError> {
    let full_path = standards_dir.join(&file.path);
    let bytes = std::fs::read(&full_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            StandardsError::MissingFile {
                path: full_path.clone(),
            }
        } else {
            StandardsError::io(full_path.clone(), e)
        }
    })?;

    let actual = sha256_hex(&bytes);
    let expected = file.sha256.to_ascii_lowercase();
    if actual != expected {
        return Err(StandardsError::Sha256Mismatch {
            path: full_path,
            expected,
            actual,
        });
    }
    Ok(())
}

fn resolve_role_path(
    standards_dir: &Path,
    files: &[ManifestFile],
    role: &str,
) -> Result<PathBuf, StandardsError> {
    let f = files
        .iter()
        .find(|f| f.role == role)
        .ok_or_else(|| StandardsError::MissingRole {
            role: role.to_string(),
        })?;
    Ok(standards_dir.join(&f.path))
}

fn resolve_role_path_any(
    standards_dir: &Path,
    files: &[ManifestFile],
    roles: &[&str],
) -> Result<PathBuf, StandardsError> {
    for role in roles {
        if let Some(path) = files.iter().find(|f| f.role == *role) {
            return Ok(standards_dir.join(&path.path));
        }
    }
    Err(StandardsError::MissingCtRole {
        roles: roles.join(", "),
    })
}

fn detect_conflicts(
    sdtm_datasets: &[DatasetMeta],
    sdtmig_datasets: &[DatasetMeta],
    sdtm_variables: &[VariableMeta],
    sdtmig_variables: &[VariableMeta],
) -> Vec<Conflict> {
    let mut conflicts: Vec<Conflict> = Vec::new();

    let sdtm_ds: BTreeMap<String, &DatasetMeta> = sdtm_datasets
        .iter()
        .map(|d| (d.domain.clone(), d))
        .collect();
    let sdtmig_ds: BTreeMap<String, &DatasetMeta> = sdtmig_datasets
        .iter()
        .map(|d| (d.domain.clone(), d))
        .collect();
    for (domain, a) in &sdtm_ds {
        if let Some(b) = sdtmig_ds.get(domain) {
            if a.class != b.class {
                conflicts.push(Conflict {
                    kind: "dataset".to_string(),
                    domain: domain.clone(),
                    var: None,
                    field: "class".to_string(),
                    sdtm: a.class.clone(),
                    sdtmig: b.class.clone(),
                });
            }
            if a.label != b.label {
                conflicts.push(Conflict {
                    kind: "dataset".to_string(),
                    domain: domain.clone(),
                    var: None,
                    field: "label".to_string(),
                    sdtm: a.label.clone(),
                    sdtmig: b.label.clone(),
                });
            }
            if a.structure != b.structure {
                conflicts.push(Conflict {
                    kind: "dataset".to_string(),
                    domain: domain.clone(),
                    var: None,
                    field: "structure".to_string(),
                    sdtm: a.structure.clone(),
                    sdtmig: b.structure.clone(),
                });
            }
        }
    }

    let sdtm_var: BTreeMap<VariableKey, &VariableMeta> = sdtm_variables
        .iter()
        .filter(|v| v.domain != "*")
        .map(|v| {
            (
                VariableKey {
                    domain: v.domain.clone(),
                    var: v.var.clone(),
                },
                v,
            )
        })
        .collect();
    let sdtmig_var: BTreeMap<VariableKey, &VariableMeta> = sdtmig_variables
        .iter()
        .map(|v| {
            (
                VariableKey {
                    domain: v.domain.clone(),
                    var: v.var.clone(),
                },
                v,
            )
        })
        .collect();

    for (key, a) in &sdtm_var {
        if let Some(b) = sdtmig_var.get(key) {
            if a.label != b.label {
                conflicts.push(Conflict {
                    kind: "variable".to_string(),
                    domain: key.domain.clone(),
                    var: Some(key.var.clone()),
                    field: "label".to_string(),
                    sdtm: a.label.clone(),
                    sdtmig: b.label.clone(),
                });
            }
            if a.data_type != b.data_type {
                conflicts.push(Conflict {
                    kind: "variable".to_string(),
                    domain: key.domain.clone(),
                    var: Some(key.var.clone()),
                    field: "type".to_string(),
                    sdtm: a.data_type.clone(),
                    sdtmig: b.data_type.clone(),
                });
            }
            if a.role != b.role {
                conflicts.push(Conflict {
                    kind: "variable".to_string(),
                    domain: key.domain.clone(),
                    var: Some(key.var.clone()),
                    field: "role".to_string(),
                    sdtm: a.role.clone(),
                    sdtmig: b.role.clone(),
                });
            }
            if a.required != b.required {
                conflicts.push(Conflict {
                    kind: "variable".to_string(),
                    domain: key.domain.clone(),
                    var: Some(key.var.clone()),
                    field: "required".to_string(),
                    sdtm: a.required.map(|v| v.to_string()),
                    sdtmig: b.required.map(|v| v.to_string()),
                });
            }
        }
    }

    conflicts.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.domain.cmp(&b.domain))
            .then_with(|| a.var.cmp(&b.var))
            .then_with(|| a.field.cmp(&b.field))
    });
    conflicts
}

fn validate_sha(sha: &str, path: &str) -> Result<(), StandardsError> {
    if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(StandardsError::InvalidSha256 {
            path: PathBuf::from(path),
            message: "sha256 must be 64 hex characters".to_string(),
        });
    }
    Ok(())
}

fn validate_path(path: &str) -> Result<PathBuf, StandardsError> {
    if path.contains('\\') {
        return Err(StandardsError::InvalidPath {
            path: PathBuf::from(path),
            message: "manifest path must use '/' separators".to_string(),
        });
    }

    let p = PathBuf::from(path);
    if p.is_absolute() {
        return Err(StandardsError::InvalidPath {
            path: p,
            message: "manifest path must be relative".to_string(),
        });
    }

    for c in p.components() {
        if matches!(c, Component::ParentDir) {
            return Err(StandardsError::InvalidPath {
                path: PathBuf::from(path),
                message: "manifest path must not traverse out of standards/".to_string(),
            });
        }
    }

    Ok(p)
}

fn list_files_under(root: &Path) -> Result<BTreeSet<PathBuf>, StandardsError> {
    let mut stack = vec![root.to_path_buf()];
    let mut files = BTreeSet::new();

    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).map_err(|e| StandardsError::io(&dir, e))? {
            let entry = entry.map_err(|e| StandardsError::io(&dir, e))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let rel = path
                    .strip_prefix(root)
                    .map_err(|e| StandardsError::InvalidPath {
                        path: path.clone(),
                        message: format!("failed to relativize path: {e}"),
                    })?
                    .to_path_buf();
                files.insert(rel);
            }
        }
    }

    Ok(files)
}

fn normalize_path(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            _ => out.push(c.as_os_str()),
        }
    }
    out
}

fn build_datasets_by_domain(
    sdtm_datasets: &[DatasetMeta],
    sdtmig_datasets: &[DatasetMeta],
) -> BTreeMap<String, DatasetMeta> {
    // Baseline: SDTM. Refine: SDTMIG (overwrite fields if SDTMIG has values).
    let mut merged: BTreeMap<String, DatasetMeta> = BTreeMap::new();
    for d in sdtm_datasets {
        merged.insert(d.domain.clone(), d.clone());
    }
    for d in sdtmig_datasets {
        merged
            .entry(d.domain.clone())
            .and_modify(|m| {
                if d.class.is_some() {
                    m.class = d.class.clone();
                }
                if d.label.is_some() {
                    m.label = d.label.clone();
                }
                if d.structure.is_some() {
                    m.structure = d.structure.clone();
                }
                m.source = "merged".to_string();
            })
            .or_insert(d.clone());
    }
    merged
}

fn build_variables_by_domain(
    sdtm_variables: &[VariableMeta],
    sdtmig_variables: &[VariableMeta],
) -> BTreeMap<String, Vec<VariableMeta>> {
    let mut map: BTreeMap<String, BTreeMap<String, VariableMeta>> = BTreeMap::new();

    // SDTM global variables are in domain="*"; keep them separately but also
    // allow later logic to merge as needed.
    for v in sdtm_variables.iter().chain(sdtmig_variables.iter()) {
        map.entry(v.domain.clone())
            .or_default()
            .insert(v.var.clone(), v.clone());
    }

    let mut out: BTreeMap<String, Vec<VariableMeta>> = BTreeMap::new();
    for (domain, by_var) in map {
        let mut vars: Vec<VariableMeta> = by_var.into_values().collect();
        vars.sort_by(|a, b| a.var.cmp(&b.var));
        out.insert(domain, vars);
    }
    out
}
