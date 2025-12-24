# CDISC Transpiler (Rust) - Rebuild Plan and Strategy

## Purpose

This document replaces the placeholder plan and serves as the single source of
truth for the Rust-first rebuild. It is grounded in the current Python CLI
behavior and sets the target architecture, milestones, and validation strategy
needed to ship a production-ready Rust CLI.

## Python CLI Deep Dive (Parity Requirements)

The current Python CLI is the functional spec we must match or intentionally
supersede. Key behaviors and expectations:

- Entry point: `cdisc_transpiler.cli:app` (Click group) with commands:
  - `study` for full processing
  - `domains` for listing supported SDTM domains
- `study` command options and defaults (Rust v1 keeps this minimal):
  - `study_folder` argument (CSV files in folder)
  - `--output-dir` (default `<study_folder>/output`)
  - `--format` (`xpt`, `xml`, `both`) default `both`
  - `-v` / `-vv` for verbosity
- Advanced flags (Define-XML, SAS, streaming, conformance gating) are deferred
  until later phases.
- Configuration:
  - No config file in Rust v1; defaults are compiled and stable.
- Domain discovery:
  - Scans `*.csv` in study folder
  - Skips metadata and helper files (`CODELISTS`, `README`, `_LC`, etc)
  - Matches domain by exact token or prefix variant (e.g., `AE`, `AE_1`, `LB*`)
- Domain processing pipeline (per domain, AE first):
  - Read CSV
  - Build column hints (numeric, null ratio, uniqueness)
  - Mapping engine uses fuzzy matching (RapidFuzz) + alias patterns
  - Domain frame builder enforces spec order and lengths
  - Domain processors apply domain-specific transforms
  - SUPPQUAL generation for non-LB domains
  - Special cases: LB de-duplication, TS parameter label fill
- Synthesis pass (if missing): RELREC, RELSUB, RELSPEC scaffolds
- Validation gating:
  - Conformance report per domain, CT checks
  - If strict outputs (XPT or SAS) and `fail-on-conformance-errors`, outputs are
    blocked
- Outputs:
  - XPT (SAS V5) with column ordering and types
  - Dataset-XML v1.0
  - Define-XML v2.1 (uses gathered dataset metadata)
  - SAS program files
  - Output layout: `output/xpt`, `output/dataset-xml`, `output/sas`
  - Conformance report JSON in output dir
- Summary output:
  - Rich table with domain descriptions, record counts, output presence
  - Non-zero errors cause CLI failure

## Rust CLI Goals and Best Practices

- Start with a minimal CLI surface area; expand to parity only when needed.
- Implement strict, typed CLI args with defaults (clap + serde).
- Provide deterministic output ordering and stable file naming.
- Keep offline operation and standards assets under version control.
- Separate business logic from I/O and CLI wiring.
- Use structured logging (`tracing`) with verbosity mapping to `-v`.
- Provide explicit exit codes and clear error messages.
- Gate outputs on validation results exactly as Python does.

## Target Architecture (Workspace Crates)

Planned workspace layout (mirrors existing `Cargo.toml` intent):

- `sdtm-cli`
  - CLI parsing, dependency wiring
- `sdtm-model`
  - Core data types (domain definitions, variables, mappings, DTOs)
- `sdtm-standards`
  - Load SDTMIG/SDTM spec CSVs and CT CSVs from `standards/`
- `sdtm-ingest`
  - CSV ingestion, dataset normalization, streaming support
- `sdtm-map`
  - Column mapping engine, patterns, fuzzy matching
- `sdtm-core`
  - Use cases: study processing, domain processing, synthesis
- `sdtm-validate`
  - Conformance checks, CT validation, XPT constraints
- `sdtm-report`
  - Output writers (XPT, Dataset-XML, Define-XML, SAS)
  - Summary and conformance report JSON

## Rust CLI Specification

Commands (minimal initial surface):

- `cdisc-transpiler study <study_folder>`
  - Minimal flags: `--output-dir`, `--format`, `-v`
  - Advanced behavior is added later
- `cdisc-transpiler domains`
  - List supported domains and descriptions

Compatibility choices:

- Keep `cdisc-transpiler` binary name for continuity.
- Exit code non-zero on errors (conformance or processing failures).

## Data and Standards Strategy

- Standards and CT are committed under `standards/` (source of truth).
- Rust loaders must read from `standards/` to ensure offline parity.
- No runtime network access; updates are explicit repo changes.

## Validation Strategy

- Domain conformance check:
  - Required/expected/permissible variables
  - Type and length enforcement
  - CT checks (submission values and synonyms)
- Output gates:
  - If strict outputs are requested (XPT/SAS), block output on errors
- Report:
  - JSON conformance report per run
  - Summary and error details in CLI output

## Output Strategy

- Preserve the Python output directory layout (`xpt/`, `dataset-xml/`, `sas/`).
- XPT writer:
  - 8-character dataset names
  - Spec ordering and label length truncation
  - Correct numeric vs char handling
- Dataset-XML v1.0:
  - Deterministic ordering
  - Streaming mode for large datasets
- Define-XML v2.1:
  - Derived from domain dataframes and spec metadata
- SAS scripts:
  - Deterministic mapping scripts with provenance annotations

## Testing Strategy

- Unit tests:
  - Mapping engine, domain processors, transformers
- Integration tests:
  - End-to-end runs against `mockdata/`
- Golden tests:
  - Snapshot outputs for XPT/XML/Define-XML and reports
- Validation tests:
  - CT and conformance rules
- Performance tests:
  - Study-sized benchmarks and hot-path microbenchmarks

## Migration Plan (Phased)

Phase 0 - Bootstrap

- Create workspace crates and shared config types
- Establish logging, error handling, and CLI skeleton

Phase 1 - Standards and Models

- Port SDTM/SDTMIG loaders and CT loaders
- Define core domain and mapping data structures

Phase 2 - Ingest and Mapping

- CSV ingestion with schema inference
- Fuzzy mapping engine and hint scoring
- Mapping configuration and deterministic outputs

Phase 3 - Domain Processing

- Implement domain processors and transformations
- SUPPQUAL generation
- RELREC/RELSUB/RELSPEC synthesis

Phase 4 - Validation and Outputs

- Conformance checks, CT validation
- XPT, Dataset-XML, Define-XML, SAS generation
- Conformance report JSON

Phase 5 - Parity and Hardening

- Parity test suite vs Python outputs
- Performance baseline and regression checks
- CLI polish, docs, and release automation

## Definition of Done

- Rust CLI can process all supported domains in `mockdata/`
- Outputs match Python for XPT/XML/Define-XML (within known tolerances)
- Conformance gating behaves identically
- Full offline operation with committed standards
- CI validates unit/integration/golden/bench suites

## Related Documents

- Task tracker: `docs/RUST_CLI_TASKS.md`
- Legacy Python CLI behavior: `cdisc_transpiler/cli/` and use cases in
  `cdisc_transpiler/application/`
