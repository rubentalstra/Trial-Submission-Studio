# Suggested Code Changes (Open)

This list tracks SDTMIG-aligned improvements identified during review. Items are
worded as actionable tasks with file-level pointers.

## Open Items

- [x] Preserve ISO 8601 precision and partial dates.
  - Avoid coercing partials to full dates or dropping time components.
  - Files: `crates/sdtm-core/src/domain_processors/common.rs`,
    `crates/sdtm-core/src/domain_processors/dm.rs`

- [x] Make USUBJID prefixing and auto `--SEQ` assignment optional.
  - Gate behavior behind config flags and emit warnings when rewrites occur.
  - Files: `crates/sdtm-core/src/processor.rs`

- [x] Enforce SDTM variable ordering from `Variables.csv`.
  - Preserve `Variable Order` during standards load and use it for output order.
  - Files: `crates/sdtm-standards/src/loaders.rs`,
    `crates/sdtm-model/src/domain.rs`,
    `crates/sdtm-report/src/lib.rs`

- [x] Tighten `--TESTCD` and `QNAM` compliance.
  - Disallow leading digits and invalid characters; ensure 8-char max.
  - Files: `crates/sdtm-core/src/data_utils.rs`,
    `crates/sdtm-validate/src/lib.rs`

- [x] Improve SUPP-- generation metadata.
  - Use source column labels for `QLABEL` when available.
  - Set `QORIG` based on mapped vs derived data (not always `CRF`).
  - Use `SQ` prefix for long `SUPP` dataset names (SDTMIG 8.4.2).
  - Files: `crates/sdtm-core/src/suppqual.rs`,
    `crates/sdtm-core/src/domain_sets.rs`

- [x] Refine RELREC generation rules.
  - Avoid cross-domain `--SEQ` as merge keys; prefer `--LNKID/--GRPID`.
  - Populate `RELTYPE` only when relationship type is determinable.
  - Files: `crates/sdtm-core/src/relationships.rs`

## Notes

- SDTMIG references: Chapter 2.7 (disallowed variables), 4.2.1 (TESTCD/QNAM
  rules), 4.4 (date/time precision), 8.3–8.4 (RELREC/SUPP-- rules).
