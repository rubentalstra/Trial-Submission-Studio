# Rust CLI Task List

This checklist is meant to be explicit enough for an AI implementation agent.
Complete phases in order; each phase unlocks the next.

## AI Execution Order (Suggested)

1. Phase 0: lock v1 scope, defaults, and standards inputs
2. Phase 1: create crate skeletons and build plumbing
3. Phase 2: decide dependencies, logging, and CLI UI spec
4. Phase 3: implement core data contracts
5. Phase 4: implement standards loaders from `standards/`
6. Phase 5: implement ingest and domain discovery
7. Phase 6: implement mapping engine and config
8. Phase 7: implement domain processing and relationship generation
9. Phase 8: implement conformance validation
10. Phase 9: implement output writers
11. Phase 10: wire CLI and summary output
12. Phase 11: parity tests and QA
13. Phase 12: release docs and CI

## Phase 0 - Scope and Inputs

Goal: lock the v1 surface area and inputs so implementation is deterministic.

- [x] v1 outputs: XPT + Dataset-XML + Define-XML; SAS scripts deferred (no CLI flag)
- [x] v1 CLI flags: `--output-dir`, `--format`, `-v`
- [x] No config file in v1; defaults are compiled
- [x] Standards sources under `standards/` (ct, sdtm, sdtmig, p21, xsl)
- [x] Compiled defaults:
  - output_format = both
  - min_confidence = 0.5
  - chunk_size = 1000
  - sdtm_version = 2.0
  - sdtmig_version = 3.4
  - dataset_xml_version = 1.0
  - define_xml_version = 2.1

Exit criteria:

- [x] v1 scope is documented and agreed
- [x] Defaults are enumerated in this file

## Phase 1 - Workspace and Crate Skeletons

Goal: create buildable crates with clear boundaries.

- [x] Create workspace crates under `crates/` per `Cargo.toml`
- [x] Add `lib.rs` + minimal module layout per crate
- [x] Define shared error types and result aliases
- [x] Wire `tracing` + `tracing-subscriber` with `-v` verbosity mapping
- [x] Set up `cargo fmt` and `cargo clippy` configuration

Exit criteria:

- [x] `cargo check` passes for all crates
- [x] Logging initializes without panics

## Phase 2 - Dependencies, Logging, and CLI UI Decisions

Goal: make dependency and UX choices explicit for deterministic output.

- [x] Use CLI crate: `clap` (derive)
- [x] Use CSV crate: `csv`
- [x] Use XML writing crate: `quick-xml`
- [x] Use table UI crate: `comfy-table`
- [x] Use progress/spinner crate: none for v1
- [x] Use logging crates: `tracing`, `tracing-subscriber`
- [x] Prune config-related crates (remove `figment`, `toml`)
- [x] Log policy: default info, `-v` => debug, `-vv` => trace, no timestamps
- [x] Log structure: study_id and domain_code as span fields
- [x] Output policy: stdout for summary, stderr for logs
- [x] CLI output spec (v1):
  - Summary table columns: Domain, Description, Records, XPT, Dataset-XML, Notes
  - Totals line at bottom with total record count
  - Error section lists domain code + message
  - Output paths shown after summary
- [x] Symbols: use ASCII markers (OK/WARN/ERR), no Unicode required
- [x] Color policy: color on TTY, plain text fallback

Exit criteria:

- [x] Dependencies are listed in `Cargo.toml` and justified
- [x] CLI output spec is documented in this file

## Phase 3 - Core Data Contracts (sdtm-model)

Goal: define the types used by every other crate.

- [x] Define `Domain`, `Variable`, `DatasetMetadata`
- [x] Define `ControlledTerminology` and registry types
- [x] Define `MappingConfig`, `MappingSuggestion`, `ColumnHint`
- [x] Define `ConformanceIssue` and `ConformanceReport`
- [x] Define `ProcessStudyRequest/Response` and per-domain results
- [x] Add serde derives for structured outputs

Exit criteria:

- [x] Types compile and are used by downstream crates
- [x] Unit tests cover serialization and basic invariants

## Phase 4 - Standards Loading (sdtm-standards)

Goal: deterministic loading of SDTM/SDTMIG/CT/P21 assets from `standards/`.

- [ ] Load SDTMIG datasets and variables from `standards/sdtmig/v3_4/`
- [ ] Load SDTM datasets/metadata from `standards/sdtm/`
- [ ] Load Controlled Terminology from `standards/ct/`
- [ ] Load Pinnacle 21 rules from `standards/p21/Rules.csv`
- [ ] Wire optional XSL assets from `standards/xsl/` for Define-XML
- [ ] Add unit tests for each loader and registry lookup behavior

Exit criteria:

- [ ] All standards loaders return deterministic, sorted outputs
- [ ] Missing files fail fast with clear errors

## Phase 5 - Ingest and Discovery (sdtm-ingest)

Goal: read source CSVs consistently and discover domains.

- [ ] Implement CSV reader with stable null/empty handling
- [ ] Normalize column names and whitespace
- [ ] Build column hints (numeric, null ratio, uniqueness)
- [ ] Implement domain discovery rules (skip metadata/helper files)
- [ ] Enforce deterministic ordering of input files

Exit criteria:

- [ ] Domain discovery matches Python behavior on mockdata
- [ ] Unit tests cover discovery edge cases

## Phase 6 - Mapping Engine (sdtm-map)

Goal: map source columns to SDTM variables deterministically.

- [ ] Implement alias pattern builder
- [ ] Port fuzzy matching and scoring rules
- [ ] Apply hint adjustments (numeric mismatch, SEQ uniqueness, null ratio)
- [ ] Build mapping suggestions and a stable mapping config
- [ ] Unit test mapping parity vs Python behavior

Exit criteria:

- [ ] Mapping is deterministic given the same inputs
- [ ] Minimum confidence handling matches v1 defaults

## Phase 7 - Domain Processing (sdtm-core)

Goal: transform input data into SDTM frames per domain.

Principle: never synthesize data; only generate derived relationship and
supporting domains from available source data.

- [ ] Implement domain frame builder (types, ordering, lengths)
- [ ] Implement base processor behaviors (USUBJID handling, study prefix)
- [ ] Port domain processors in priority order (DM, AE, CM, DS, EX, LB, MH, PR,
      QS, SE, TA, TE, TS, VS, DA, IE, PE)
- [ ] Generate SUPPQUAL from source-mapped data (non-LB domains)
- [ ] Generate relationship domains (RELREC, RELSPEC, RELSUB) from available
      data
- [ ] Add regression tests for domain-specific transforms

Exit criteria:

- [ ] End-to-end domain processing works for mockdata
- [ ] Relationship/supporting domains are generated only when inputs support
      them

## Phase 8 - Validation (sdtm-validate)

Goal: enforce conformance and CT validation with deterministic reporting.

- [ ] Implement required/expected/permissible variable checks
- [ ] Enforce type and length constraints
- [ ] Validate CT submission values and synonyms
- [ ] Apply P21 rules with error/warn severity mapping
- [ ] Emit conformance report JSON schema
- [ ] Gate strict outputs on conformance errors

Exit criteria:

- [ ] Conformance report matches Python structure
- [ ] Gating behavior matches v1 spec

## Phase 9 - Output Writers (sdtm-report)

Goal: generate files with the same layout and constraints as Python.

- [ ] Write XPT files (8-char dataset names, labels, type coercion)
- [ ] Write Dataset-XML v1.0 (deterministic order, streaming option)
- [ ] Generate Define-XML v2.1 from dataset metadata
- [ ] Generate SAS scripts (deterministic)
- [ ] Preserve output layout: `output/xpt`, `output/dataset-xml`, `output/sas`

Exit criteria:

- [ ] Output files are deterministic and parseable
- [ ] Output paths follow the expected layout

## Phase 10 - CLI Wiring (sdtm-cli)

Goal: provide a minimal but stable CLI entry point.

- [ ] Implement `study` command with `--output-dir`, `--format`, `-v`
- [ ] Implement `domains` command
- [ ] Render a summary table and error details per the CLI spec
- [ ] Map verbosity flags to log levels
- [ ] Use non-zero exit codes on failure
- [ ] Add `--dry-run` support (optional)

Exit criteria:

- [ ] CLI runs end-to-end on mockdata
- [ ] Summary output matches the CLI spec

## Phase 11 - Parity and QA

Goal: prove correctness and performance against the Python reference.

- [ ] Build parity harness (Python vs Rust outputs)
- [ ] Define tolerances for ordering and timestamps
- [ ] Add golden tests for `mockdata/` studies
- [ ] Establish performance baselines and regression checks

Exit criteria:

- [ ] Parity tests pass within defined tolerances
- [ ] Performance baselines are captured

## Phase 12 - Release and Docs

Goal: make the Rust CLI usable and shippable.

- [ ] Update README with Rust CLI usage
- [ ] Add CI workflows for `cargo test`, `cargo clippy`, and benches
- [ ] Provide release build scripts and artifacts
- [ ] Document migration notes from Python CLI

Exit criteria:

- [ ] CI runs for tests and linting
- [ ] Release artifacts can be built locally

## Cross-Cutting Tasks

- [ ] Ensure deterministic results across platforms
- [ ] Maintain offline-only behavior (no network at runtime)
- [ ] Track known deviations from Python behavior
