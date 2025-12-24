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

- [ ] Confirm v1 outputs: XPT, Dataset-XML, Define-XML, SAS
- [ ] Confirm v1 CLI flags: `--output-dir`, `--format`, `-v`
- [ ] Confirm no config file in v1; defaults are compiled
- [ ] Verify standards sources under `standards/` (`ct`, `sdtm`, `sdtmig`,
      `p21`, `xsl`)
- [ ] Define compiled defaults (min confidence, chunk size, default date)

Exit criteria:

- [ ] v1 scope is documented and agreed
- [ ] Defaults are enumerated in this file

## Phase 1 - Workspace and Crate Skeletons

Goal: create buildable crates with clear boundaries.

- [ ] Create workspace crates under `crates/` per `Cargo.toml`
- [ ] Add `lib.rs` + minimal module layout per crate
- [ ] Define shared error types and result aliases
- [ ] Wire `tracing` + `tracing-subscriber` with `-v` verbosity mapping
- [ ] Set up `cargo fmt` and `cargo clippy` configuration

Exit criteria:

- [ ] `cargo check` passes for all crates
- [ ] Logging initializes without panics

## Phase 2 - Dependencies, Logging, and CLI UI Decisions

Goal: make dependency and UX choices explicit for deterministic output.

- [ ] Use CLI crate: `clap` (derive)
- [ ] Use CSV crate: `csv`
- [ ] Use XML writing crate or strategy: `quick-xml` or custom writer
- [ ] Use table UI crate: `comfy-table` or `tabled`
- [ ] Use progress/spinner crate: `indicatif` or none
- [ ] Use logging crates: `tracing`, `tracing-subscriber`
- [ ] Prune config-related crates if unused (remove `figment`, `toml` if no
      config)
- [ ] Define log policy: levels, default level, `-v` mapping, and log format
- [ ] Define log structure: study/domain spans and key fields
- [ ] Define output policy: stdout for summary, stderr for logs
- [ ] Define CLI output spec: table columns, totals line, error sections
- [ ] Define summary table columns (Domain, Description, Records, XPT,
      Dataset-XML, SAS, Notes)
- [ ] Define symbols and ASCII fallback (checkmarks, warnings, errors)
- [ ] Define color policy: color on TTY, plain text fallback

Exit criteria:

- [ ] Dependencies are listed in `Cargo.toml` and justified
- [ ] CLI output spec is documented in this file

## Phase 3 - Core Data Contracts (sdtm-model)

Goal: define the types used by every other crate.

- [ ] Define `Domain`, `Variable`, `DatasetMetadata`
- [ ] Define `ControlledTerminology` and registry types
- [ ] Define `MappingConfig`, `MappingSuggestion`, `ColumnHint`
- [ ] Define `ConformanceIssue` and `ConformanceReport`
- [ ] Define `ProcessStudyRequest/Response` and per-domain results
- [ ] Add serde derives for structured outputs

Exit criteria:

- [ ] Types compile and are used by downstream crates
- [ ] Unit tests cover serialization and basic invariants

## Phase 4 - Standards Loading (sdtm-standards)

Goal: deterministic loading of SDTM/SDTMIG/CT/P21 assets from `standards/`.

- [ ] Load SDTMIG datasets and variables from `standards/sdtmig/v3_4/`
- [ ] Load SDTM datasets/metadata from `standards/sdtm/`
- [ ] Load Controlled Terminology from `standards/ct/`
- [ ] Load Pinnacle 21 rules from `standards/p21/rules.csv`
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
