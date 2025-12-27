# AGENTS.md

## Project overview

This repo is a strict SDTM transpiler. Treat all data as medical data: never
invent values, never silently mutate collected data, and always preserve
provenance. Output must match SDTMIG and MSG conventions exactly.

## Sources of truth

- SDTMIG assumptions: `standards/sdtmig/v3_4/chapters`
- SDTMIG metadata tables: `standards/sdtmig/v3_4/Datasets.csv`,
  `standards/sdtmig/v3_4/Variables.csv`
- Controlled terminology: `docs/Controlled_Terminology/`
- Define-XML spec: `docs/Define-XML_2.1/`
- Dataset-XML spec: `docs/Dataset-XML_1-0/`
- MSG v2.0 golden standard: `docs/SDTM-MSG_v2.0_Sample_Submission_Package/`

### SDTMIG v3.4 Chapter Reference

The full SDTMIG v3.4 specification is available in Markdown format:

- `chapter_02_fundamentals-of-the-sdtm.md` - Core concepts
- `chapter_03_submitting-data-in-standard-format.md` - Submission metadata
- `chapter_04_assumptions-for-domain-models.md` - Variable rules, timing, text
- `chapter_05_models-for-special-purpose-domains.md` - DM, SE, SV, CO
- `chapter_06_domain-models-based-on-the-general-observation-classes.md` -
  Findings, Events, Interventions
- `chapter_07_trial-design-model-datasets.md` - TA, TE, TV, TI, TS, TD, TM
- `chapter_08_representing-relationships-and-data.md` - RELREC, RELSPEC, RELSUB,
  SUPPQUAL
- `chapter_09_study-references.md` - DI, OI
- `chapter_10_appendices.md` - CT, naming fragments, QNAM codes

**Always verify rules against these chapters before implementing.**

## Build and test commands

- Run from repo root to use workspace settings.
- After any code edit, run `cargo fmt` then `cargo clippy` and address warnings.
- Use `cargo build` for quick compile checks when needed.

## Code style guidelines

- Keep edits ASCII unless a file already uses non-ASCII characters.
- Prefer explicit, readable transformations over deeply nested helper chains.
- Refactor to remove redundant legacy wrappers (simple pass-through helpers);
  call shared utilities directly when no extra behavior exists.
- Use `tracing` for logs; avoid `println!` in production paths.
- Favor Polars expressions for batch transforms; avoid row-by-row loops when
  possible.

## Security considerations

- Treat all input data as PHI/PII; do not log raw values by default.
- Do not copy sample subject data into new files outside `docs/` or `tests/`.
- Avoid sending data to external services or downloading external assets without
  approval.

## Non-negotiable rules

- **Never fabricate SDTM rules**: always verify against
  `standards/sdtmig/v3_4/chapters/` before implementing any rule or validation.
- **Cite chapter/section** in code comments when implementing SDTMIG rules.
- Strict mode only: missing Required/Expected data must error, not auto-fill.
- No imputation or heuristic fills without explicit mapping/derivation rules.
- CT normalization must be exact or explicit synonym mapping; record dictionary
  metadata in Define-XML.
- Never rename or repurpose standard SDTM variables; put sponsor variables in
  SUPPQUAL.
- Enforce split-domain rules and --SEQ uniqueness across splits.
- For XPT, reject non-ASCII values in strict mode.
- Never drop records or auto-correct dates silently; emit explicit validation
  errors and preserve raw values.
- Required identifiers (STUDYID/USUBJID/DOMAIN/--SEQ) must be present for GO
  domains; do not fabricate.
- Do not invent variable constraints, CT values, or domain rules not explicitly
  documented in the standards.

## Determinism and output parity

- Outputs must be deterministic (ordering, lengths, timestamps driven by config
  or fixed values).
- Align Define-XML/Dataset-XML/XPT with MSG sample conventions and directory
  layout.
- Use standards tables for variable order, roles, core designations, and dataset
  class.
- Preserve dataset naming rules for split datasets and SUPP naming (no
  auto-labeling when metadata is missing).

## Pipeline and architecture

- Keep stages explicit: ingest -> map -> preprocess -> domain rules ->
  validation -> outputs.
- Domain processors live in `crates/sdtm-core/src/domain_processors/` and must
  be per-domain, minimal nesting, and share common utilities.
- Relationship datasets (RELREC/RELSPEC/RELSUB/SUPPQUAL) must be explicit-only;
  never inferred from incidental data.
- Prefer Polars expressions for bulk transforms; avoid per-row loops unless
  necessary.

## Logging and observability

- Use `tracing` + `tracing-subscriber` for structured logs.
- Include `study_id`, `domain_code`, `dataset_name`, and source file path in
  spans.
- Default to redacted logs; require an explicit flag to log row-level values.
- Log counts and durations per pipeline stage, not raw data, unless explicitly
  enabled.

## Workspace and dependencies

- Use workspace dependencies from `Cargo.toml`; avoid new crates unless required
  and justified.
- Run commands from the repo root to ensure workspace settings apply.

## Testing instructions

- Prefer `cargo test` from workspace root for changes.
- Add parity tests against MSG sample outputs when modifying
  Define-XML/Dataset-XML/XPT.
- Keep outputs deterministic (timestamps, ordering, lengths).

## Large datasets and fixtures

- MSG sample package and XML/XPT fixtures are large; avoid printing whole files.
- Use targeted `rg`/`sed` slices and keep new fixtures minimal and
  deterministic.

## Commit and PR guidelines

- Keep changes focused and explain SDTMIG/MSG rationale in descriptions.
- Update `docs/SUGGESTED_CODE_CHANGES.md` when requirements or assumptions
  change.
- Do not rewrite history or amend unless explicitly requested.

## Deployment and release

- There is no production deploy flow; build releases with
  `cargo build --release`.
- CLI entry point lives in `crates/sdtm-cli`; validate outputs against MSG
  samples.

## Working practices

- Prefer `rg` for file and text search.
- Always check the task scope and status in `docs/SUGGESTED_CODE_CHANGES.md`
  before starting work.
- Mark completed tasks with `[x]` in `docs/SUGGESTED_CODE_CHANGES.md`.
- If requirements change, update `docs/SUGGESTED_CODE_CHANGES.md`.
- Before implementing any SDTM rule, read the relevant section in
  `standards/sdtmig/v3_4/chapters/` to verify the requirement.
- When in doubt about SDTM behavior, consult the chapter documentation first.
