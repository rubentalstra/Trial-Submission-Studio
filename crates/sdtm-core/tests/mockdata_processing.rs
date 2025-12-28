use std::collections::BTreeSet;
use std::path::Path;

use sdtm_core::{
    DomainFrame, ProcessingContext, RelationshipConfig, SuppqualInput, build_domain_frame,
    build_domain_frame_with_mapping, build_relrec, build_relspec, build_relsub, build_suppqual,
    process_domain_with_context,
};
use sdtm_ingest::{build_column_hints, discover_domain_files, list_csv_files, read_csv_table};
use sdtm_map::MappingEngine;
use sdtm_standards::{load_default_ct_registry, load_default_sdtm_ig_domains};

#[test]
fn processes_mockdata_end_to_end() {
    let standards = load_default_sdtm_ig_domains().expect("standards");
    let ct_registry = load_default_ct_registry().expect("ct registry");
    let domain_codes: Vec<String> = standards.iter().map(|domain| domain.code.clone()).collect();
    let mut domain_map = std::collections::BTreeMap::new();
    for domain in &standards {
        domain_map.insert(domain.code.to_uppercase(), domain);
    }
    let suppqual = standards
        .iter()
        .find(|domain| domain.code == "SUPPQUAL")
        .expect("SUPPQUAL domain");
    let relrec = standards
        .iter()
        .find(|domain| domain.code == "RELREC")
        .expect("RELREC domain");
    let relspec = standards
        .iter()
        .find(|domain| domain.code == "RELSPEC")
        .expect("RELSPEC domain");
    let relsub = standards
        .iter()
        .find(|domain| domain.code == "RELSUB")
        .expect("RELSUB domain");

    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mockdata/DEMO_GDISC_20240903_072908");
    let csv_files = list_csv_files(&root).expect("list csv files");
    let discovered = discover_domain_files(&csv_files, &domain_codes);

    let study_id = "DEMO_GDISC_20240903_072908";
    let ctx = ProcessingContext::new(study_id).with_ct_registry(&ct_registry);
    let mut processed_frames: Vec<DomainFrame> = Vec::new();
    let mut suppqual_frames: Vec<DomainFrame> = Vec::new();

    for (domain_code, files) in discovered {
        let domain = domain_map
            .get(&domain_code.to_uppercase())
            .expect("domain metadata");
        let mut combined: Option<polars::prelude::DataFrame> = None;
        for (path, _variant) in files {
            let table = read_csv_table(&path).expect("read csv");
            let hints = build_column_hints(&table);
            let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
            let result = engine.suggest(&table.headers);
            let mapping_config = engine.to_config(study_id, result);
            let mut mapped = build_domain_frame_with_mapping(&table, domain, Some(&mapping_config))
                .expect("build mapped frame");
            process_domain_with_context(domain, &mut mapped.data, &ctx).expect("process");

            if let Some(existing) = &mut combined {
                existing.vstack_mut(&mapped.data).expect("vstack");
            } else {
                combined = Some(mapped.data.clone());
            }

            let used: BTreeSet<String> = mapping_config
                .mappings
                .iter()
                .map(|mapping| mapping.source_column.clone())
                .collect();
            let source = build_domain_frame(&table, &domain_code).expect("source frame");
            if let Some(result) = build_suppqual(SuppqualInput {
                parent_domain: domain,
                suppqual_domain: suppqual,
                source_df: &source.data,
                mapped_df: Some(&mapped.data),
                used_source_columns: &used,
                study_id,
                exclusion_columns: None,
                source_labels: None,
                derived_columns: None,
            })
            .expect("suppqual")
            {
                suppqual_frames.push(DomainFrame {
                    domain_code: result.domain_code,
                    data: result.data,
                    meta: None,
                });
            }
        }
        if let Some(data) = combined {
            processed_frames.push(DomainFrame {
                domain_code: domain.code.clone(),
                data,
                meta: None,
            });
        }
    }

    assert!(!processed_frames.is_empty());
    assert!(
        processed_frames
            .iter()
            .any(|frame| frame.domain_code.eq_ignore_ascii_case("DM"))
    );

    let config = RelationshipConfig::default();
    let _ = build_relrec(&processed_frames, &standards, relrec, study_id, &config).expect("relrec");
    let _ = build_relspec(&processed_frames, &standards, relspec, study_id).expect("relspec");
    let _ = build_relsub(&processed_frames, relsub, study_id).expect("relsub");
    let _ = suppqual_frames;
}
