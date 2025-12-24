# Rust CLI Task List

This task list tracks the work required to replace the Python CLI with a
production-grade Rust CLI. It mirrors the phases in `docs/REFRACTOR_PLAN.md`.

## Phase 0 - Bootstrap

- [ ] Confirm final CLI binary name (`cdisc-transpiler` preferred for parity)
- [ ] Create workspace crates under `crates/` per `Cargo.toml`
- [ ] Add common error and logging utilities
- [ ] Wire `tracing` + `tracing-subscriber` with `-v` verbosity mapping
- [ ] Set up `cargo fmt` and `cargo clippy` configuration

## Phase 1 - Standards and Core Models

- [ ] Define core data models (domain, variable, mapping config, results)
- [ ] Implement SDTM spec CSV loaders (domain variables, dataset metadata)
- [ ] Implement Controlled Terminology loaders from `standards/ct`
- [ ] Build registries for lookup by codelist code and name
- [ ] Add unit tests for standards parsing and registry behavior

## Phase 2 - Ingest and Mapping

- [ ] Implement CSV reader with consistent null handling
- [ ] Add schema inference and column hint extraction
- [ ] Implement domain discovery (CSV matching, skip metadata/helpers)
- [ ] Port fuzzy matching (RapidFuzz or equivalent) and alias patterns
- [ ] Implement mapping suggestions and mapping config builder
- [ ] Create deterministic column ordering strategy
- [ ] Unit test mapping engine parity against Python behavior

## Phase 3 - Domain Processing

- [ ] Implement domain frame builder (type normalization, spec ordering)
- [ ] Implement base domain processor behaviors:
  - [ ] USUBJID placeholder handling
  - [ ] Study ID prefixing
- [ ] Port domain processors (priority order):
  - [ ] DM
  - [ ] AE
  - [ ] CM
  - [ ] DS
  - [ ] EX
  - [ ] LB
  - [ ] MH
  - [ ] PR
  - [ ] QS
  - [ ] SE
  - [ ] TA
  - [ ] TE
  - [ ] TS
  - [ ] VS
  - [ ] DA
  - [ ] IE
  - [ ] PE
- [ ] Implement SUPPQUAL generation
- [ ] Implement RELREC/RELSUB/RELSPEC synthesis pass
- [ ] Add regression tests for domain-specific transformations

## Phase 4 - Validation and Outputs

- [ ] Implement conformance checks (required/expected/permissible)
- [ ] Add CT validation using codelist registry
- [ ] Port XPT writer (8 char names, labels, type coercion)
- [ ] Port Dataset-XML writer with streaming option
- [ ] Port Define-XML generator (Define-XML 2.1)
- [ ] Port SAS program writer (deterministic, optional)
- [ ] Write conformance report JSON
- [ ] Add validation tests and fixture outputs

## Phase 5 - CLI and UX

- [ ] Implement `study` command with minimal flags (`--output-dir`, `--format`, `-v`)
- [ ] Implement `domains` command
- [ ] Implement summary table output
- [ ] Handle exit codes for failure states
- [ ] Add `--dry-run` support (optional but useful)
- [ ] Defer advanced CLI flags to later phases

## Phase 6 - Parity and QA

- [ ] Build parity test harness (Python vs Rust outputs)
- [ ] Define acceptable tolerances (ordering, whitespace, timestamps)
- [ ] Add golden tests for mock studies
- [ ] Establish performance baselines and regression checks

## Phase 7 - Release Readiness

- [ ] Update README with Rust CLI usage and installation
- [ ] Add CI workflow for `cargo test`, `cargo clippy`, and benchmarks
- [ ] Provide binary release build scripts
- [ ] Document upgrade path from Python CLI

## Cross-Cutting Tasks

- [ ] Define stable output directory layout
- [ ] Ensure deterministic results across platforms
- [ ] Track known deviations from Python behavior
