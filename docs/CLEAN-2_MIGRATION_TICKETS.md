# CLEAN-2: Core Migration Tickets (Finish Ports & Adapters)

**Purpose:** Generate an LLM-ready, step-by-step ticket set to migrate the
remaining “old architecture” modules into the new `cli/` → `application/` →
`domain/` → `infrastructure/` structure.

**Scope of CLEAN-2:** Move/replace remaining legacy logic in:

- `cdisc_transpiler/legacy/`
- `cdisc_transpiler/services/` (where it still acts as “core”)
- `cdisc_transpiler/domains_module/`
- `cdisc_transpiler/terminology_module/`
- `cdisc_transpiler/io_module/`
- `cdisc_transpiler/metadata_module/`
- `cdisc_transpiler/mapping_module/`
- `cdisc_transpiler/xpt_module/`
- `cdisc_transpiler/xml_module/`
- `cdisc_transpiler/sas_module/`
- `cdisc_transpiler/submission_module/`

**Hard requirement (CLEAN-2 Definition of Done):**

1. No imports of `cdisc_transpiler.cli.*` outside `cdisc_transpiler/cli/`.
2. `cdisc_transpiler/application/*` no longer imports or delegates to
   `cdisc_transpiler/legacy/*`.
3. Repository ports in `cdisc_transpiler/application/ports/repositories.py` have
   concrete infrastructure implementations.
4. `StudyProcessingUseCase` and `DomainProcessingUseCase` run end-to-end using
   injected dependencies (container wiring), not legacy coordinators.
5. Full test suite passes: `pytest`.

---

## How To Use These Tickets With An LLM

**Rules for the LLM implementing tickets:**

- Implement **one ticket per PR** (or per branch) to keep diffs small and
  reviewable.
- Keep the CLI interface unchanged (commands/flags/output formats stay
  compatible).
- Preserve behavior unless a ticket explicitly allows a change.
- Update/add tests for the code you touch.
- Prefer **thin compatibility shims** instead of mass renames:
  - Leave the old module path in place, but make it import and delegate to the
    new implementation.
  - Add `DeprecationWarning` only at shim boundaries.

**Pre-flight for every ticket:**

- Run `pytest` (or the closest relevant subset) before and after.
- Use `rg` checks in acceptance criteria (explicitly listed per ticket).

---

## Migration Map (Old → Target Home)

This map is the intended “end state”. Tickets below migrate incrementally.

| Current Module                                      | Target Home                                                                                          | Notes                                                  |
| --------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| `cdisc_transpiler/legacy/*`                         | `cdisc_transpiler/application/*` + `cdisc_transpiler/domain/*` + `cdisc_transpiler/infrastructure/*` | Legacy becomes thin shims, then removed                |
| `cdisc_transpiler/cli/logging_config.py`            | `cdisc_transpiler/infrastructure/logging/*`                                                          | CLI should only adapt + configure                      |
| `cdisc_transpiler/cli/helpers.py` (non-CLI helpers) | `cdisc_transpiler/infrastructure/io/*`                                                               | Must not be imported by core logic                     |
| `cdisc_transpiler/domains_module/*`                 | `cdisc_transpiler/infrastructure/repositories/*` (+ `domain/entities`)                               | Eliminate hardcoded paths + import-time initialization |
| `cdisc_transpiler/terminology_module/*`             | `cdisc_transpiler/infrastructure/repositories/*`                                                     | Replace global CT registry with repository + caching   |
| `cdisc_transpiler/io_module/*`                      | `cdisc_transpiler/infrastructure/io/*`                                                               | Unify CSV/XLS/SAS reading behind a repository          |
| `cdisc_transpiler/metadata_module/*`                | `cdisc_transpiler/infrastructure/repositories/*`                                                     | Metadata loading becomes an adapter/repository         |
| `cdisc_transpiler/mapping_module/*`                 | `cdisc_transpiler/domain/services/*` (+ infra for config I/O)                                        | Keep `mapping_module` as compatibility wrapper         |
| `cdisc_transpiler/xpt_module/*`                     | `cdisc_transpiler/infrastructure/io/*` + `domain/services/*`                                         | Split writer vs “build domain dataframe”               |
| `cdisc_transpiler/xml_module/*`                     | `cdisc_transpiler/infrastructure/io/*`                                                               | Dataset-XML and Define-XML become infra adapters       |
| `cdisc_transpiler/sas_module/*`                     | `cdisc_transpiler/infrastructure/io/*`                                                               | SAS generation is infra output adapter                 |
| `cdisc_transpiler/submission_module/*`              | `cdisc_transpiler/domain/services/*`                                                                 | SUPPQUAL is domain logic, not “submission” glue        |

---

## Epic A — Boundary Cleanup (Core Must Not Import CLI)

### CLEAN2-A1 — Remove `cli.helpers` from core (split dataset writing)

**Priority:** P0 (unblocks circular import removal)\
**Problem:** `cdisc_transpiler/legacy/domain_processing_coordinator.py` imports
`cdisc_transpiler/cli/helpers.py` (`write_variant_splits`) which makes core
depend on CLI.

**Goal:** Move split-dataset writing logic to infrastructure and make it usable
from core without Rich/CLI imports.

**Implementation steps:**

1. Create `cdisc_transpiler/infrastructure/io/split_xpt_writer.py` (or similar)
   containing the split-writing function(s).
2. Change the function signature to accept `LoggerPort` (or be logger-agnostic).
   Do not import `cli.logging_config`.
3. Update `cdisc_transpiler/legacy/domain_processing_coordinator.py` to import
   from the new infrastructure module instead of CLI.
4. Keep `cdisc_transpiler/cli/helpers.py` as a thin wrapper that imports and
   delegates (temporary).

**Acceptance criteria:**

- `rg -n \"from \\.\\.cli\\.helpers\" cdisc_transpiler` returns no matches
  outside `cdisc_transpiler/cli/`.
- Existing integration tests that exercise split writing still pass.

**Suggested tests:**

- `pytest tests/unit/infrastructure/io/test_file_generator.py`
- `pytest tests/integration/test_study_workflow.py -k split` (or nearest
  equivalent)

---

### CLEAN2-A2 — Remove `cli.logging_config.get_logger()` usage outside CLI

**Priority:** P0\
**Problem:** Multiple non-CLI modules import
`cdisc_transpiler.cli.logging_config` (global logger singleton), creating
circular imports and hidden global state.

**Target files (non-exhaustive):**

- `cdisc_transpiler/services/domain_discovery_service.py`
- `cdisc_transpiler/services/progress_reporting_service.py`
- `cdisc_transpiler/legacy/domain_processing_coordinator.py`
- `cdisc_transpiler/legacy/domain_synthesis_coordinator.py`
- `cdisc_transpiler/xpt_module/domain_processors/*` (where applicable)

**Goal:** Core logic depends only on `LoggerPort` (injected), never on
`cli.logging_config`.

**Implementation steps:**

1. Add `logger: LoggerPort` as an injected dependency to affected service
   classes or pass it via method arguments.
2. Use `infrastructure/logging/NullLogger` as a default when constructing
   services internally (only where needed).
3. Update `DependencyContainer` to create one logger and pass it down when
   building use cases/services.
4. Update tests to pass a `NullLogger` or `ConsoleLogger` as needed.

**Acceptance criteria:**

- `rg -n \"cli\\.logging_config\" cdisc_transpiler --glob '!cdisc_transpiler/cli/**'`
  returns **no matches**.
- `DomainProcessingUseCase` and `StudyProcessingUseCase` import without runtime
  `TYPE_CHECKING` hacks for logger.

**Suggested tests:**

- `pytest tests/unit/application/`
- `pytest tests/unit/infrastructure/logging/`
- `pytest tests/integration/test_cli.py`

---

### CLEAN2-A3 — Add “architecture boundary” tests (prevent regressions)

**Priority:** P1\
**Goal:** Add a small test suite that fails if core imports CLI or application
imports legacy.

**Implementation steps:**

1. Add a unit test that scans `cdisc_transpiler/` python files (AST-based or
   simple text search).
2. Enforce:
   - No `cdisc_transpiler.cli` imports outside `cdisc_transpiler/cli/`
   - No `cdisc_transpiler.legacy` imports inside `cdisc_transpiler/application/`
3. Keep the check fast and deterministic.

**Acceptance criteria:**

- New tests are green and fail reliably when a forbidden import is introduced.

**Suggested tests:**

- `pytest tests/unit/architecture/` (new)

---

### CLEAN2-A4 — Refactor `DomainDiscoveryService` to be core-safe (no CLI imports)

**Priority:** P0\
**Problem:** `cdisc_transpiler/services/domain_discovery_service.py` imports
`cli.logging_config` inside methods and is therefore not usable from the
application/domain layers without leaking CLI concerns.

**Goal:** Make domain discovery reusable from application code by injecting
`LoggerPort` (and removing any `cli.*` imports).

**Implementation steps:**

1. Change `DomainDiscoveryService` to accept `logger: LoggerPort` in `__init__`
   (or accept it on `discover_domain_files()`).
2. Replace `from ..cli.logging_config import get_logger` calls with
   `self._logger.*`.
3. Keep SDTM categorization (`get_domain_class`) optional and only for logging
   (do not make discovery depend on it).
4. Update `StudyProcessingUseCase` (and any other call sites) to pass the
   injected logger instance.
5. Add unit tests for:
   - metadata file skipping
   - exact match vs prefix match
   - variant naming rules

**Acceptance criteria:**

- `rg -n \"cli\\.logging_config\" cdisc_transpiler/services/domain_discovery_service.py`
  returns no matches.
- Domain discovery behavior remains unchanged (same mapping output for fixture
  file sets).

**Suggested tests:**

- `pytest tests/unit/ -k domain_discovery`

---

### CLEAN2-A5 — Refactor `ProgressReportingService` to use `LoggerPort`

**Priority:** P1\
**Problem:** `cdisc_transpiler/services/progress_reporting_service.py` also
imports `cli.logging_config` and keeps hidden coupling to CLI output.

**Goal:** Ensure progress reporting is a pure service that can run in tests and
in non-CLI contexts.

**Implementation steps:**

1. Inject `LoggerPort` into `ProgressReportingService`.
2. Replace any logger singleton usage with `logger` calls.
3. Ensure it never imports Rich or CLI modules directly.
4. Update any call sites to pass the injected logger.

**Acceptance criteria:**

- `rg -n \"cli\\.logging_config\" cdisc_transpiler/services/progress_reporting_service.py`
  returns no matches.

**Suggested tests:**

- `pytest tests/unit/ -k progress_reporting`

---

## Epic B — Repositories & Configuration (Stop Hardcoded Paths + Global Registries)

### CLEAN2-B1 — Implement `SDTMSpecRepositoryPort` (infrastructure adapter)

**Priority:** P0\
**Problem:** `domains_module/registry.py` loads spec CSVs via hardcoded
repo-relative paths and initializes global registries at import time.

**Goal:** Provide `SDTMSpecRepositoryPort` implementation in
`cdisc_transpiler/infrastructure/repositories/` using
`TranspilerConfig.sdtm_spec_dir`.

**Implementation steps:**

1. Create
   `cdisc_transpiler/infrastructure/repositories/sdtm_spec_repository.py`.
2. Implement:
   - `get_domain_variables(domain_code)`
   - `get_dataset_attributes(domain_code)`
   - `list_available_domains()`
3. Use `TranspilerConfig` for paths (no `Path(__file__).../docs/...` in core
   logic).
4. Add caching (in-memory) so repeated calls don’t reread CSVs.
5. Add unit tests using small fixture CSVs (avoid depending on the full `docs/`
   payload).

**Acceptance criteria:**

- No hardcoded path usage remains in `domains_module/registry.py` after
  follow-up tickets migrate it.
- Repository works with alternate spec directories via `TranspilerConfig`.

**Suggested tests:**

- `pytest tests/unit/infrastructure/repositories/ -k sdtm_spec`

---

### CLEAN2-B2 — Implement `CTRepositoryPort` (infrastructure adapter)

**Priority:** P0\
**Problem:** `terminology_module/registry.py` uses global caches and
repo-relative directories; CT loading and normalization logic is scattered.

**Goal:** Provide a `CTRepositoryPort` implementation in infrastructure backed
by `TranspilerConfig.ct_dir`.

**Implementation steps:**

1. Create `cdisc_transpiler/infrastructure/repositories/ct_repository.py`.
2. Decide CT “version resolution” strategy (deterministic):
   - Prefer latest subfolder in `docs/Controlled_Terminology/` by ISO date
     naming, or allow explicit config.
3. Implement:
   - `get_by_code(codelist_code)`
   - `get_by_name(codelist_name)`
   - `list_all_codes()`
4. Provide memoization/caching to avoid re-reading large CT CSVs.
5. Add tests with small fake CT CSV fixture.

**Acceptance criteria:**

- CT lookup is independent of global module-level registries.
- CT path is configurable via `TranspilerConfig`.

**Suggested tests:**

- `pytest tests/unit/infrastructure/repositories/ -k ct_repository`

---

### CLEAN2-B3 — Implement `StudyDataRepositoryPort` (datasets + metadata)

**Priority:** P0\
**Problem:** `io_module/load_input_dataset()` and
`metadata_module/load_study_metadata()` are used directly from use cases. This
bypasses ports/adapters and keeps logic scattered.

**Goal:** Implement `StudyDataRepositoryPort` in
`cdisc_transpiler/infrastructure/repositories/` using:

- `infrastructure/io/CSVReader` for CSV
- optional Excel/SAS support (reuse existing implementations or gate behind
  optional deps)
- metadata loading as `load_study_metadata()`

**Implementation steps:**

1. Create
   `cdisc_transpiler/infrastructure/repositories/study_data_repository.py`.
2. Implement:
   - `read_dataset(file_path)` for CSV (and optionally XLSX/SAS)
   - `list_data_files(folder, pattern)`
   - `load_study_metadata(study_folder)` (delegating to a metadata loader
     component)
3. Add unit tests:
   - CSV read success + header normalization
   - file not found error mapping
   - metadata files missing (graceful)

**Acceptance criteria:**

- `StudyProcessingUseCase` can be refactored to only talk to this repository
  (follow-up ticket).
- `io_module/load_input_dataset()` becomes unused by application layer
  (follow-up ticket).

**Suggested tests:**

- `pytest tests/unit/infrastructure/io/test_csv_reader.py`
- `pytest tests/unit/application/ports/test_repository_contracts.py`

---

### CLEAN2-B4 — Add infrastructure caching primitives and wire them into repositories

**Priority:** P1\
**Problem:** Spec and CT parsing is expensive; current implementations rely on
module globals or ad-hoc `lru_cache`.

**Goal:** Provide a small, explicit caching adapter in
`cdisc_transpiler/infrastructure/caching/` and use it in repositories to control
memory and test behavior.

**Implementation steps:**

1. Add `cdisc_transpiler/infrastructure/caching/memory_cache.py`:
   - `get(key)`, `set(key, value)`, `clear()`
   - optional TTL is nice-to-have but not required
2. Inject the cache into:
   - `SDTMSpecRepository` (cache CSV loads / parsed rows)
   - `CTRepository` (cache CT file parse results)
3. Add tests verifying caching prevents repeated disk reads (use spies/mocks or
   temp files).

**Acceptance criteria:**

- Repositories do not use module-level mutable caches for correctness.
- Cache can be cleared between tests to avoid cross-test interference.

**Suggested tests:**

- `pytest tests/unit/infrastructure/repositories/`

---

## Epic C — Refactor Old Modules Into Thin Compatibility Layers

### CLEAN2-C1 — Refactor `domains_module` to delegate to repositories

**Priority:** P1\
**Goal:** Keep public API (`get_domain()`, `list_domains()`,
`get_domain_class()`) but remove hardcoded paths, global eager initialization,
and CSV parsing in-module.

**Implementation steps:**

1. Introduce a small `DomainRepositoryPort` (or extend `SDTMSpecRepositoryPort`)
   that returns `SDTMDomain` entities directly.
2. Implement the repository in infrastructure using the spec repository + the
   existing domain-building logic.
3. Change `domains_module/registry.py` to become a wrapper around the new
   repository (and keep `lru_cache` there only if needed for backwards
   compatibility).
4. Ensure `TranspilerConfig` can control spec paths without touching module
   code.

**Acceptance criteria:**

- `domains_module/registry.py` contains no direct filesystem path construction
  to `docs/...`.
- Domain registry is lazily initialized.
- Existing imports from `cdisc_transpiler.domains_module` still work.

**Suggested tests:**

- `pytest tests/validation/test_sdtm_compliance.py`

---

### CLEAN2-C2 — Refactor `terminology_module` to delegate to `CTRepositoryPort`

**Priority:** P1\
**Goal:** Keep current convenience functions (`normalize_testcd`,
`get_testcd_label`, etc.) but re-implement on top of the injected repository (or
a default repository instance from config).

**Implementation steps:**

1. Replace global caches with a repository instance (default created from
   `TranspilerConfig`).
2. Keep a compatibility surface in `terminology_module/__init__.py` unchanged.
3. Ensure the Findings transformers (`transformations/findings/*`) only need the
   normalizer/label getter callables (no module globals).

**Acceptance criteria:**

- No module-level CT CSV parsing logic remains in
  `terminology_module/registry.py`.
- `normalize_testcd()` and label lookups behave identically on existing
  fixtures.

**Suggested tests:**

- `pytest tests/unit/transformations/ -k vs_transformer`
- `pytest tests/unit/transformations/ -k lb_transformer`

---

### CLEAN2-C3 — Move SUPPQUAL logic into `domain/services/` (keep wrapper)

**Priority:** P1\
**Problem:** `submission_module/suppqual.py` is domain/business logic but lives
outside the domain layer.

**Goal:** Move SUPPQUAL building to
`cdisc_transpiler/domain/services/suppqual_service.py`.

**Implementation steps:**

1. Create `domain/services/suppqual_service.py` with:
   - `build_suppqual(...)`
   - `extract_used_columns(...)`
2. Keep `submission_module/suppqual.py` as a thin wrapper importing from the new
   location (temporary).
3. Update legacy coordinator and new use case code to call domain service
   directly.
4. Add unit tests for:
   - QNAM sanitization
   - missing USUBJID handling
   - deduping behavior

**Acceptance criteria:**

- No application/service code imports `submission_module` directly after
  follow-ups.

**Suggested tests:**

- `pytest tests/unit/ -k suppqual`

---

### CLEAN2-C4 — Migrate `metadata_module` loaders into infrastructure repository adapters

**Priority:** P1\
**Problem:** Metadata loading (`Items.csv`, `CodeLists.csv`) is still in
`metadata_module/*` and used directly by use cases.

**Goal:** Move metadata file parsing to infrastructure (repository/adapter),
keep `metadata_module` as a compatibility wrapper.

**Implementation steps:**

1. Create
   `cdisc_transpiler/infrastructure/repositories/study_metadata_loader.py` (or
   embed inside `StudyDataRepository`).
2. Move parsing logic from:
   - `cdisc_transpiler/metadata_module/loaders.py`
   - `cdisc_transpiler/metadata_module/csv_utils.py` into the infrastructure
     component.
3. Ensure all produced models are `domain.entities.study_metadata.*` (no
   duplicate model types).
4. Update `StudyDataRepository.load_study_metadata()` to call the new loader.
5. Convert `metadata_module/loaders.py` into a thin wrapper (temporary)
   delegating to the new loader.

**Acceptance criteria:**

- Application layer no longer imports `metadata_module.load_study_metadata`.
- Only one set of metadata models is used across the codebase (domain entities).

**Suggested tests:**

- `pytest tests/unit/ -k metadata`

---

### CLEAN2-C5 — Deprecate `io_module` by routing all reads through `StudyDataRepositoryPort`

**Priority:** P1\
**Problem:** `io_module/readers.py` duplicates behavior with
`infrastructure/io/CSVReader` and bypasses ports/adapters.

**Goal:** Make `io_module` a compatibility wrapper only; all internal reads go
through `StudyDataRepositoryPort`.

**Implementation steps:**

1. Ensure `StudyDataRepository.read_dataset()` supports the currently used
   formats:
   - CSV (required)
   - XLS/XLSX and SAS7BDAT (optional; keep parity with `io_module` if
     dependencies exist)
2. Replace internal usage of `load_input_dataset()` with repository calls:
   - `StudyProcessingUseCase` column counting
   - legacy coordinator (until removed) or new `DomainProcessingUseCase`
3. Convert `io_module/load_input_dataset()` into a wrapper that calls the
   repository (temporary).
4. Add tests that exercise each supported format (CSV mandatory; others skip if
   deps missing).

**Acceptance criteria:**

- `rg -n \"load_input_dataset\\(\" cdisc_transpiler/application` returns no
  matches.
- Internal code has exactly one dataset-reading path (the repository).

**Suggested tests:**

- `pytest tests/unit/infrastructure/repositories/ -k study_data_repository`

---

### CLEAN2-C6 — Move mapping config I/O to infrastructure (keep `mapping_module` surface stable)

**Priority:** P2\
**Problem:** `mapping_module/config_io.py` is infrastructure (filesystem I/O)
but currently lives in a domain-ish module.

**Goal:** Create a mapping-config repository adapter under infrastructure and
keep existing `mapping_module.load_config/save_config` working as wrappers.

**Implementation steps:**

1. Create
   `cdisc_transpiler/infrastructure/repositories/mapping_config_repository.py`:
   - `load(path) -> MappingConfig`
   - `save(config, path) -> None`
2. Update `mapping_module/config_io.py` to delegate to that repository.
3. Ensure JSON schema remains unchanged (backward compatible).
4. Add tests:
   - round-trip load/save
   - unknown fields handling (if supported)

**Acceptance criteria:**

- No direct filesystem I/O for mapping configs remains outside infrastructure.

**Suggested tests:**

- `pytest tests/unit/ -k mapping_config`

---

### CLEAN2-C7 — Move mapping engines into `domain/services/` (keep wrapper)

**Priority:** P2\
**Problem:** `mapping_module/engine.py` and `mapping_module/metadata_mapper.py`
are core business logic but live outside the domain layer.

**Goal:** Move mapping algorithms under
`cdisc_transpiler/domain/services/mapping/` while keeping the
`cdisc_transpiler.mapping_module` public API stable.

**Implementation steps:**

1. Create `cdisc_transpiler/domain/services/mapping/` package:
   - `engine.py` (MappingEngine)
   - `metadata_mapper.py` (MetadataAwareMapper)
   - `factory.py` (create_mapper)
2. Update `mapping_module/__init__.py` to re-export from the new location.
3. Ensure mapping models are imported from `domain.entities.mapping`.
4. Update internal code to import mapping services from domain directly (stop
   importing via wrapper).

**Acceptance criteria:**

- No non-wrapper mapping logic remains in `cdisc_transpiler/mapping_module/`
  after migration.

**Suggested tests:**

- `pytest tests/unit/ -k mapping`

---

### CLEAN2-C8 — Move `xpt_module/builder.build_domain_dataframe` into domain services (keep wrapper)

**Priority:** P1\
**Problem:** Domain dataframe building is core SDTM business logic but lives in
`xpt_module/builder.py` (mixed with output concerns).

**Goal:** Create `cdisc_transpiler/domain/services/domain_frame_builder.py` and
move the build logic there.

**Implementation steps:**

1. Create `domain/services/domain_frame_builder.py` exposing:
   - `build_domain_dataframe(...)`
2. Move logic from `xpt_module/builder.py` with minimal behavior changes.
3. Update imports so this service uses:
   - domain entities (`SDTMDomain`, variables)
   - transformation helpers in `cdisc_transpiler/transformations/*` where
     appropriate
4. Convert `xpt_module/builder.py` into a wrapper calling the new service
   (temporary).

**Acceptance criteria:**

- `DomainProcessingUseCase` calls the domain service builder, not
  `xpt_module/builder.py`.

**Suggested tests:**

- `pytest tests/validation/test_sdtm_compliance.py`

---

### CLEAN2-C9 — Move `xpt_module/domain_processors/` into domain services (keep wrapper)

**Priority:** P2\
**Problem:** `xpt_module/domain_processors/*` contains domain-specific core
logic, and some modules import CLI logging helpers.

**Goal:** Relocate domain-specific processing logic to
`cdisc_transpiler/domain/services/domain_processors/` and keep
`xpt_module/domain_processors` as a re-exporting wrapper package.

**Implementation steps:**

1. Create `cdisc_transpiler/domain/services/domain_processors/` with a stable
   internal interface:
   - base processor protocol/class (if needed)
   - per-domain processors (`dm.py`, `lb.py`, `ae.py`, ...)
2. Move files from `cdisc_transpiler/xpt_module/domain_processors/` into the new
   location.
3. Remove all `cli.logging_config` imports from processors (inject `LoggerPort`
   or remove logging entirely).
4. Update the domain dataframe builder (new `domain_frame_builder.py`) to import
   processors from the new domain location.
5. Update `xpt_module/domain_processors/__init__.py` to re-export from the new
   location for compatibility.

**Acceptance criteria:**

- `rg -n \"cli\\.logging_config\" cdisc_transpiler/xpt_module/domain_processors`
  returns no matches.
- Existing domain-specific behaviors remain unchanged (validated by
  integration/validation tests).

**Suggested tests:**

- `pytest tests/integration/test_domain_workflow.py`

---

## Epic D — Implement Real Use Cases (Stop Delegating To Legacy)

### CLEAN2-D1 — Make `DomainProcessingUseCase` real (pipeline orchestration)

**Priority:** P0\
**Problem:** `DomainProcessingUseCase` currently delegates to
`legacy.DomainProcessingCoordinator` due to circular imports and mixed
responsibilities.

**Goal:** Implement domain processing as explicit pipeline stages using injected
dependencies (ports).

**Required pipeline stages (minimum):**

1. Load input file(s) via `StudyDataRepositoryPort`
2. Apply transformations via `TransformationPipeline` (VS/LB at least)
3. Map columns via mapping service/engine (existing `mapping_module` ok
   initially)
4. Build SDTM domain dataframe (wrap existing
   `xpt_module/builder.build_domain_dataframe` initially)
5. Generate SUPPQUAL (domain service)
6. Generate outputs via `FileGeneratorPort`

**Implementation steps:**

1. Extend `DependencyContainer` to construct and inject required dependencies
   into `DomainProcessingUseCase`.
2. Refactor `DomainProcessingUseCase.execute()` to implement the stages above
   and return `ProcessDomainResponse`.
3. Leave `legacy.DomainProcessingCoordinator` as a wrapper calling the new use
   case until CLEAN2-CLEANUP.
4. Remove runtime-import hacks once circular imports are gone.

**Acceptance criteria:**

- `cdisc_transpiler/application/domain_processing_use_case.py` contains **no**
  `from ..services import DomainProcessingCoordinator`.
- `rg -n \"legacy\\.domain_processing_coordinator\" cdisc_transpiler/application`
  returns no matches.
- Domain processing integration tests pass for at least one study.

**Suggested tests:**

- `pytest tests/unit/application/test_domain_processing_use_case.py` (add if
  missing)
- `pytest tests/integration/test_domain_workflow.py`

---

### CLEAN2-D2 — Make `StudyProcessingUseCase` real (no legacy orchestration)

**Priority:** P0\
**Problem:** `StudyProcessingUseCase` currently imports and uses legacy
coordinators + old modules directly.

**Goal:** `StudyProcessingUseCase` orchestrates only:

- discovery → per-domain processing → synthesis → define.xml generation using
  injected ports/use cases.

**Implementation steps:**

1. Inject into `StudyProcessingUseCase`:
   - `StudyDataRepositoryPort`
   - domain discovery component (no CLI imports)
   - `DomainProcessingUseCase`
   - synthesis service (trial design + empty domains)
   - define.xml generator adapter
2. Replace direct imports of:
   - `domains_module.list_domains/get_domain`
   - `io_module.load_input_dataset`
   - `metadata_module.load_study_metadata`
   - `services.Domain*Coordinator` (legacy)
3. Keep the Click command unchanged; only wiring changes via
   `DependencyContainer`.

**Acceptance criteria:**

- `rg -n \"from \\.+legacy\" cdisc_transpiler/application/study_processing_use_case.py`
  has no matches.
- Use case has a constructor that takes dependencies (no instantiating services
  inside `__init__`).

**Suggested tests:**

- `pytest tests/unit/application/`
- `pytest tests/integration/test_study_workflow.py`

---

### CLEAN2-D3 — Implement synthesis as a domain/application service (replace legacy coordinator)

**Priority:** P1\
**Problem:** `legacy/domain_synthesis_coordinator.py` synthesizes required
domains and writes files directly using old modules.

**Goal:** Create a synthesis service that:

- generates trial design domains (TS/TA/TE/SE/DS)
- generates empty required observation domains (AE/LB/VS/EX) when missing
- delegates file generation to `FileGeneratorPort`

**Implementation steps:**

1. Create `cdisc_transpiler/domain/services/synthesis_service.py` (or
   `cdisc_transpiler/synthesis/*` with a thin domain-facing API).
2. Reuse and/or move logic from
   `cdisc_transpiler/services/trial_design_service.py` (but remove dependencies
   on old modules).
3. Ensure synthesized domains use the same mapping/config model types as real
   domains.
4. Update `StudyProcessingUseCase` to call this service instead of
   `legacy.DomainSynthesisCoordinator`.
5. Add unit tests for at least TS + one observation domain synthesis
   (structure + required columns).

**Acceptance criteria:**

- No application code imports `legacy/domain_synthesis_coordinator.py`.
- Synthesis output is included in Define-XML when enabled.

**Suggested tests:**

- `pytest tests/integration/test_study_workflow.py -k synth`

---

### CLEAN2-D4 — Implement RELREC generation without `StudyOrchestrationService`

**Priority:** P2\
**Problem:** Relationship record generation is currently tied to legacy
orchestration logic.

**Goal:** Build RELREC as a pure service that consumes processed domain
dataframes and emits a RELREC dataframe + mapping config.

**Implementation steps:**

1. Identify the minimum linking rules currently implemented (from legacy
   orchestration).
2. Create `cdisc_transpiler/domain/services/relrec_service.py`:
   - input: `{domain_code: pd.DataFrame}`
   - output: `pd.DataFrame` for RELREC + config
3. Update `StudyProcessingUseCase` to generate RELREC after domain processing
   (and after synthesis where needed).
4. Generate output files via `FileGeneratorPort`.
5. Add unit tests:
   - empty inputs → empty structured RELREC
   - at least one link rule produces expected RELREC rows

**Acceptance criteria:**

- `legacy/study_orchestration_service.py` is no longer needed for RELREC.

**Suggested tests:**

- `pytest tests/unit/ -k relrec`

---

## Epic E — Output Adapters (XPT / Dataset-XML / Define-XML / SAS)

### CLEAN2-E1 — Convert `FileGenerator` to true port adapter (inject writers)

**Priority:** P1\
**Problem:** `infrastructure/io/file_generator.py` still imports module-level
writer functions from old modules (`xpt_module`, `xml_module`, `sas_module`).

**Goal:** `FileGenerator` depends on injected writer adapters, not old module
functions.

**Implementation steps:**

1. Create writer adapters in `cdisc_transpiler/infrastructure/io/`:
   - `xpt_writer.py` (wrap current `xpt_module/write_xpt_file`)
   - `dataset_xml_writer.py` (wrap current
     `xml_module/dataset_module/write_dataset_xml`)
   - `sas_writer.py` (wrap current `sas_module/generator.py` + writer)
2. Update `FileGenerator.__init__` to accept these adapters.
3. Update `DependencyContainer.create_file_generator()` to wire the adapters.
4. Keep old modules as wrappers initially; remove later.

**Acceptance criteria:**

- `infrastructure/io/file_generator.py` has no imports from `.../xpt_module`,
  `.../xml_module`, `.../sas_module`.

**Suggested tests:**

- `pytest tests/unit/infrastructure/io/test_file_generator.py`

---

### CLEAN2-E2 — Define-XML generation as an infrastructure adapter

**Priority:** P1\
**Problem:** Define-XML generation is still invoked via
`xml_module.define_module` directly from use cases.

**Goal:** Create a `DefineXmlGenerator` adapter in infrastructure and inject it
into `StudyProcessingUseCase`.

**Implementation steps:**

1. Add a port interface (e.g., `DefineXmlGeneratorPort`) in
   `application/ports/services.py`.
2. Implement `infrastructure/io/define_xml_generator.py` delegating to current
   `xml_module.define_module` functions initially.
3. Refactor `StudyProcessingUseCase` to call the port, not the module.
4. Add tests for successful generation on a small in-memory `StudyDataset`
   fixture.

**Acceptance criteria:**

- Application layer no longer imports
  `cdisc_transpiler.xml_module.define_module`.

**Suggested tests:**

- `pytest tests/validation/test_define_xml_format.py`

---

### CLEAN2-E3 — Convert `xpt_module`, `xml_module`, and `sas_module` into wrappers (internal imports use infrastructure)

**Priority:** P2\
**Goal:** After E1/E2, ensure internal code no longer imports output modules
directly; those packages remain only as compatibility surfaces.

**Implementation steps:**

1. Replace internal imports:
   - `from ..xpt_module import write_xpt_file` → use injected
     `XPTWriter`/`FileGeneratorPort`
   - `from ..xml_module.dataset_module import write_dataset_xml` → use injected
     adapter
   - `from ..sas_module import generate_sas_program` → use injected adapter
2. Keep the public functions in those modules but delegate to infrastructure
   adapters.
3. Add `DeprecationWarning` at wrapper entrypoints only (optional until v1.0).

**Acceptance criteria:**

- `rg -n \"from \\.+xpt_module\" cdisc_transpiler/application cdisc_transpiler/domain`
  returns no matches.
- Same for `xml_module` and `sas_module`.

**Suggested tests:**

- `pytest tests/unit/`

---

### CLEAN2-E4 — Retire `services/FileGenerationService` (single file-generation path)

**Priority:** P2\
**Problem:** There are multiple competing “file generation” implementations
(`services/file_generation_service.py` and
`infrastructure/io/file_generator.py`), which increases drift risk.

**Goal:** Make `FileGeneratorPort` + `infrastructure/io/FileGenerator` the only
internal file generation mechanism.

**Implementation steps:**

1. Ensure all internal code uses `FileGeneratorPort.generate()` (directly or via
   use cases).
2. Convert `services/file_generation_service.py` into:
   - a thin wrapper around `FileGeneratorPort` (temporary), or
   - remove it entirely if unused externally (verify with `rg`).
3. Update `services/__init__.py` exports accordingly.
4. Add a test that guards against re-introducing duplicate generation paths
   (optional).

**Acceptance criteria:**

- `rg -n \"FileGenerationService\" cdisc_transpiler` returns matches only in
  wrappers/tests (or none).

**Suggested tests:**

- `pytest tests/unit/infrastructure/io/test_file_generator.py`

---

### CLEAN2-E5 — Move Dataset-XML + Define-XML implementation into infrastructure

**Priority:** P1\
**Status:** ⏳ Not Started (blocker resolved) — the prior circular import between
`cdisc_transpiler.domains_module` and `cdisc_transpiler.domain.entities.mapping`
has been addressed via lazy domain lookup in `MappingConfig`. Ticket is ready to
proceed with implementation in a follow-up PR.
**Problem:** Even after adding adapters, the real implementation still lives in
`xml_module/*`, which is the “old layout”.

**Goal:** Move implementation modules to `cdisc_transpiler/infrastructure/io/`
and keep `xml_module` as a thin compatibility surface.

**Implementation steps:**

1. Move (or copy then switch) implementation code:
   - `xml_module/dataset_module/*` → `infrastructure/io/dataset_xml/*`
   - `xml_module/define_module/*` → `infrastructure/io/define_xml/*`
2. Update adapters created in CLEAN2-E1/E2 to call the new implementation
   directly.
3. Keep `xml_module/*` modules as wrappers that import from infrastructure.
4. Run validation tests for Dataset-XML and Define-XML.

**Acceptance criteria:**

- No internal code imports from `cdisc_transpiler/xml_module` (wrappers only for
  external callers).

**Suggested tests:**

- `pytest tests/validation/test_xml_format.py`
- `pytest tests/validation/test_define_xml_format.py`

---

### CLEAN2-E6 — Move XPT implementation into infrastructure

**Priority:** P1\
**Status:** ⏳ Not Started — dependency on CLEAN2-E5 circular import has been
resolved; work remains to move the implementation.
**Problem:** XPT writing and validation still live in `xpt_module/*` even after
adding adapters.

**Goal:** Move XPT implementation to `cdisc_transpiler/infrastructure/io/xpt/*`
and keep `xpt_module` as a wrapper.

**Implementation steps:**

1. Move (or copy then switch) implementation files:
   - `xpt_module/writer.py` → `infrastructure/io/xpt/writer.py`
   - `xpt_module/validators.py` → `infrastructure/io/xpt/validators.py`
   - `xpt_module/transformers/*` → `infrastructure/io/xpt/transformers/*` (or
     consolidate with `transformations/` where it makes sense)
2. Update the `XPTWriter` adapter from CLEAN2-E1 to call the new implementation.
3. Convert `xpt_module/__init__.py` exports to delegate to infrastructure.
4. Run XPT validation tests.

**Acceptance criteria:**

- No internal code imports from `cdisc_transpiler/xpt_module` directly (wrappers
  only).

**Suggested tests:**

- `pytest tests/validation/test_xpt_format.py`

---

### CLEAN2-E7 — Move SAS implementation into infrastructure

**Priority:** P1\
**Status:** ⏳ Not Started — dependency on CLEAN2-E5 circular import has been
resolved; work remains to move the implementation.
**Problem:** SAS program generation still lives in `sas_module/*`.

**Goal:** Move SAS generation/writing to
`cdisc_transpiler/infrastructure/io/sas/*` and keep `sas_module` as a wrapper.

**Implementation steps:**

1. Move (or copy then switch) implementation files:
   - `sas_module/generator.py` → `infrastructure/io/sas/generator.py`
   - `sas_module/writer.py` → `infrastructure/io/sas/writer.py`
2. Update the `SASWriter` adapter from CLEAN2-E1 to call the new implementation.
3. Convert `sas_module/__init__.py` exports to delegate to infrastructure.
4. Run integration tests that generate SAS output.

**Acceptance criteria:**

- No internal code imports from `cdisc_transpiler/sas_module` directly (wrappers
  only).

**Suggested tests:**

- `pytest tests/integration/test_study_workflow.py -k sas`

---

## Epic F — Cleanup (Remove Legacy + Old Modules)

### CLEAN2-F1 — Deprecation + shim pass (old modules become wrappers)

**Priority:** P1\
**Goal:** After the new use cases run end-to-end, reduce old modules to
compatibility shells and remove duplicated logic.

**Implementation steps:**

1. For each migrated area, keep the public function/class but delegate to the
   new implementation:
   - `io_module/*` → `infrastructure/repositories/study_data_repository.py`
   - `domains_module/*` → `infrastructure/repositories/*`
   - `terminology_module/*` → `infrastructure/repositories/ct_repository.py`
   - `submission_module/*` → `domain/services/suppqual_service.py`
   - `mapping_module/*` → `domain/services/mapping/*` (+ infra config repo)
   - `xpt_module/*` / `xml_module/*` / `sas_module/*` → `infrastructure/io/*`
2. Add `DeprecationWarning` only in the wrapper entrypoints, not deep code.
3. Update internal imports to use the new locations (stop “going through”
   wrappers inside the project).

**Acceptance criteria:**

- Core code uses new modules directly; wrappers are only for external callers.

---

### CLEAN2-F2 — Remove `cdisc_transpiler/legacy/` (final step)

**Priority:** P0 (final gate)\
**Goal:** Delete `cdisc_transpiler/legacy/` once no internal code depends on it.

**Implementation steps:**

1. Verify `rg -n \"cdisc_transpiler\\.legacy\" cdisc_transpiler tests` only
   shows references in docs (or none).
2. Remove `cdisc_transpiler/legacy/` and update:
   - `cdisc_transpiler/services/__init__.py` (stop re-exporting legacy)
   - any remaining compatibility warnings
3. Run the full test suite.

**Acceptance criteria:**

- No `legacy` package exists.
- `pytest` passes.
