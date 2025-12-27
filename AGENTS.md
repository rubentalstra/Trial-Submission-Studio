# AGENTS.md

## Project intent
This repo is a strict SDTM transpiler. Treat all data as medical data: never invent values, never silently mutate collected data, and always preserve provenance. Output must match SDTMIG and MSG conventions exactly.

## Sources of truth
- SDTMIG assumptions: `standards/sdtmig/v3_4/chapters`
- SDTMIG metadata tables: `standards/sdtmig/v3_4/Datasets.csv`, `standards/sdtmig/v3_4/Variables.csv`
- Controlled terminology: `docs/Controlled_Terminology/`
- Define-XML spec: `docs/Define-XML_2.1/`
- Dataset-XML spec: `docs/Dataset-XML_1-0/`
- MSG v2.0 golden standard: `docs/SDTM-MSG_v2.0_Sample_Submission_Package/`

## Non-negotiable rules
- Strict mode only: missing Required/Expected data must error, not auto-fill.
- No imputation or heuristic fills without explicit mapping/derivation rules.
- CT normalization must be exact or explicit synonym mapping; record dictionary metadata in Define-XML.
- Never rename or repurpose standard SDTM variables; put sponsor variables in SUPPQUAL.
- Enforce split-domain rules and --SEQ uniqueness across splits.
- For XPT, reject non-ASCII values in strict mode.
- Never drop records or auto-correct dates silently; emit explicit validation errors and preserve raw values.
- Required identifiers (STUDYID/USUBJID/DOMAIN/--SEQ) must be present for GO domains; do not fabricate.

## Determinism and output parity
- Outputs must be deterministic (ordering, lengths, timestamps driven by config or fixed values).
- Align Define-XML/Dataset-XML/XPT with MSG sample conventions and directory layout.
- Use standards tables for variable order, roles, core designations, and dataset class.
- Preserve dataset naming rules for split datasets and SUPP naming (no auto-labeling when metadata is missing).

## Pipeline and architecture
- Keep stages explicit: ingest -> map -> preprocess -> domain rules -> validation -> outputs.
- Domain processors live in `crates/sdtm-core/src/domain_processors/` and must be per-domain, minimal nesting, and share common utilities.
- Relationship datasets (RELREC/RELSPEC/RELSUB/SUPPQUAL) must be explicit-only; never inferred from incidental data.
- Prefer Polars expressions for bulk transforms; avoid per-row loops unless necessary.

## Logging and observability
- Use `tracing` + `tracing-subscriber` for structured logs.
- Include `study_id`, `domain_code`, `dataset_name`, and source file path in spans.
- Default to redacted logs; require an explicit flag to log row-level values.
- Log counts and durations per pipeline stage, not raw data, unless explicitly enabled.

## Workspace and dependencies
- Use workspace dependencies from `Cargo.toml`; avoid new crates unless required and justified.
- Run commands from the repo root to ensure workspace settings apply.

## Tests and verification
- After any code edit, run `cargo fmt` then `cargo clippy` and address warnings.
- Prefer `cargo test` from workspace root for changes.
- Add parity tests against MSG sample outputs when modifying Define-XML/Dataset-XML/XPT.
- Keep outputs deterministic (timestamps, ordering, lengths).

## Working practices
- Prefer `rg` for file and text search.
- If requirements change, update `docs/SUGGESTED_CODE_CHANGES.md`.
- Keep edits ASCII unless a file already uses non-ASCII characters.
