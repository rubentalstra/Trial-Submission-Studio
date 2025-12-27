use std::fs;
use std::path::PathBuf;

use sdtm_map::{MappingConfigLoader, MappingRepository, StoredMappingConfig};
use sdtm_model::{MappingConfig, MappingSuggestion};

fn temp_repo_dir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    dir.push(format!("sdtm_map_repo_{stamp}"));
    dir
}

fn cleanup_dir(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

fn sample_config(study_id: &str, domain: &str) -> MappingConfig {
    MappingConfig {
        domain_code: domain.to_string(),
        study_id: study_id.to_string(),
        mappings: vec![
            MappingSuggestion {
                source_column: "SUBJID".to_string(),
                target_variable: "USUBJID".to_string(),
                confidence: 0.95,
                transformation: None,
            },
            MappingSuggestion {
                source_column: "AGE_YEARS".to_string(),
                target_variable: "AGE".to_string(),
                confidence: 0.85,
                transformation: Some("numeric".to_string()),
            },
        ],
        unmapped_columns: vec!["EXTRA_COL".to_string()],
    }
}

#[test]
fn repository_save_and_load() {
    let dir = temp_repo_dir();
    let repo = MappingRepository::new(&dir).expect("create repo");

    let config = sample_config("STUDY001", "DM");
    let path = repo.save(&config).expect("save mapping");

    assert!(path.exists());
    assert!(path.to_string_lossy().contains("STUDY001_DM.json"));

    let loaded = repo
        .load("STUDY001", "DM")
        .expect("load mapping")
        .expect("mapping should exist");

    assert_eq!(loaded.study_id, "STUDY001");
    assert_eq!(loaded.domain_code, "DM");
    assert_eq!(loaded.mappings.len(), 2);
    assert_eq!(loaded.unmapped_columns, vec!["EXTRA_COL"]);

    cleanup_dir(&dir);
}

#[test]
fn repository_load_nonexistent() {
    let dir = temp_repo_dir();
    let repo = MappingRepository::new(&dir).expect("create repo");

    let loaded = repo.load("NOEXIST", "XX").expect("load attempt");
    assert!(loaded.is_none());

    cleanup_dir(&dir);
}

#[test]
fn repository_exists_check() {
    let dir = temp_repo_dir();
    let repo = MappingRepository::new(&dir).expect("create repo");

    assert!(!repo.exists("STUDY001", "DM"));

    let config = sample_config("STUDY001", "DM");
    repo.save(&config).expect("save mapping");

    assert!(repo.exists("STUDY001", "DM"));
    assert!(!repo.exists("STUDY001", "AE"));

    cleanup_dir(&dir);
}

#[test]
fn repository_delete() {
    let dir = temp_repo_dir();
    let repo = MappingRepository::new(&dir).expect("create repo");

    let config = sample_config("STUDY001", "DM");
    repo.save(&config).expect("save mapping");

    assert!(repo.exists("STUDY001", "DM"));

    let deleted = repo.delete("STUDY001", "DM").expect("delete");
    assert!(deleted);
    assert!(!repo.exists("STUDY001", "DM"));

    let deleted_again = repo.delete("STUDY001", "DM").expect("delete again");
    assert!(!deleted_again);

    cleanup_dir(&dir);
}

#[test]
fn repository_list() {
    let dir = temp_repo_dir();
    let repo = MappingRepository::new(&dir).expect("create repo");

    repo.save(&sample_config("STUDY001", "DM")).expect("save");
    repo.save(&sample_config("STUDY001", "AE")).expect("save");
    repo.save(&sample_config("STUDY002", "DM")).expect("save");

    let list = repo.list().expect("list mappings");
    assert_eq!(list.len(), 3);

    // Sorted by study_id, then domain_code
    assert_eq!(list[0].study_id, "STUDY001");
    assert_eq!(list[0].domain_code, "AE");
    assert_eq!(list[1].study_id, "STUDY001");
    assert_eq!(list[1].domain_code, "DM");
    assert_eq!(list[2].study_id, "STUDY002");
    assert_eq!(list[2].domain_code, "DM");

    cleanup_dir(&dir);
}

#[test]
fn repository_load_study_mappings() {
    let dir = temp_repo_dir();
    let repo = MappingRepository::new(&dir).expect("create repo");

    repo.save(&sample_config("STUDY001", "DM")).expect("save");
    repo.save(&sample_config("STUDY001", "AE")).expect("save");
    repo.save(&sample_config("STUDY002", "DM")).expect("save");

    let study1_mappings = repo.load_study_mappings("STUDY001").expect("load study");
    assert_eq!(study1_mappings.len(), 2);
    assert!(study1_mappings.contains_key("DM"));
    assert!(study1_mappings.contains_key("AE"));

    let study2_mappings = repo.load_study_mappings("STUDY002").expect("load study");
    assert_eq!(study2_mappings.len(), 1);
    assert!(study2_mappings.contains_key("DM"));

    cleanup_dir(&dir);
}

#[test]
fn stored_config_with_metadata() {
    let dir = temp_repo_dir();
    let repo = MappingRepository::new(&dir).expect("create repo");

    let config = sample_config("STUDY001", "DM");
    let stored = StoredMappingConfig::new(config).with_description("Test mapping for DM domain");

    repo.save_stored(&stored).expect("save stored");

    let loaded = repo
        .load_stored("STUDY001", "DM")
        .expect("load")
        .expect("exists");

    assert_eq!(
        loaded.description,
        Some("Test mapping for DM domain".to_string())
    );
    assert!(loaded.saved_at.is_some());
    assert_eq!(loaded.version, "1.0");

    cleanup_dir(&dir);
}

#[test]
fn mapping_config_loader_with_repository() {
    let dir = temp_repo_dir();
    let repo = MappingRepository::new(&dir).expect("create repo");

    // Save a mapping first
    repo.save(&sample_config("STUDY001", "DM")).expect("save");

    let loader = MappingConfigLoader::new("STUDY001").with_repository(repo);

    // Should load from repository
    let dm_config = loader
        .load_or_default("DM", || panic!("should not call default"))
        .expect("load");
    assert_eq!(dm_config.domain_code, "DM");

    // Should fall back to default when not found
    let ae_config = loader
        .load_or_default("AE", || sample_config("STUDY001", "AE"))
        .expect("load");
    assert_eq!(ae_config.domain_code, "AE");

    cleanup_dir(&dir);
}

#[test]
fn mapping_config_loader_without_repository() {
    let loader = MappingConfigLoader::new("STUDY001");

    // Should always use default when no repository configured
    let config = loader
        .load_or_default("DM", || sample_config("STUDY001", "DM"))
        .expect("load");
    assert_eq!(config.domain_code, "DM");
}

#[test]
fn normalize_special_characters_in_ids() {
    let dir = temp_repo_dir();
    let repo = MappingRepository::new(&dir).expect("create repo");

    // Study ID with special characters
    let config = MappingConfig {
        domain_code: "DM".to_string(),
        study_id: "STUDY-001/A".to_string(),
        mappings: vec![],
        unmapped_columns: vec![],
    };

    repo.save(&config).expect("save");

    // Should normalize to STUDY_001_A_DM.json
    let loaded = repo
        .load("STUDY-001/A", "DM")
        .expect("load")
        .expect("exists");
    assert_eq!(loaded.study_id, "STUDY-001/A");

    cleanup_dir(&dir);
}
