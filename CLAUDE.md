# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CDISC Transpiler is a Rust CLI tool that transforms clinical trial source data (CSV) into CDISC SDTM format (XPT, Dataset-XML, Define-XML). It emphasizes strict conformance to SDTMIG v3.4, deterministic output, and offline operation with all standards committed in the repository.

## Essential Commands

```bash
# Build
cargo build                     # Debug build
cargo build --release           # Production build

# Test
cargo test                      # All tests
cargo test -p sdtm-ingest       # Single crate
cargo test csv_table            # Specific test
cargo test -- --nocapture       # With output

# Lint and format
cargo fmt                       # Format (required)
cargo clippy                    # Lint (address all warnings)

# Run
cargo run -- study <study_folder> --output-dir <dir>
cargo run -- domains            # List supported domains
```

## Architecture

### Workspace Crates (9 crates)

```
sdtm-cli       → CLI entry point, pipeline orchestration
sdtm-core      → Domain processing, SUPPQUAL, relationships
sdtm-ingest    → CSV discovery, metadata loading
sdtm-map       → Column mapping engine (fuzzy matching)
sdtm-model     → Core types (Domain, Variable, Codelist, Term)
sdtm-standards → Load SDTMIG/CT from committed standards/
sdtm-validate  → CT-based conformance validation
sdtm-report    → XPT/Dataset-XML/Define-XML writers
sdtm-xpt       → SAS Transport format handling
```

### Pipeline Stages

1. **Ingest**: Discover CSV files, read metadata
2. **Map**: Apply column mappings to SDTM variables
3. **Preprocess**: Extract reference dates, fill missing fields
4. **Domain Rules**: Per-domain processing (DM, AE, LB, etc.)
5. **SUPPQUAL**: Build supplemental qualifier datasets
6. **Relationships**: Build RELREC/RELSPEC/RELSUB
7. **Validate**: Check CT values against TerminologyRegistry
8. **Output**: Write XPT, Dataset-XML, Define-XML

### Key Entry Points

- CLI: `crates/sdtm-cli/src/main.rs` → `cli.rs` → `commands.rs`
- Pipeline: `crates/sdtm-cli/src/pipeline.rs`
- Domain processors: `crates/sdtm-core/src/domain_processors/<domain>.rs`
- Validation: `crates/sdtm-validate/src/lib.rs`
- Output writers: `crates/sdtm-report/src/lib.rs`

## Non-Negotiable Constraints

**Read AGENTS.md fully before making changes.** Key constraints:

- **Never fabricate SDTM rules**: Always cite SDTMIG v3.4 chapter/section
- **Strict medical data handling**: Treat as PHI; no row-level logging
- **Required fields**: STUDYID/USUBJID/DOMAIN/--SEQ must exist; never auto-fill
- **CT must be exact**: Normalized values must match submission_value or defined synonyms
- **No silent mutations**: Preserve data provenance, explicit validation errors
- **Tests in `tests/` folder**: Not inline `#[cfg(test)]`
- **Use `tracing`**: Not `println!` for logging

## Code Style

- Rust Edition 2024, MSRV 1.92
- `cargo fmt` and `cargo clippy` are required
- Avoid abbreviations: no `ctx`, `df`, `cfg`, `val`, `proc`
- Use Polars expressions for bulk transforms (avoid per-row loops)

## Key Dependencies

- **polars**: Data processing (DataFrame)
- **clap**: CLI argument parsing
- **tracing/tracing-subscriber**: Structured logging
- **quick-xml**: XML output generation
- **rapidfuzz**: Fuzzy string matching (mapping)
- **anyhow/thiserror**: Error handling

## Where to Find Things

| Need | Location |
|------|----------|
| Domain rules | `crates/sdtm-core/src/domain_processors/<domain>.rs` |
| CT matching | `crates/sdtm-core/src/ct_utils.rs` |
| Validation | `crates/sdtm-validate/src/lib.rs` |
| Output formats | `crates/sdtm-report/src/lib.rs` |
| CLI args | `crates/sdtm-cli/src/cli.rs` |
| Pipeline | `crates/sdtm-cli/src/pipeline.rs` |
| Core types | `crates/sdtm-model/src/` |
| Standards data | `standards/SDTMIG_v3.4/`, `standards/Controlled_Terminology/` |

## Adding New Domain Processors

1. Create `crates/sdtm-core/src/domain_processors/<domain>.rs`
2. Register in `crates/sdtm-core/src/domain_processors/mod.rs`
3. Follow existing pattern: normalize CT → apply rules → build --SEQ → validate
4. Add test in `crates/sdtm-core/tests/`

## Test Data

- Mock study data: `mockdata/DEMO_GDISC_20240903_072908/`
- MSG sample: `docs/SDTM-MSG_v2.0/`
- Use parity tests against MSG outputs for validation
