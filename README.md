# Trial Submission Studio (Rust GUI)

A Rust-first CLI tool for transpiling clinical trial source data into CDISC SDTM
outputs (XPT, Dataset-XML, Define-XML) with strict, offline validation.

```bash
cargo run --package cdisc-gui 
```

## Target Features

- Fully offline operation with committed standards and CT
- Deterministic, auditable output generation
- Validation-first pipeline with conformance gating
- Outputs: XPT (SAS V5), Dataset-XML 1.0, Define-XML 2.1





## References
[record-layout-of-a-sas-version-5-or-6-data-set-in-sas-transport-xport-format.pdf](crates/cdisc-xpt/record-layout-of-a-sas-version-5-or-6-data-set-in-sas-transport-xport-format.pdf)
[record-layout-of-a-sas-version-8-or-9-data-set-in-sas-transport-format.pdf](crates/cdisc-xpt/record-layout-of-a-sas-version-8-or-9-data-set-in-sas-transport-format.pdf)
