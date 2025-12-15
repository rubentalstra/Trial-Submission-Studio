# Agent Instructions (CDISC Transpiler)

This repository is mid-refactor to a consistent **Ports & Adapters (Hexagonal /
Clean Architecture)** shape. When working in this repo, follow these rules so
the codebase converges instead of drifting.

## Quality Gates (run before you finish a chunk)

- `pyright`
- `ruff check .`
- `ruff format .`
- `pytest`
- If you touched transformers/generators/builders: `pytest -m validation` and
  `pytest -m benchmark --benchmark-only`

## Layer Boundaries (hard rules)

### `cdisc_transpiler/domain/` (pure business logic)

- May depend on: stdlib, `pandas`, `pydantic` (if needed), domain
  entities/services.
- Must NOT import: `cdisc_transpiler.cli`, `cdisc_transpiler.infrastructure`,
  Click/Rich, filesystem/network I/O, XML/XPT/SAS writers.
- Domain services return **data** (e.g., DataFrames + metadata), never write
  files.

### `cdisc_transpiler/application/` (use cases + ports + DTOs)

- Use cases orchestrate workflows using **ports** and domain services.
- Must NOT import: `cdisc_transpiler.infrastructure` or `cdisc_transpiler.cli`.
- Ports live in `cdisc_transpiler/application/ports/*` and must not reference
  infrastructure types.

### `cdisc_transpiler/infrastructure/` (adapters + wiring)

- Implements ports with concrete I/O: CSV/Excel/SAS reading, CT/spec loading,
  caching, XPT/XML/Define-XML/SAS generation.
- DI wiring belongs here (composition root).

### `cdisc_transpiler/cli/` (driver adapter)

- Click commands: parse args → build request DTO → call use case → present
  results.
- Presenters format output only; no orchestration/business logic.

## Compatibility Policy (until next major)

- `cdisc_transpiler/legacy/*` and the various “compatibility wrappers” exist
  only to keep external API stability.
- Do **not** add new internal call sites to legacy or wrapper modules; migrate
  call sites to the proper layer instead.
- If you remove a wrapper/legacy module, migrate call sites first, then delete,
  then run the full test suite.

## Naming Conventions

- Use cases: `*UseCase`
- Ports: `*Port`
- Adapters/implementations: `*Adapter` (or `*Repository`, `*Writer` when clearly
  infrastructure)
- Avoid vague names (`utils`, `helpers`, `manager`, `processor`) unless narrowly
  scoped and layer-specific.

## Refactor Protocol (small, safe, reversible)

- Prefer mechanical refactors (move/rename/extract) over logic changes.
- Keep public CLI flags/commands stable.
- Do not introduce new “lazy imports” to paper over cycles; fix the dependency
  direction instead.
- Do not re-export imports (no “barrel” exports via `__init__.py`); import from
  the defining module instead.

## Source of truth

- Architecture + current violations + migration plan: `docs/ARCHITECTURE.md`
