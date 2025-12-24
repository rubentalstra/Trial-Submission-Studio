use std::fs;
use std::path::{Path, PathBuf};

use sdtm_standards::hash::sha256_hex;

fn unique_temp_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "cdisc-transpiler-{}-{}-{}",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    dir
}

fn write(path: &Path, contents: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn sha(path: &Path) -> String {
    let bytes = fs::read(path).unwrap();
    sha256_hex(&bytes)
}

#[test]
fn verify_and_doctor_report_snapshot_is_stable() {
    let standards_dir = unique_temp_dir("standards");
    fs::create_dir_all(&standards_dir).unwrap();

    // Minimal SDTM datasets/variables
    write(
        &standards_dir.join("sdtm/v2_0/Datasets.csv"),
        br#""Version","Class","Dataset Name","Dataset Label","Structure"
"SDTM v2.0","Special-Purpose","DM","Demographics",""
"#,
    );
    write(
        &standards_dir.join("sdtm/v2_0/Variables.csv"),
        br#""Version","Variable Order","Class","Dataset Name","Variable Name","Variable Label","Type","Described Value Domain","Role","Variables Qualified","Usage Restrictions","Variable C-Code","Definition","Notes","Examples"
"SDTM v2.0","1","General Observations","DM","STUDYID","Study Identifier","Char","","Identifier","","","","","",""
"#,
    );

    // Minimal SDTMIG datasets/variables
    write(
        &standards_dir.join("sdtmig/v3_4/Datasets.csv"),
        br#""Version","Class","Dataset Name","Dataset Label","Structure"
"SDTMIG v3.4","Special-Purpose","DM","Demographics","One record per subject"
"#,
    );
    write(
        &standards_dir.join("sdtmig/v3_4/Variables.csv"),
        br#""Version","Variable Order","Class","Dataset Name","Variable Name","Variable Label","Type","CDISC CT Codelist Code(s)","Codelist Submission Values","Described Value Domain(s)","Value List","Role","CDISC Notes","Core"
"SDTMIG v3.4","1","Special-Purpose","DM","STUDYID","Study Identifier","Char","","","","","Identifier","","Req"
"#,
    );

    // Minimal CT
    write(
        &standards_dir.join("ct/2024-03-29/SDTM_CT_2024-03-29.csv"),
        br#""Code","Codelist Code","Codelist Extensible (Yes/No)","Codelist Name","CDISC Submission Value","CDISC Synonym(s)","CDISC Definition","NCI Preferred Term","Standard and Date"
"C0001",,"No","Test Codelist","","","","","SDTM CT 2024-03-29"
"C0002","C0001","No","Test Codelist","VAL1","","","Value 1","SDTM CT 2024-03-29"
"#,
    );

    // Conformance rules + XSL
    write(
        &standards_dir.join("conformance_rules/v2_0/catalog.toml"),
        br#"[catalog]
schema = "cdisc-transpiler.conformance-rules"
schema_version = 1
ruleset = "sdtm-sdtmig-conformance"
ruleset_version = "v2_0"
"#,
    );
    write(
        &standards_dir.join("xsl/define2-1.xsl"),
        b"<xsl:stylesheet version=\"1.0\"></xsl:stylesheet>",
    );
    write(
        &standards_dir.join("xsl/define2-0-0.xsl"),
        b"<xsl:stylesheet version=\"1.0\"></xsl:stylesheet>",
    );

    let manifest = format!(
        r#"[manifest]
schema = "cdisc-transpiler.standards-manifest"
schema_version = 1

[pins]
sdtm = "v2_0"
sdtmig = "v3_4"
conformance_rules = "v2_0"
ct = "2024-03-29"

[policy]
precedence = "sdtm_then_sdtmig"

[[files]]
path = "sdtm/v2_0/Datasets.csv"
sha256 = "{}"
kind = "csv"
role = "sdtm_datasets"

[[files]]
path = "sdtm/v2_0/Variables.csv"
sha256 = "{}"
kind = "csv"
role = "sdtm_variables"

[[files]]
path = "sdtmig/v3_4/Datasets.csv"
sha256 = "{}"
kind = "csv"
role = "sdtmig_datasets"

[[files]]
path = "sdtmig/v3_4/Variables.csv"
sha256 = "{}"
kind = "csv"
role = "sdtmig_variables"

[[files]]
path = "ct/2024-03-29/SDTM_CT_2024-03-29.csv"
sha256 = "{}"
kind = "csv"
role = "ct_sdtm"

[[files]]
path = "conformance_rules/v2_0/catalog.toml"
sha256 = "{}"
kind = "toml"
role = "conformance_rules_catalog"

[[files]]
path = "xsl/define2-1.xsl"
sha256 = "{}"
kind = "xsl"
role = "define_xsl_2_1"

[[files]]
path = "xsl/define2-0-0.xsl"
sha256 = "{}"
kind = "xsl"
role = "define_xsl_2_0"
"#,
        sha(&standards_dir.join("sdtm/v2_0/Datasets.csv")),
        sha(&standards_dir.join("sdtm/v2_0/Variables.csv")),
        sha(&standards_dir.join("sdtmig/v3_4/Datasets.csv")),
        sha(&standards_dir.join("sdtmig/v3_4/Variables.csv")),
        sha(&standards_dir.join("ct/2024-03-29/SDTM_CT_2024-03-29.csv")),
        sha(&standards_dir.join("conformance_rules/v2_0/catalog.toml")),
        sha(&standards_dir.join("xsl/define2-1.xsl")),
        sha(&standards_dir.join("xsl/define2-0-0.xsl")),
    );

    write(&standards_dir.join("manifest.toml"), manifest.as_bytes());

    let (registry, summary) = sdtm_standards::StandardsRegistry::verify_and_load(&standards_dir)
        .expect("verify_and_load should succeed");

    assert_eq!(summary.domain_count_sdtm, 1);
    assert_eq!(summary.domain_count_sdtmig, 1);
    assert_eq!(summary.codelist_count, 1);

    let report = sdtm_standards::DoctorReport::from_verify_summary(
        &summary,
        registry.manifest.policy.clone(),
        registry.files.clone(),
        registry.conflicts.clone(),
    );

    insta::assert_json_snapshot!(serde_json::to_value(report).unwrap());
}
