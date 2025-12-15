# CDISC Transpiler — Architecture, Boundaries, and Refactoring Guide

This repo is mid-migration to **Ports & Adapters (Hexagonal / Clean Architecture)**. This document is written for maintainers/agents and is the single source of truth for the **current layout**, **boundary violations**, and the **remaining migration steps**.

Before doing refactor work, also read `AGENTS.md` (repo-scoped rules and quality gates).

## 0) Agent Quick Start

### Non-negotiables

- Keep the CLI interface stable (`cdisc-transpiler study …`, `cdisc-transpiler domains`).
- Keep SDTM compliance and output formats stable (XPT, Dataset-XML, Define-XML 2.1, SAS).
- Prefer small, reversible refactors over “big bang” rewrites.
- Do not introduce new legacy wrappers or new internal call sites to existing wrappers.
- If you touch transformation/build/generation hot paths, run benchmarks and validation.

### Quality gates

This repo expects these to pass (see `AGENTS.md` for the full checklist):

- `pyright`
- `ruff check .`
- `ruff format .`
- `pytest`

## 1) Repo Inventory (Concrete)

### Entrypoints & composition roots

- **CLI entrypoint (packaging):** `pyproject.toml` → `cdisc-transpiler = "cdisc_transpiler.cli:app"`
- **CLI module entrypoint:** `cdisc_transpiler/cli/__main__.py` (supports `python -m cdisc_transpiler.cli`)
- **Click app:** `cdisc_transpiler/cli/__init__.py` exposes `app`
- **Composition root / DI container:** `cdisc_transpiler/infrastructure/container.py` (`DependencyContainer`, `create_default_container`)

### Layer responsibilities (as implemented today)

#### `cdisc_transpiler/cli/` (Driver adapter)
- `cdisc_transpiler/cli/commands/study.py`: Thin Click adapter → builds `ProcessStudyRequest`, calls use case via DI container, renders via presenter.
- `cdisc_transpiler/cli/commands/domains.py`: Lists supported domains (currently reads from `domains_module` directly).
- `cdisc_transpiler/cli/presenters/*`: Rich formatting only (tables/progress).
- `cdisc_transpiler/cli/helpers.py`: CLI-facing helpers (still contains output-writing helpers like split XPTs; should move out of CLI).
- Compatibility wrappers:
  - `cdisc_transpiler/cli/logging_config.py`: global logger compatibility layer.
  - `cdisc_transpiler/cli/utils.py`: `ProgressTracker` alias.

#### `cdisc_transpiler/application/` (Use cases + ports + DTOs)
- Use cases (currently at package root, not under `application/use_cases/`):
  - `cdisc_transpiler/application/study_processing_use_case.py`
  - `cdisc_transpiler/application/domain_processing_use_case.py`
- DTOs (requests/responses): `cdisc_transpiler/application/models.py`
- Ports (Protocols): `cdisc_transpiler/application/ports/`
  - `repositories.py`: `StudyDataRepositoryPort`, `CTRepositoryPort`, `SDTMSpecRepositoryPort`
  - `services.py`: `LoggerPort`, `FileGeneratorPort`, writer/generator ports

#### `cdisc_transpiler/domain/` (Entities + domain services)
- Entities: `cdisc_transpiler/domain/entities/` (e.g., `sdtm_domain.py`, `study_metadata.py`, `mapping.py`)
- Domain services: `cdisc_transpiler/domain/services/`
  - `domain_frame_builder.py`: builds SDTM domain DataFrames
  - `suppqual_service.py`: builds SUPPQUAL frames
  - `relrec_service.py`: RELREC synthesis logic
  - `synthesis_service.py`: domain synthesis (currently mixes orchestration + file generation concerns; see violations)

#### `cdisc_transpiler/infrastructure/` (Adapters + wiring)
- DI container: `cdisc_transpiler/infrastructure/container.py`
- I/O adapters: `cdisc_transpiler/infrastructure/io/`
  - reading: `csv_reader.py`
  - writing: `xpt_writer.py`, `dataset_xml_writer.py`, `sas_writer.py`
  - output orchestration: `file_generator.py`
  - Define-XML: `define_xml_generator.py`, plus `define_xml/*` builders
  - DTOs currently (but should be application-level): `infrastructure/io/models.py`
- Repositories (adapters over disk/metadata): `cdisc_transpiler/infrastructure/repositories/`
  - `study_data_repository.py`, `study_metadata_loader.py`
  - `ct_repository.py`, `sdtm_spec_repository.py`
- Logging: `cdisc_transpiler/infrastructure/logging/` (`ConsoleLogger`, `NullLogger`)

#### `cdisc_transpiler/metadata_module/` (Compatibility layer)
- Mostly re-exports:
  - metadata entities now live in `cdisc_transpiler/domain/entities/study_metadata.py`
  - loaders implemented in `cdisc_transpiler/infrastructure/repositories/study_metadata_loader.py`

### Other important packages (current state)

These modules exist outside the four “clean” layer folders and are a major source of confusion. They must either become true adapters/ports, move into the proper layer, or be reduced to thin compatibility shims.

- `cdisc_transpiler/domains_module/`: SDTM domain/variable registry loaded from spec CSVs. Currently re-exports domain entities for backwards compatibility.
- `cdisc_transpiler/terminology_module/`: controlled terminology registry + normalization helpers. Duplicates infrastructure repository responsibilities today.
- `cdisc_transpiler/transformations/`: transformation framework and domain-specific transformers (VS/LB wide-to-long).
- `cdisc_transpiler/mapping_module/`: fuzzy/metadata-aware mapping engine; contains compatibility wrappers (`config_io.py` delegates to infrastructure).
- `cdisc_transpiler/xml_module/`, `cdisc_transpiler/xpt_module/`, `cdisc_transpiler/sas_module/`: output-generation implementation modules (some are wrappers around newer domain/infrastructure code).
- `cdisc_transpiler/io_module/`, `cdisc_transpiler/submission_module/`: compatibility wrappers over the newer repository/domain-service implementations.
- `cdisc_transpiler/services/`: layer-ambiguous “service” bucket (mixes domain/application/infrastructure responsibilities).

### Public API surface (stability constraints)

Be careful with these, because downstream users may import them directly:

- `cdisc_transpiler/__init__.py` re-exports XML builders and domain metadata accessors (this is part of the public API).
- `cdisc_transpiler/domains_module/__init__.py` re-exports `SDTMDomain`/`SDTMVariable` from `cdisc_transpiler/domain/entities/sdtm_domain.py`.
- Wrapper modules under `cdisc_transpiler/*_module/` may be externally imported even if internally we want to migrate away from them.

### Legacy code candidates (safe-to-remove once call sites migrate)

- `cdisc_transpiler/legacy/`
  - `domain_processing_coordinator.py`
  - `domain_synthesis_coordinator.py`
  - `study_orchestration_service.py`
  - `cdisc_transpiler/legacy/__init__.py` (deprecation shim)
- `cdisc_transpiler/services/__init__.py` re-exports deprecated legacy classes (should stop doing this once downstream code migrates).

### Rewrapping / compatibility-layer candidates (thin pass-through)

These are not “bad” per se, but should be reduced/removed once internal call sites stop using them:

- `cdisc_transpiler/io_module/readers.py` (delegates to `infrastructure.repositories.study_data_repository`)
- `cdisc_transpiler/mapping_module/config_io.py` (delegates to `infrastructure.repositories.mapping_config_repository`)
- `cdisc_transpiler/metadata_module/loaders.py` (delegates to `infrastructure.repositories.study_metadata_loader`)
- `cdisc_transpiler/submission_module/suppqual.py` (delegates to `domain.services.suppqual_service`)
- `cdisc_transpiler/xpt_module/builder.py` (delegates to `domain.services.domain_frame_builder`)
- `cdisc_transpiler/cli/logging_config.py` (global logger; bypasses `LoggerPort`)

## 2) Clean/Hexagonal Diagnosis (Boundary Violations)

### Violations found (with concrete examples)

1) **Domain importing infrastructure**
- `cdisc_transpiler/domain/services/synthesis_service.py` imports `cdisc_transpiler.infrastructure.io.models` (and constructs output requests) → domain now knows about infrastructure DTOs and output formats.

2) **Application ports importing infrastructure types**
- `cdisc_transpiler/application/ports/services.py` type-check imports `OutputRequest`/`OutputResult` from `cdisc_transpiler.infrastructure.io.models` → port boundary “leaks” infrastructure details upward.

3) **Use cases importing infrastructure DTOs**
- `cdisc_transpiler/application/domain_processing_use_case.py` and `cdisc_transpiler/application/study_processing_use_case.py` import `OutputRequest`/`OutputDirs` from `cdisc_transpiler.infrastructure.io.models` → application depends on infrastructure.

4) ~~**Use cases importing concrete XML models**~~ ✅ RESOLVED
- ~~`cdisc_transpiler/application/study_processing_use_case.py` imports `StudyDataset` from `cdisc_transpiler/xml_module.define_module` to drive Define-XML generation → application is coupled to a concrete Define-XML representation.~~
- **Resolution:** Created `DefineDatasetDTO` in application layer. The adapter converts DTOs to infrastructure `StudyDataset`.

5) **“Service” package is layer-ambiguous**
- `cdisc_transpiler/services/*` contains a mix of:
  - domain-ish logic (trial design synthesis),
  - application-ish orchestration helpers,
  - infrastructure-ish filesystem side effects (output dir creation, PDF stub generation).

6) **Duplicate file-writing logic lives in multiple layers**
- Domain split XPT writing exists both in `cdisc_transpiler/cli/helpers.py` and in `cdisc_transpiler/application/domain_processing_use_case.py` (`_write_variant_splits`) and bypasses the injected writer adapters.

7) **Domain depends on non-domain package (`domains_module`)**
- `cdisc_transpiler/domain/entities/mapping.py` and `cdisc_transpiler/domain/entities/variable.py` import from `cdisc_transpiler/domains_module/*` to validate domains and derive General Observation Class behavior. This creates import-direction ambiguity and contributes to circular dependency pressure.

8) **Terminology responsibilities duplicated**
- `cdisc_transpiler/terminology_module/*` loads CT and performs normalization, while `cdisc_transpiler/infrastructure/repositories/ct_repository.py` also loads CT with caching. Choose one “real implementation” and make the other a shim.

### Why these are harmful here

- **Testability:** Ports should be mockable without importing infrastructure modules; otherwise unit tests pull in concrete I/O and large dependency graphs.
- **Maintainability:** Boundaries become unclear, leading to circular imports and “lazy import” workarounds that hide design debt.
- **Performance risk:** Extra wrapper hops and duplicated pipelines are easy to accidentally reintroduce; clear boundaries keep hot paths explicit and benchmarkable.
- **Refactor friction:** If application depends on infrastructure types, moving/adapting infrastructure becomes a wide blast-radius change.

## 3) Target Architecture (Final Shape for THIS Repo)

Keep the current top-level layout, but enforce strict boundaries:

### Domain (pure)
- Location: `cdisc_transpiler/domain/`
- Contains:
  - entities: `Study`, `Domain`, `Variable` (and current SDTM entities)
  - domain services: `DomainFrameBuilder`, `SuppQualService`, `RelRecService`, `SynthesisService` (pure synthesis only)
- Must NOT:
  - touch filesystem, Click, Rich, XML/XPT/SAS writers, pandas I/O
  - import from `cdisc_transpiler.infrastructure` or `cdisc_transpiler.cli`

### Application (use cases + ports + DTOs)
- Location: `cdisc_transpiler/application/`
- Contains:
  - use cases (move into `application/use_cases/` over time): `StudyProcessingUseCase`, `DomainProcessingUseCase`
  - DTOs: `ProcessStudyRequest/Response`, `ProcessDomainRequest/Response`, plus output-generation DTOs (move out of infrastructure)
  - ports for:
    - study input reading (`StudyDataRepositoryPort`)
    - transformation pipeline (if we want to swap pipelines)
    - output generation (XPT/XML/Define-XML/SAS)
    - metadata access (CT/spec repositories)
    - progress/logging (`LoggerPort`)
- Must NOT:
  - import from `cdisc_transpiler.infrastructure` or `cdisc_transpiler.cli`

### Infrastructure (adapters)
- Location: `cdisc_transpiler/infrastructure/`
- Contains concrete implementations:
  - repositories: CSV/Excel/SAS reading, metadata loading, CT/spec loading, caching
  - writers/generators: XPT, Dataset-XML, Define-XML, SAS
  - DI container wiring only

### CLI (driver adapter)
- Location: `cdisc_transpiler/cli/`
- Click commands remain thin: args → request DTO → use case → presenter output.

### Allowed dependency direction (enforced mental model)

- `cli` → `application` → `domain`
- `infrastructure` → (`application` + `domain`)
- `domain` → (no internal dependencies outside domain; external libs allowed but no I/O)

If a module “doesn’t fit” those arrows, it either moves or becomes an adapter.

### Boundary smoke checks (fast greps)

Use these when you’re about to finish a refactor chunk:

- Application must not import infrastructure:
  - `rg -n "cdisc_transpiler\\.infrastructure|from \\.\\..*infrastructure" -S cdisc_transpiler/application`
- Domain must not import infrastructure or CLI:
  - `rg -n "cdisc_transpiler\\.infrastructure|from \\.\\..*infrastructure" -S cdisc_transpiler/domain`
  - `rg -n "cdisc_transpiler\\.cli|from \\.\\..*cli" -S cdisc_transpiler/domain`
- No new internal usage of legacy coordinators:
  - `rg -n "cdisc_transpiler\\.legacy|from \\.\\..*legacy" -S cdisc_transpiler`

## 4) Migration Rules (Definitions)

### Legacy code
Code is considered **legacy** if it:
- bypasses application use cases/ports and directly orchestrates end-to-end workflows, OR
- duplicates the pipeline in a parallel “old path”, OR
- lives under `cdisc_transpiler/legacy/`.

### Rewrapping
Code is considered **rewrapping** if it:
- forwards calls 1:1 to another module without adding domain value, OR
- exists only to avoid moving imports while the real implementation lives elsewhere.

Examples today: `cdisc_transpiler/io_module/readers.py`, `cdisc_transpiler/xpt_module/builder.py`.

### Allowed wrappers (only if they add real value)
Wrappers are acceptable when they add **cross-cutting concerns** and live in the correct layer, e.g.:
- caching (`infrastructure/caching/*`)
- metrics/timing/logging (`infrastructure/logging/*`)
- validation/guardrails at boundaries (CLI input validation, application request validation)
- retry/backoff for external systems (if/when added)

## 5) Migration Map (Current → Target Home)

Use this as the default “where should I move this?” reference.

- Output request/response DTOs: `cdisc_transpiler/infrastructure/io/models.py` → application DTOs (new `cdisc_transpiler/application/*`)
- Define-XML dataset model used by the use case: `cdisc_transpiler/xml_module/define_module/models.py:StudyDataset` → application DTO (infra converts)
- Output directory creation + ACRF PDF placeholder: `cdisc_transpiler/services/file_organization_service.py` → infrastructure adapter
- Domain split XPT writer: `cdisc_transpiler/cli/helpers.py` + `cdisc_transpiler/application/domain_processing_use_case.py` → infrastructure adapter (single implementation)
- Controlled terminology loading: `cdisc_transpiler/terminology_module/*` ↔ `cdisc_transpiler/infrastructure/repositories/ct_repository.py` → one real implementation + one shim
- SDTM domain metadata registry: `cdisc_transpiler/domains_module/*` → either a domain-owned registry or an infrastructure-backed repository (but with clean import direction)

## 6) Naming & Structure Improvement Plan (Required)

### Conventions to enforce
- Use cases end with `UseCase`
- Port interfaces end with `Port` (e.g., `StudyReaderPort`, `OutputGeneratorPort`)
- Adapter implementations end with `Adapter` (e.g., `CSVStudyReaderAdapter`, `XPTWriterAdapter`)
- Avoid vague names (`utils`, `helpers`, `manager`, `processor`) unless narrowly scoped and layer-specific
- Filenames match the primary class/function inside

### Concrete cleanup targets (proposed)

Low-risk, high-signal naming changes (with compatibility aliases where needed):
- `cdisc_transpiler/infrastructure/container.py` → keep file name, but consider exporting an alias `DependencyContainer` in a small `dependency_container.py` shim if external docs/tools assume that name.
- `cdisc_transpiler/infrastructure/io/file_generator.py:FileGenerator` → consider `OutputGenerationAdapter` or `FileGeneratorAdapter` (keep `FileGenerator` alias temporarily).
- `cdisc_transpiler/infrastructure/repositories/study_data_repository.py:StudyDataRepository` → consider `StudyDataRepositoryAdapter` (keep old class name alias temporarily).
- `cdisc_transpiler/services/domain_discovery_service.py:DomainDiscoveryService` → rename to `DomainFileDiscoveryService` and move to application/domain (depending on final responsibility).
- `cdisc_transpiler/services/file_organization_service.py` → move to infrastructure and rename to `OutputLayoutAdapter` / `OutputDirectoryService`.
- `cdisc_transpiler/services/trial_design_service.py` → deprecate in favor of a single synthesis service (avoid duplication with `domain/services/synthesis_service.py`).

## 7) Refactor Plan (Small, Safe Steps)

Each step is intended to be PR-sized and reversible.

### Step 1 — Move output-generation DTOs to application (Risk: Medium)
- **Goal:** Remove application/port dependencies on `infrastructure.io.models`.
- **Affects:** `cdisc_transpiler/infrastructure/io/models.py`, `cdisc_transpiler/application/ports/services.py`, both use cases.
- **Mechanics:** Create application-level DTOs (e.g., `application/models_output.py` or extend `application/models.py`) and have infra adapters consume those.
- **Verify:**
  - `pyright`
  - `ruff check .`
  - `pytest`

### Step 2 — Make domain synthesis pure (Risk: Medium/High)
- **Goal:** `domain/services/synthesis_service.py` returns only domain data (DataFrames + metadata), no output writing, no infra DTOs.
- **Affects:** `cdisc_transpiler/domain/services/synthesis_service.py`, `cdisc_transpiler/application/study_processing_use_case.py`
- **Verify:**
  - `pyright`
  - `ruff check .`
  - `pytest -m validation`
  - `pytest -m benchmark --benchmark-only`

### Step 3 — Define-XML boundary cleanup (Risk: Medium) ✅ DONE
- **Goal:** Application produces a Define-XML-neutral DTO; infrastructure adapter turns it into `StudyDataset` / XML.
- **Affects:** `cdisc_transpiler/application/study_processing_use_case.py`, `cdisc_transpiler/infrastructure/io/define_xml_generator.py`
- **Status:** Completed
- **Changes made:**
  - Created `DefineDatasetDTO` in `cdisc_transpiler/application/models.py` as application-layer DTO
  - Updated `DefineXmlGeneratorPort` in `cdisc_transpiler/application/ports/services.py` to accept `DefineDatasetDTO`
  - Updated `StudyProcessingUseCase` to use `DefineDatasetDTO` instead of infrastructure `StudyDataset`
  - Updated `DefineXmlGenerator` adapter in infrastructure to convert from `DefineDatasetDTO` to `StudyDataset`
  - Removed imports of `xml_module.define_module.StudyDataset` and constants from application layer
- **Verify:** `pytest -m validation`

### Step 4 — Remove duplication: trial design synthesis (Risk: Medium)
- **Goal:** Have exactly one synthesis implementation (domain service), with application orchestrating and infra writing.
- **Affects:** `cdisc_transpiler/services/trial_design_service.py`, `cdisc_transpiler/domain/services/synthesis_service.py`, legacy coordinators.
- **Verify:** `pytest`, `pytest -m validation`

### Step 5 — Reduce compatibility shims (Risk: Low/Medium)
- **Goal:** Stop importing wrappers from internal code; keep shims only for external API stability until next major version.
- **Affects:** `cdisc_transpiler/io_module/*`, `cdisc_transpiler/xpt_module/*`, `cdisc_transpiler/submission_module/*`, `cdisc_transpiler/metadata_module/*`
- **Verify:** `pyright`, `pytest`

### Performance note
When changing transformation/build/generation hot paths, run:
- `pytest -m benchmark --benchmark-only`

And for end-to-end correctness:
- `pytest -m validation`

## 8) “Definition of Done” for this migration

The repo is considered “clean architecture consistent” when:

- CLI commands call only use cases and presenters (no orchestration logic in Click modules).
- Use cases depend only on domain + ports + DTOs (no infrastructure imports, no concrete XML/XPT/SAS types).
- Ports do not reference infrastructure types.
- Infrastructure holds all concrete I/O and all external library glue (XPT/XML/SAS, filesystem, `pyreadstat`, etc.).
- Legacy coordinators are removed, and wrapper modules are either removed or no longer used internally.
- `pyright && ruff check . && ruff format . && pytest` pass.
- Validation suite passes: `pytest -m validation`
- No significant benchmark regression: `pytest -m benchmark --benchmark-only`
