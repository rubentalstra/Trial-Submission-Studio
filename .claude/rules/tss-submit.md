---
paths:
  - "crates/tss-submit/**/*.rs"
---

# Submission Pipeline Rules (tss-submit)

## Core Responsibilities

- Mapping source data to CDISC domains
- Normalization and validation
- Export to XPT/XML formats

## Before Modifying

ASK before:
- Changing validation rules or error types
- Modifying the mapping pipeline
- Changing export format behavior

## Dependencies

Uses:
- `tss-standards` for CDISC definitions
- `tss-ingest` for source data
- `xportrs` for XPT generation
- `quick-xml` for XML generation