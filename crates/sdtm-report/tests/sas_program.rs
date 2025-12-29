//! Integration tests for SAS program generation.

use polars::prelude::{Column, DataFrame, IntoColumn, NamedFrom, Series};

use sdtm_model::{Domain, MappingConfig, MappingSuggestion, Variable, VariableType};
use sdtm_report::{SasProgramOptions, generate_sas_program};
use sdtm_transform::frame::DomainFrame;

fn test_df(columns: Vec<(&str, Vec<&str>)>) -> DataFrame {
    let cols: Vec<Column> = columns
        .into_iter()
        .map(|(name, values)| {
            Series::new(
                name.into(),
                values.iter().copied().map(String::from).collect::<Vec<_>>(),
            )
            .into_column()
        })
        .collect();
    DataFrame::new(cols).unwrap()
}

fn test_variable(name: &str, data_type: VariableType) -> Variable {
    Variable {
        name: name.to_string(),
        label: Some(format!("{} Label", name)),
        data_type,
        length: None,
        role: None,
        core: None,
        codelist_code: None,
        order: None,
    }
}

fn test_domain() -> Domain {
    Domain {
        code: "AE".to_string(),
        description: Some("Adverse Events".to_string()),
        class_name: Some("Events".to_string()),
        dataset_class: None,
        label: Some("Adverse Events".to_string()),
        structure: Some("One record per event".to_string()),
        dataset_name: None,
        variables: vec![
            {
                let mut v = test_variable("STUDYID", VariableType::Char);
                v.role = Some("Identifier".to_string());
                v.core = Some("Req".to_string());
                v
            },
            {
                let mut v = test_variable("DOMAIN", VariableType::Char);
                v.role = Some("Identifier".to_string());
                v.core = Some("Req".to_string());
                v
            },
            {
                let mut v = test_variable("USUBJID", VariableType::Char);
                v.role = Some("Identifier".to_string());
                v.core = Some("Req".to_string());
                v
            },
            {
                let mut v = test_variable("AESEQ", VariableType::Num);
                v.role = Some("Identifier".to_string());
                v.core = Some("Req".to_string());
                v
            },
            {
                let mut v = test_variable("AETERM", VariableType::Char);
                v.role = Some("Topic".to_string());
                v.core = Some("Req".to_string());
                v
            },
            {
                let mut v = test_variable("AEDECOD", VariableType::Char);
                v.role = Some("Synonym Qualifier".to_string());
                v.codelist_code = Some("C66830".to_string());
                v
            },
            {
                let mut v = test_variable("AESTDTC", VariableType::Char);
                v.role = Some("Timing".to_string());
                v
            },
        ],
    }
}

fn test_frame(domain: &Domain) -> DomainFrame {
    let df = test_df(vec![
        ("STUDYID", vec!["STUDY01", "STUDY01"]),
        ("DOMAIN", vec!["AE", "AE"]),
        ("USUBJID", vec!["STUDY01-001", "STUDY01-002"]),
        ("AESEQ", vec!["1", "1"]),
        ("AETERM", vec!["Headache", "Nausea"]),
        ("AEDECOD", vec!["HEADACHE", "NAUSEA"]),
        ("AESTDTC", vec!["2024-01-15", "2024-01-16"]),
    ]);
    DomainFrame::new(domain.code.clone(), df)
}

fn test_mapping() -> MappingConfig {
    MappingConfig {
        domain_code: "AE".to_string(),
        study_id: "STUDY01".to_string(),
        mappings: vec![
            MappingSuggestion {
                source_column: "subject_id".to_string(),
                target_variable: "USUBJID".to_string(),
                confidence: 0.95,
                transformation: None,
            },
            MappingSuggestion {
                source_column: "adverse_event".to_string(),
                target_variable: "AETERM".to_string(),
                confidence: 0.90,
                transformation: None,
            },
            MappingSuggestion {
                source_column: "event_code".to_string(),
                target_variable: "AEDECOD".to_string(),
                confidence: 0.85,
                transformation: Some("upcase(event_code)".to_string()),
            },
            MappingSuggestion {
                source_column: "start_date".to_string(),
                target_variable: "AESTDTC".to_string(),
                confidence: 0.88,
                transformation: None,
            },
        ],
        unmapped_columns: vec!["extra_col".to_string()],
    }
}

/// Generate SAS program with timestamp stripped for snapshot comparison.
fn generate_sas_program_no_timestamp(
    domain: &Domain,
    frame: &DomainFrame,
    mapping: &MappingConfig,
    options: &SasProgramOptions,
) -> anyhow::Result<String> {
    let program = generate_sas_program(domain, frame, mapping, options)?;
    // Strip the generated timestamp line for deterministic snapshots
    let lines: Vec<&str> = program
        .lines()
        .filter(|line| !line.starts_with("/* Generated:"))
        .collect();
    Ok(lines.join("\n"))
}

#[test]
fn test_generate_sas_program_snapshot() {
    let domain = test_domain();
    let frame = test_frame(&domain);
    let mapping = test_mapping();
    let options = SasProgramOptions::default();

    let program = generate_sas_program_no_timestamp(&domain, &frame, &mapping, &options)
        .expect("SAS program generation failed");

    insta::assert_snapshot!(program);
}

#[test]
fn test_generate_sas_program_with_custom_datasets() {
    let domain = test_domain();
    let frame = test_frame(&domain);
    let mapping = test_mapping();
    let options = SasProgramOptions {
        input_dataset: Some("source.ae_raw".to_string()),
        output_dataset: Some("sdtm.ae".to_string()),
    };

    let program = generate_sas_program_no_timestamp(&domain, &frame, &mapping, &options)
        .expect("SAS program generation failed");

    insta::assert_snapshot!(program);
}

#[test]
fn test_sas_program_contains_required_sections() {
    let domain = test_domain();
    let frame = test_frame(&domain);
    let mapping = test_mapping();
    let options = SasProgramOptions::default();

    let program = generate_sas_program(&domain, &frame, &mapping, &options)
        .expect("SAS program generation failed");

    // Verify essential sections are present
    assert!(program.contains("/* Generated by CDISC Transpiler */"));
    assert!(program.contains("/* Domain: AE */"));
    assert!(program.contains("DATA sdtm.ae;"));
    assert!(program.contains("SET work.ae;"));
    assert!(program.contains("length"));
    assert!(program.contains("/* Column mappings */"));
    assert!(program.contains("/* Defaulted required fields */"));
    assert!(program.contains("KEEP"));
    assert!(program.contains("RUN;"));
}

#[test]
fn test_sas_program_variable_lengths() {
    let domain = test_domain();
    let frame = test_frame(&domain);
    let mapping = test_mapping();
    let options = SasProgramOptions::default();

    let program = generate_sas_program(&domain, &frame, &mapping, &options)
        .expect("SAS program generation failed");

    // Verify variable length declarations
    assert!(program.contains("STUDYID $7")); // "STUDY01" = 7 chars
    assert!(program.contains("DOMAIN $2")); // "AE" = 2 chars
    assert!(program.contains("USUBJID $11")); // "STUDY01-001" = 11 chars
    assert!(program.contains("AESEQ 8")); // Numeric = 8
    assert!(program.contains("AETERM $8")); // "Headache" = 8 chars
}
