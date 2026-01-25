---
name: export-domain
description: Export SDTM domains to XPT or XML format
---
# Export CDISC Domain

Export domain data to FDA-compliant formats.

## Usage
`/export-domain <format>` where format is `xpt`, `dataset-xml`, or `define-xml`

## Workflow
1. Review export requirements in tss-submit/src/export/
2. Understand the format-specific constraints
3. Run export tests: `cargo test --package tss-submit export`
4. Verify output structure matches CDISC specs

## Key Files
- `crates/tss-submit/src/export/mod.rs` - Export entry points
- `crates/tss-submit/src/export/xpt.rs` - SAS Transport format
- `crates/tss-submit/src/export/dataset_xml.rs` - Dataset-XML
- `crates/tss-submit/src/export/define_xml.rs` - Define-XML metadata

## Export Formats
- **XPT v5/v8**: FDA standard submission format (via xportrs)
- **Dataset-XML**: CDISC XML interchange
- **Define-XML**: Metadata documentation with OIDs
