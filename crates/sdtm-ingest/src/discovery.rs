use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

const METADATA_SKIP_PATTERNS: [&str; 5] = ["CODELISTS", "CODELIST", "ITEMS", "README", "METADATA"];

pub fn list_csv_files(study_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(study_dir).with_context(|| {
        format!("read study directory: {}", study_dir.display())
    })? {
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

fn match_domain(
    filename: &str,
    supported_domains: &[String],
) -> Option<(String, String)> {
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

#[cfg(test)]
mod tests {
    use super::{discover_domain_files, list_csv_files};
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        let mut dir = std::env::temp_dir();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        dir.push(format!("sdtm_ingest_{stamp}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn touch(dir: &PathBuf, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, "A,B\n1,2\n").expect("write file");
        path
    }

    #[test]
    fn discovers_domains_and_skips_metadata() {
        let dir = temp_dir();
        let _ = touch(&dir, "AE.csv");
        let _ = touch(&dir, "AEVAR.csv");
        let _ = touch(&dir, "foo_AE_bar.csv");
        let _ = touch(&dir, "DM_LC.csv");
        let _ = touch(&dir, "LB_PREG.csv");
        let _ = touch(&dir, "LBCC.csv");
        let _ = touch(&dir, "LBHM.csv");
        let _ = touch(&dir, "LBSA.csv");
        let _ = touch(&dir, "LBUR.csv");
        let _ = touch(&dir, "DS_EOT.csv");
        let _ = touch(&dir, "README.csv");
        let _ = touch(&dir, "CODELISTS.csv");

        let files = list_csv_files(&dir).expect("list csv");
        let supported = vec![
            "AE".to_string(),
            "DM".to_string(),
            "LB".to_string(),
            "DS".to_string(),
        ];
        let discovered = discover_domain_files(&files, &supported);

        let ae = discovered.get("AE").expect("AE matches");
        assert_eq!(ae.len(), 3);
        let dm = discovered.get("DM").expect("DM matches");
        assert_eq!(dm.len(), 1);
        let lb = discovered.get("LB").expect("LB matches");
        assert_eq!(lb.len(), 5);
        let ds = discovered.get("DS").expect("DS matches");
        assert_eq!(ds.len(), 1);

        fs::remove_dir_all(&dir).expect("cleanup");
    }
}
