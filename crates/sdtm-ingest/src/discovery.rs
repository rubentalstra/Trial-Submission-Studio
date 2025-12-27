use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

const METADATA_SKIP_PATTERNS: [&str; 5] = ["CODELISTS", "CODELIST", "ITEMS", "README", "METADATA"];

pub fn list_csv_files(study_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(study_dir)
        .with_context(|| format!("read study directory: {}", study_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path
            .extension()
            .and_then(|v| v.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("csv"))
            != Some(true)
        {
            continue;
        }
        files.push(path);
    }
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(files)
}

pub fn discover_domain_files(
    csv_files: &[PathBuf],
    supported_domains: &[String],
) -> BTreeMap<String, Vec<(PathBuf, String)>> {
    let supported: Vec<String> = supported_domains
        .iter()
        .map(|d| d.trim().to_uppercase())
        .collect();
    let mut domain_files: BTreeMap<String, Vec<(PathBuf, String)>> = BTreeMap::new();
    for path in csv_files {
        let stem = path
            .file_stem()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_string();
        let filename = stem.to_uppercase();
        if is_metadata_file(&filename) {
            continue;
        }
        if let Some((domain, variant)) = match_domain(&filename, &supported) {
            domain_files
                .entry(domain)
                .or_default()
                .push((path.clone(), variant));
        }
    }
    domain_files
}

fn is_metadata_file(filename: &str) -> bool {
    METADATA_SKIP_PATTERNS
        .iter()
        .any(|pattern| filename.contains(pattern))
}

fn match_domain(filename: &str, supported_domains: &[String]) -> Option<(String, String)> {
    let padded = format!("_{filename}_");
    if let Some(domain) = supported_domains
        .iter()
        .find(|domain| padded.contains(&format!("_{domain}_")))
    {
        return Some((domain.clone(), domain.clone()));
    }
    let mut domains_by_len: Vec<&String> = supported_domains.iter().collect();
    domains_by_len.sort_by_key(|domain| std::cmp::Reverse(domain.len()));
    let parts: Vec<&str> = filename.split('_').collect();
    for part in parts {
        for domain in &domains_by_len {
            if part.starts_with(domain.as_str()) {
                return Some(((*domain).clone(), part.to_string()));
            }
        }
    }
    None
}
