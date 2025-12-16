# CDISC Transpiler — Architecture, Boundaries, and Refactoring Guide

This repo is mid-migration to **Ports & Adapters (Hexagonal / Clean
Architecture)**. This document is written for maintainers/agents and is the
single source of truth for the **current layout**, **boundary violations**, and
the **remaining migration steps**.

Before doing refactor work, also read `AGENTS.md` (repo-scoped rules and quality
gates).

## 0) Agent Quick Start

### Non-negotiables

- Keep the CLI interface stable (`cdisc-transpiler study …`,
  `cdisc-transpiler domains`).
- Keep SDTM compliance and output formats stable (XPT, Dataset-XML, Define-XML
  2.1, SAS).
- Prefer small, reversible refactors over “big bang” rewrites.
- Do not introduce new legacy wrappers or new internal call sites to existing
  wrappers.
- If you touch transformation/build/generation hot paths, run benchmarks and
  validation.

### Quality gates

This repo expects these to pass (see `AGENTS.md` for the full checklist):

- `pyright`
- `ruff check .`
- `ruff format .`
- `pytest`

### Current baseline (informational)

As of a repo-wide check on 2025-12-16:

- `pyright` currently reports type errors across application + domain +
  infrastructure.
- `ruff check .` currently reports lint issues across library + tests.
- `pytest` currently has integration + validation failures because the study CLI
  workflow exits non-zero (trial-design SE synthesis fails with missing
  `USUBJID`), which cascades into downstream validation assertions.

This section is not a release gate by itself, but it explains why “verify” steps
below may fail until the hot-path errors are addressed.

## 1) Repo Inventory (Concrete)

### Entrypoints & composition roots

- **CLI entrypoint (packaging):** `pyproject.toml` →
  `cdisc-transpiler = "cdisc_transpiler.cli:app"`
- **CLI module entrypoint:** `cdisc_transpiler/cli/__main__.py` (supports
  `python -m cdisc_transpiler.cli`)
- **Click app:** `cdisc_transpiler/cli/__init__.py` exposes `app`
- **Composition root / DI container:**
  `cdisc_transpiler/infrastructure/container.py` (`DependencyContainer`,
  `create_default_container`)

### Layer responsibilities (as implemented today)

#### `cdisc_transpiler/cli/` (Driver adapter)

- `cdisc_transpiler/cli/commands/study.py`: Thin Click adapter → builds
  `ProcessStudyRequest`, calls use case via DI container, renders via presenter.
- `cdisc_transpiler/cli/commands/domains.py`: Lists supported domains via the
  SDTM spec registry.
- `cdisc_transpiler/cli/presenters/*`: Rich formatting only (tables/progress).
- CLI should contain only drivers/presentation concerns (argument parsing,
  calling use cases, and formatting output).

#### `cdisc_transpiler/application/` (Use cases + ports + DTOs)

- Use cases (currently at package root, not under `application/use_cases/`):
  - `cdisc_transpiler/application/study_processing_use_case.py`
  - `cdisc_transpiler/application/domain_processing_use_case.py`
- DTOs (requests/responses): `cdisc_transpiler/application/models.py`
- Ports (Protocols): `cdisc_transpiler/application/ports/`
  - `repositories.py`: `StudyDataRepositoryPort`, `CTRepositoryPort`,
    `SDTMSpecRepositoryPort`
  - `services.py`: `LoggerPort`, `FileGeneratorPort`, writer/generator ports

#### `cdisc_transpiler/domain/` (Entities + domain services)

- Entities: `cdisc_transpiler/domain/entities/` (e.g., `sdtm_domain.py`,
  `study_metadata.py`, `mapping.py`)
- Domain services: `cdisc_transpiler/domain/services/`
  - `domain_frame_builder.py`: builds SDTM domain DataFrames
  - `suppqual_service.py`: builds SUPPQUAL frames
  - `relrec_service.py`: RELREC synthesis logic
  - `synthesis_service.py`: domain synthesis (currently mixes orchestration +
    file generation concerns; see violations)

#### `cdisc_transpiler/infrastructure/` (Adapters + wiring)

- DI container: `cdisc_transpiler/infrastructure/container.py`
- I/O adapters: `cdisc_transpiler/infrastructure/io/`
  - reading: `csv_reader.py`
  - writing: `xpt_writer.py`, `dataset_xml_writer.py`, `sas_writer.py`
  - output orchestration: `file_generator.py`
  - Define-XML: `define_xml_generator.py`, plus `define_xml/*` builders
  - Output DTOs live in the application layer
    (`cdisc_transpiler/application/models.py`:
    `OutputRequest`/`OutputResult`/`OutputDirs`).
- Repositories (adapters over disk/metadata):
  `cdisc_transpiler/infrastructure/repositories/`
  - `study_data_repository.py`, `study_metadata_loader.py`
  - `ct_repository.py`, `sdtm_spec_repository.py`
- Logging: `cdisc_transpiler/infrastructure/logging/` (`ConsoleLogger`,
  `NullLogger`)

#### Removed compatibility wrappers

The following wrapper packages have been removed after migrating all internal
call sites to the clean layers:

- `cdisc_transpiler/io_module/`
- `cdisc_transpiler/mapping_module/`
- `cdisc_transpiler/metadata_module/`
- `cdisc_transpiler/submission_module/`
- `cdisc_transpiler/terminology_module/`

### Other important packages (current state)

These modules exist outside the four “clean” layer folders and are a major
source of confusion. They must either become true adapters/ports, move into the
proper layer, or be removed.

- `cdisc_transpiler/infrastructure/sdtm_spec/`: SDTM domain/variable registry
  loaded from spec CSVs (current implementation).
- `cdisc_transpiler/transformations/`: transformation framework and
  domain-specific transformers (VS/LB wide-to-long).
- Output generation implementations live under
  `cdisc_transpiler/infrastructure/io/` (XPT, Dataset-XML, Define-XML, SAS).
- `cdisc_transpiler/services/`: layer-ambiguous “service” bucket (mixes
  domain/application/infrastructure responsibilities).

### Public API surface (stability constraints)

Be careful with these, because downstream users may import them directly:

- `cdisc_transpiler/__init__.py` re-exports XML builders and domain metadata
  accessors (this is part of the public API).
- Wrapper modules under `cdisc_transpiler/*_module/` have been removed after
  migrating internal call sites; do not introduce new compatibility shims.

### Legacy code candidates (safe-to-remove once call sites migrate)

- `cdisc_transpiler/legacy/`
  - `domain_processing_coordinator.py`
  - `domain_synthesis_coordinator.py`
  - `study_orchestration_service.py`
  - `cdisc_transpiler/legacy/__init__.py` (deprecation shim)
- `cdisc_transpiler/services/__init__.py` re-exports deprecated legacy classes
  (should stop doing this once downstream code migrates).

### Rewrapping / compatibility-layer candidates (thin pass-through)

The wrapper packages listed above have been removed. Remaining candidates for
cleanup are layer-crossing helpers and re-export modules that bypass ports.

## 2) Clean/Hexagonal Diagnosis (Boundary Violations)

### Violations found (with concrete examples)

1. ~~**Domain importing infrastructure**~~ ✅ RESOLVED

- The domain layer no longer imports infrastructure I/O DTOs.

2. ~~**Application ports importing infrastructure types**~~ ✅ RESOLVED

- Application ports no longer reference infrastructure DTOs.

3. ~~**Use cases importing infrastructure DTOs**~~ ✅ RESOLVED

- Output-generation DTOs are application-layer types now.

4. ~~**Use cases importing concrete XML models**~~ ✅ RESOLVED

- ~~`cdisc_transpiler/application/study_processing_use_case.py` imports a
  concrete Define-XML dataset model to drive generation → application is coupled
  to an infrastructure representation.~~
- **Resolution:** Created `DefineDatasetDTO` in application layer. The adapter
  converts DTOs to infrastructure `StudyDataset`.

5. **“Service” package is layer-ambiguous**

- `cdisc_transpiler/services/*` contains a mix of:
  - domain-ish logic (trial design synthesis),
  - application-ish orchestration helpers,
  - infrastructure-ish filesystem side effects (output dir creation, PDF stub
    generation).

6. **Duplicate file-writing logic lives in multiple layers**

- Split XPT writing should live in the application layer (use case
  orchestration) and route through injected ports/adapters.

7. **(Resolved) Domain depended on `domains_module`**

- Domain entities/services no longer import the compatibility shim. General
  Observation Class helpers live in the domain layer, and spec loading lives in
  `cdisc_transpiler/infrastructure/sdtm_spec/`.

8. **(Resolved) Terminology responsibilities duplicated**

- Controlled terminology is loaded via `CTRepository` (infrastructure), and the
  application accesses terminology via injected ports/adapters.

9. **(Resolved) Domain imported compatibility/output modules (`*_module`)**

- Domain processors previously imported transformers from an output-focused
  compatibility layer. These transformers now live in
  `cdisc_transpiler/domain/services/transformers/`.
- Domain mapping/synthesis uses domain entities/services directly, and receives
  infrastructure concerns (spec/CT) via injected ports/adapters.

10. **(Resolved) Application layer depended on compatibility modules**

- Use cases and ports have been migrated to depend on ports and domain entities,
  with infrastructure adapters supplied by the DI container.

11. **Legacy import side effects leak via `services/__init__.py`**

- `cdisc_transpiler/services/__init__.py` imports from
  `cdisc_transpiler/legacy`, which triggers deprecation warnings (and couples
  unrelated imports to legacy code paths).

12. **Duplicated Dataset-XML implementation exists in two places**

- Dataset-XML glue historically existed in two places. The concrete
  implementation now lives under
  `cdisc_transpiler/infrastructure/io/dataset_xml/*`.

### Why these are harmful here

- **Testability:** Ports should be mockable without importing infrastructure
  modules; otherwise unit tests pull in concrete I/O and large dependency
  graphs.
- **Maintainability:** Boundaries become unclear, leading to circular imports
  and “lazy import” workarounds that hide design debt.
- **Performance risk:** Extra wrapper hops and duplicated pipelines are easy to
  accidentally reintroduce; clear boundaries keep hot paths explicit and
  benchmarkable.
- **Refactor friction:** If application depends on infrastructure types,
  moving/adapting infrastructure becomes a wide blast-radius change.

## 3) Target Architecture (Final Shape for THIS Repo)

Keep the current top-level layout, but enforce strict boundaries:

### Domain (pure)

- Location: `cdisc_transpiler/domain/`
- Contains:
  - entities: `Study`, `Domain`, `Variable` (and current SDTM entities)
  - domain services: `DomainFrameBuilder`, `SuppQualService`, `RelRecService`,
    `SynthesisService` (pure synthesis only)
- Must NOT:
  - touch filesystem, Click, Rich, XML/XPT/SAS writers, pandas I/O
  - import from `cdisc_transpiler.infrastructure` or `cdisc_transpiler.cli`

### Application (use cases + ports + DTOs)

- Location: `cdisc_transpiler/application/`
- Contains:
  - use cases (move into `application/use_cases/` over time):
    `StudyProcessingUseCase`, `DomainProcessingUseCase`
  - DTOs: `ProcessStudyRequest/Response`, `ProcessDomainRequest/Response`, plus
    output-generation DTOs (move out of infrastructure)
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
  - repositories: CSV/Excel/SAS reading, metadata loading, CT/spec loading,
    caching
  - writers/generators: XPT, Dataset-XML, Define-XML, SAS
  - DI container wiring only

### CLI (driver adapter)

- Location: `cdisc_transpiler/cli/`
- Click commands remain thin: args → request DTO → use case → presenter output.

### Allowed dependency direction (enforced mental model)

- `cli` → `application` → `domain`
- `infrastructure` → (`application` + `domain`)
- `domain` → (no internal dependencies outside domain; external libs allowed but
  no I/O)

Compatibility wrapper _packages_ (e.g. `io_module`, `terminology_module`,
`submission_module`) are not part of the clean architecture.

These wrapper packages have been removed in this repository after migrating
internal call sites to `domain`/`application`/`infrastructure`. Avoid adding
compatibility shims; migrate call sites and remove shims when possible.

### Policy clarifications (make the boundaries real)

- **Composition root only:** DI wiring and adapter selection happens only in
  `cdisc_transpiler/infrastructure/container.py`.
- **Driver owns wiring:** CLI builds request DTOs, calls use cases, and presents
  results. CLI should not construct repositories/writers/generators directly.
- **No side-effectful re-exports:** Avoid importing `legacy/*` (or triggering
  deprecation warnings) from non-legacy modules like
  `cdisc_transpiler/services/__init__.py`.
- **Compatibility shims are discouraged:** Prefer migrating call sites to the
  canonical API and removing shims. If a shim exists for public API stability,
  keep it as a thin re-export only (no duplicated implementations, no
  orchestration).
- **Ports reference stable types:** Application ports should reference
  application/domain DTOs/entities, not `*_module` types.

If a module “doesn’t fit” those arrows, it either moves or becomes an adapter.

### Boundary smoke checks (fast greps)

Use these when you’re about to finish a refactor chunk:

- `rg -n "cdisc_transpiler\\.infrastructure|from \\.\\..*infrastructure" -S cdisc_transpiler/application`
- `rg -n "cdisc_transpiler\\.infrastructure|from \\.\\..*infrastructure" -S cdisc_transpiler/domain`
- `rg -n "cdisc_transpiler\\.cli|from \\.\\..*cli" -S cdisc_transpiler/domain`
- `rg -n "cdisc_transpiler\\.legacy|from \\.\\..*legacy" -S cdisc_transpiler`

- Domain/application must not import removed wrapper modules (internal call
  sites):
  - `rg -n "\\b(io_module|terminology_module|domains_module|metadata_module|mapping_module|submission_module)\\b" -S cdisc_transpiler/domain cdisc_transpiler/application`

- “Services” package must not pull in legacy as a side effect:
  - `rg -n "from \\.\\.legacy" -S cdisc_transpiler/services`

## 4) Migration Rules (Definitions)

### Legacy code

Code is considered **legacy** if it:

- bypasses application use cases/ports and directly orchestrates end-to-end
  workflows, OR
- duplicates the pipeline in a parallel “old path”, OR
- lives under `cdisc_transpiler/legacy/`.

### Rewrapping

Code is considered **rewrapping** if it:

- forwards calls 1:1 to another module without adding domain value, OR
- exists only to avoid moving imports while the real implementation lives
  elsewhere.

Examples (historical): `cdisc_transpiler/io_module/readers.py`,
`cdisc_transpiler/mapping_module/config_io.py`.

### Allowed wrappers (only if they add real value)

Wrappers are acceptable when they add **cross-cutting concerns** and live in the
correct layer, e.g.:

- caching (`infrastructure/caching/*`)
- metrics/timing/logging (`infrastructure/logging/*`)
- validation/guardrails at boundaries (CLI input validation, application request
  validation)
- retry/backoff for external systems (if/when added)

## 5) Migration Map (Current → Target Home)

Use this as the default “where should I move this?” reference.

- Output request/response DTOs: ✅ now live in
  `cdisc_transpiler/application/models.py`
- Define-XML dataset model used by the use case: infrastructure `StudyDataset` →
  application DTO (infra converts)
- Output directory creation + ACRF PDF placeholder:
  `cdisc_transpiler/services/file_organization_service.py` → infrastructure
  adapter
- Split XPT writing:
  `cdisc_transpiler/application/domain_processing_use_case.py`
  (`_write_variant_splits`) → consider unifying behind an infrastructure adapter
  / generator path (single implementation)
- Controlled terminology loading:
  `cdisc_transpiler/infrastructure/repositories/ct_repository.py` (current
  implementation)
- SDTM domain metadata registry: `cdisc_transpiler/infrastructure/sdtm_spec/*`
  (current implementation)

Additional high-impact migrations (current reality):

- Domain transformers: `cdisc_transpiler/domain/services/transformers/*`
- Mapping engine: `cdisc_transpiler/domain/services/mapping/*` (now pure
  domain).
- Dataset-XML: `cdisc_transpiler/infrastructure/io/dataset_xml/*`

## 6) Naming & Structure Improvement Plan (Required)

### Conventions to enforce

- Use cases end with `UseCase`
- Port interfaces end with `Port` (e.g., `StudyReaderPort`,
  `OutputGeneratorPort`)
- Adapter implementations end with `Adapter` (e.g., `CSVStudyReaderAdapter`,
  `XPTWriterAdapter`)
- Avoid vague names (`utils`, `helpers`, `manager`, `processor`) unless narrowly
  scoped and layer-specific
- Filenames match the primary class/function inside

### Concrete cleanup targets (proposed)

Low-risk, high-signal naming changes (with compatibility aliases where needed):

- `cdisc_transpiler/infrastructure/container.py` → keep file name, but consider
  exporting an alias `DependencyContainer` in a small `dependency_container.py`
  shim if external docs/tools assume that name.
- `cdisc_transpiler/infrastructure/io/file_generator.py:FileGenerator` →
  consider `OutputGenerationAdapter` or `FileGeneratorAdapter` (keep
  `FileGenerator` alias temporarily).
- `cdisc_transpiler/infrastructure/repositories/study_data_repository.py:StudyDataRepository`
  → consider `StudyDataRepositoryAdapter` (keep old class name alias
  temporarily).
- `cdisc_transpiler/services/domain_discovery_service.py:DomainDiscoveryService`
  → rename to `DomainFileDiscoveryService` and move to application/domain
  (depending on final responsibility).
- `cdisc_transpiler/services/file_organization_service.py` → move to
  infrastructure and rename to `OutputLayoutAdapter` / `OutputDirectoryService`.
- `cdisc_transpiler/services/trial_design_service.py` → deprecate in favor of a
  single synthesis service (avoid duplication with
  `domain/services/synthesis_service.py`).

## 7) Refactor Plan (Small, Safe Steps)

Each step is intended to be PR-sized and reversible.

### Step 1 — Move output-generation DTOs to application (Risk: Medium)

- **Goal:** Remove application/port dependencies on infrastructure I/O DTOs.
- **Status:** ✅ Completed (DTOs are in
  `cdisc_transpiler/application/models.py`).
- **Verify:** `pyright`, `pytest`

### Step 2 — Define-XML boundary cleanup (Risk: Medium) ✅ DONE

- **Goal:** Application produces a Define-XML-neutral DTO; infrastructure
  adapter turns it into `StudyDataset` / XML.
- **Affects:** `cdisc_transpiler/application/study_processing_use_case.py`,
  `cdisc_transpiler/infrastructure/io/define_xml_generator.py`
- **Status:** Completed
- **Changes made:**
  - Created `DefineDatasetDTO` in `cdisc_transpiler/application/models.py` as
    application-layer DTO
  - Updated `DefineXmlGeneratorPort` in
    `cdisc_transpiler/application/ports/services.py` to accept
    `DefineDatasetDTO`
  - Updated `StudyProcessingUseCase` to use `DefineDatasetDTO` instead of
    infrastructure `StudyDataset`
  - Updated `DefineXmlGenerator` adapter in infrastructure to convert from
    `DefineDatasetDTO` to `StudyDataset`
  - Removed imports of the concrete `StudyDataset` model and constants from the
    application layer
- **Verify:** `pytest -m validation`

### Step 3 — Remove legacy import side effects from `services/__init__.py` (Risk: Low)

- **Goal:** Importing `cdisc_transpiler.services` must not pull
  `cdisc_transpiler.legacy` (and emit deprecation warnings) unless a caller
  explicitly imports legacy.
- **Affects:** `cdisc_transpiler/services/__init__.py`
- **Verify:** `pytest -q tests/unit/architecture/test_import_boundaries.py`

### Step 4 — Add an application port for output layout (dirs + ACRF placeholder) (Risk: Medium)

- **Goal:** Output directory creation and placeholder files are infrastructure
  concerns.
- **Affects:** `cdisc_transpiler/services/file_organization_service.py`,
  `cdisc_transpiler/application/study_processing_use_case.py`, new port under
  `cdisc_transpiler/application/ports/`
- **Verify:**
  `pytest -q tests/integration/test_cli.py::TestStudyCommand::test_study_with_default_options`

### Step 5 — Remove `io_module` fallbacks from use cases (Risk: Medium)

- **Goal:** Application must depend on ports, not “if not injected, import
  wrapper”.
- **Affects:** `cdisc_transpiler/application/domain_processing_use_case.py`,
  `cdisc_transpiler/application/study_processing_use_case.py`
- **Verify:** `pytest -q tests/unit/application`

✅ Completed (wrappers removed; use cases depend on ports/adapters).

### Step 6 — Unify XPT split writing behind infrastructure adapter (Risk: Medium)

- **Goal:** There is exactly one implementation for split writing, and it is
  injected via ports.
- **Affects:** `cdisc_transpiler/application/domain_processing_use_case.py`,
  `cdisc_transpiler/infrastructure/io/file_generator.py`
- **Verify:**
  `pytest -q tests/integration/test_cli.py::TestStudyCommandWithGDISC::test_study_with_split_datasets`

### Step 7 — Make domain synthesis pure (Risk: Medium/High)

- **Goal:** `domain/services/synthesis_service.py` returns only domain data; it
  must not import from removed wrapper packages.
- **Affects:** `cdisc_transpiler/domain/services/synthesis_service.py`,
  `cdisc_transpiler/application/study_processing_use_case.py`
- **Verify:** `pytest -m validation`, `pytest -m benchmark --benchmark-only`

### Step 8 — Move domain processors off output-focused transformers (Risk: Medium)

- **Goal:** Domain processors must not import output-focused transformer
  modules.
- **Affects:** `cdisc_transpiler/domain/services/domain_processors/*`
- **Mechanics:** move/duplicate transformers into a domain-owned module.
- **Verify:** `pytest -q tests/unit/domain`

### Step 9 — Controlled terminology access via ports (Risk: Medium)

- **Goal:** Domain/application should not call controlled terminology via
  wrapper packages; use a repository/port.
- **Affects:** `cdisc_transpiler/domain/services/domain_processors/*`,
  `cdisc_transpiler/application/ports/repositories.py`,
  `cdisc_transpiler/infrastructure/repositories/ct_repository.py`
- **Verify:**
  `pytest -q tests/unit/infrastructure/repositories/test_ct_repository.py`

✅ Completed (CT via `CTRepository` + injected adapters).

### Step 10 — Remove `domains_module` shim (Risk: Medium)

- **Goal:** Remove the `cdisc_transpiler.domains_module` compatibility import
  path and migrate call sites to the SDTM spec registry.
- **Affects:** `cdisc_transpiler/domain/entities/mapping.py`,
  `cdisc_transpiler/domain/entities/variable.py`,
  `cdisc_transpiler/application/study_processing_use_case.py`
- **Verify:** boundary grep:
  `rg -n "\\bdomains_module\\b" -S cdisc_transpiler/domain cdisc_transpiler/application`

✅ Completed (shim removed; use `cdisc_transpiler.infrastructure.sdtm_spec`).

### Step 11 — De-duplicate Dataset-XML implementation (Risk: Medium)

- **Goal:** Choose one concrete implementation (recommend infrastructure) and
  make the other a shim.
- **Affects:** `cdisc_transpiler/infrastructure/io/dataset_xml/*`
- **Verify:** `pytest -m validation`

### Step 12 — Reduce compatibility shims (Risk: Low/Medium)

- **Goal:** Stop importing wrappers from internal code; keep shims only for
  external API stability until next major version.
- **Affects (historical):** removed wrapper packages like
  `cdisc_transpiler/io_module/*`, `cdisc_transpiler/submission_module/*`,
  `cdisc_transpiler/metadata_module/*`
- **Verify:** `pyright`, `pytest`

✅ Completed (wrapper folders removed; compatibility maintained via explicit
shims).

### Performance note

When changing transformation/build/generation hot paths, run:

- `pytest -m benchmark --benchmark-only`

And for end-to-end correctness:

- `pytest -m validation`

## 8) “Definition of Done” for this migration

The repo is considered “clean architecture consistent” when:

- CLI commands call only use cases and presenters (no orchestration logic in
  Click modules).
- Use cases depend only on domain + ports + DTOs (no infrastructure imports, no
  concrete XML/XPT/SAS types).
- Ports do not reference infrastructure types.
- Infrastructure holds all concrete I/O and all external library glue
  (XPT/XML/SAS, filesystem, `pyreadstat`, etc.).
- Legacy coordinators are removed, and wrapper modules are either removed or no
  longer used internally.
- `pyright && ruff check . && ruff format . && pytest` pass.
- Validation suite passes: `pytest -m validation`
- No significant benchmark regression: `pytest -m benchmark --benchmark-only`
