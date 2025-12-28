//! Tests for domain file discovery.

use sdtm_ingest::{discover_domain_files, list_csv_files};
use std::fs;
use std::path::{Path, PathBuf};

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

fn touch(dir: &Path, name: &str) -> PathBuf {
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
