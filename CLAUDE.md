# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CDISC Transpiler is a Rust CLI tool for converting clinical trial source data into CDISC SDTM formats (XPT, Dataset-XML,
Define-XML). It operates fully offline with committed standards and controlled terminology.

## Build & Test Commands

```bash
cargo build                        # Debug build
cargo build --release              # Release build
cargo test                         # All tests
cargo test --package sdtm-core     # Single crate tests
cargo test test_name               # Single test by name
cargo clippy                       # Linting (MSRV 1.92)
cargo fmt                          # Format code
```

## Architecture

### Crate Dependency Graph

```
sdtm-cli (entry point)
├── sdtm-core (orchestration, domain processing)
│   ├── sdtm-ingest (CSV → DataFrame)
│   ├── sdtm-map (fuzzy column mapping)
│   └── sdtm-validate (conformance checks)
├── sdtm-report (XPT, Dataset-XML, Define-XML output)
│   ├── sdtm-xpt (SAS Transport format)
│   └── sdtm-standards (CT/SDTM loaders)
└── sdtm-model (pure types, no dependencies)
```

### Crate Responsibilities

| Crate            | Purpose                                                                                                               |
|------------------|-----------------------------------------------------------------------------------------------------------------------|
| `sdtm-model`     | Types only (Domain, Variable, Term, Codelist, ValidationIssue). No I/O.                                               |
| `sdtm-cli`       | CLI parsing, logging, dependency wiring. No business logic.                                                           |
| `sdtm-core`      | Business logic: USUBJID prefixing, --SEQ assignment, CT normalization, domain-specific rules in `domain_processors/`. |
| `sdtm-standards` | Load SDTM/CT from offline CSV files in `standards/`.                                                                  |
| `sdtm-validate`  | Conformance gating (block XPT output if errors exist).                                                                |
| `sdtm-ingest`    | CSV discovery, parsing, schema detection.                                                                             |
| `sdtm-map`       | Fuzzy column mapping using rapidfuzz.                                                                                 |
| `sdtm-report`    | Multi-format output generation.                                                                                       |
| `sdtm-xpt`       | SAS Transport v5 format writer.                                                                                       |

### Key Design Patterns

- **Offline-first**: All standards committed in `standards/` directory
- **Validation-gating**: Errors can block stricter output formats (XPT)
- **Case-insensitive matching**: Use `CaseInsensitiveSet` for column/variable lookups
- **Immutable context**: `PipelineContext` carries read-only study metadata
- **Domain-specific rules**: Each domain's business logic in `crates/sdtm-core/src/domain_processors/`

### Processing Pipeline

1. **Discovery** - Find CSV files in study folder
2. **Ingest** - Load source data into Polars DataFrames
3. **Mapping** - Match source columns to SDTM variables
4. **Domain Processing** - Apply transformations (USUBJID prefix, --SEQ, CT normalization)
5. **Validation** - Conformance checks (CT values, required variables)
6. **Gating** - Decide outputs based on validation results
7. **Report Generation** - Write XPT, Dataset-XML, Define-XML

## Standards Directory

Offline-committed CDISC standards (source of truth):

```
standards/
├── ct/                    # Controlled Terminology CSVs by version
├── sdtmig/v3_4/
│   ├── Datasets.csv       # Domain metadata
│   ├── Variables.csv      # Variable definitions with CT codelist codes
│   └── chapters/          # SDTMIG v3.4 chapter documentation
└── sdtm/                  # SDTM model specifications
```

**Before implementing any SDTM rule, read the relevant section in `standards/sdtmig/v3_4/chapters/` to verify the
requirement.**

## Key Types

```rust
Domain { code, class, label, variables }
Variable { name, label, data_type, role, core, ct_codelist }
Term { code, submission_value, synonyms }
TerminologyRegistry { catalogs: HashMap<String, TerminologyCatalog> }
ValidationIssue { severity, code, variable, message }
Severity { Error, Warning, Info }
CoreDesignation { Required, Expected, Permissible }
```

## Naming Conventions

See `docs/NAMING_CONVENTIONS.md` for SDTM ↔ Rust terminology mapping.

- Use "terminology" in public APIs, "CT" only internally
- Use `Severity` enum (Error/Warning/Info), not strings
- Distinguish `Domain` (metadata) from `Dataset` (DataFrame)

## Commit Style

Use conventional commits: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `perf:`, `chore:`
