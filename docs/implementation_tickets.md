# Implementation Tickets: Study Command Refactoring

**Project:** CDISC Transpiler Refactoring  
**Epic:** Study Command Flow Redesign  
**Status:** Ready for Implementation  
**Last Updated:** 2025-12-14

---

## Quick Reference

- **Total Tickets:** 60
- **Estimated Total Effort:** 6 weeks (1 developer)
- **Epics:** 6 major work streams
- **Priority:** High (addresses technical debt and maintainability)

---

## Epic 1: Infrastructure Layer (Week 1)

### INFRA-1: Create New Folder Structure
**Priority:** P0 (Blocker for other work)  
**Effort:** 2 hours  
**Description:** Set up the new architectural folder structure without deleting existing code.

**Tasks:**
- [ ] Create `infrastructure/` directory with subdirectories
- [ ] Create `domain/` directory with subdirectories  
- [ ] Create `application/` directory with subdirectories
- [ ] Create `transformations/` directory with subdirectories
- [ ] Create `synthesis/` directory
- [ ] Update `.gitignore` if needed
- [ ] Create `__init__.py` files for all new packages

**Acceptance Criteria:**
- New folder structure exists alongside old structure
- All new directories are Python packages (have `__init__.py`)
- Can import from new packages without errors

---

### INFRA-2: Implement Unified CSV Reader
**Priority:** P0 (Blocker for file I/O)  
**Effort:** 4 hours  
**Depends on:** INFRA-1  
**Description:** Create single source of truth for CSV reading, replacing 3 different implementations.

**Current implementations to replace:**
- `io_module/readers.py:40` - `load_input_dataset()`
- `metadata_module/loaders.py:70` - `_load_items_csv()`
- `metadata_module/csv_utils.py:10` - `read_csv_safely()`

**Tasks:**
- [ ] Create `infrastructure/io/csv_reader.py`
- [ ] Implement `CSVReadOptions` dataclass for configuration
- [ ] Implement `CSVReader` class with consistent behavior:
  - Consistent `dtype` handling
  - Consistent `na_values` settings
  - Optional header normalization
  - Consistent error handling
- [ ] Define custom exceptions: `DataSourceNotFoundError`, `DataParseError`
- [ ] Write unit tests:
  - Test header normalization
  - Test NA value handling
  - Test file not found error
  - Test malformed CSV error
  - Test encoding handling

**Acceptance Criteria:**
- CSVReader can read all types of CSV files in the project
- All tests pass with >95% coverage
- Error messages are clear and actionable
- Can configure behavior without changing implementation

**Files to create:**
```
infrastructure/
└── io/
    ├── __init__.py
    ├── csv_reader.py       # Main implementation
    └── exceptions.py       # Custom exceptions

tests/
└── unit/
    └── infrastructure/
        └── io/
            └── test_csv_reader.py
```

---

### INFRA-3: Implement Unified File Generator
**Priority:** P1  
**Effort:** 6 hours  
**Depends on:** INFRA-1  
**Description:** Consolidate file generation logic that's duplicated across 3+ services.

**Current duplicates to replace:**
- `domain_processing_coordinator.py:531-570` - XPT/XML/SAS generation
- `domain_synthesis_coordinator.py:354-386` - Same pattern
- `study_orchestration_service.py:590-617` - Same pattern

**Tasks:**
- [ ] Create `infrastructure/io/file_generator.py`
- [ ] Implement `OutputRequest` dataclass (input)
- [ ] Implement `OutputResult` dataclass (output)
- [ ] Implement `OutputDirs` dataclass (directory configuration)
- [ ] Implement `FileGenerator` class with methods:
  - `generate()` - main entry point
  - `_generate_xpt()` - XPT file generation
  - `_generate_xml()` - XML file generation
  - `_generate_sas()` - SAS file generation
- [ ] Integrate with existing writers (delegate to them)
- [ ] Write unit tests:
  - Test XPT generation
  - Test XML generation
  - Test SAS generation
  - Test multi-format generation
  - Test error handling for each format
  - Test logging behavior

**Acceptance Criteria:**
- Single method call generates all requested formats
- Consistent error handling across all formats
- Consistent logging messages
- All tests pass with >90% coverage

**Files to create:**
```
infrastructure/
└── io/
    ├── file_generator.py   # Main implementation
    └── models.py           # OutputRequest, OutputResult, etc.

tests/
└── unit/
    └── infrastructure/
        └── io/
            └── test_file_generator.py
```

---

### INFRA-4: Create Configuration System
**Priority:** P0 (Needed throughout)  
**Effort:** 4 hours  
**Depends on:** INFRA-1  
**Description:** Create centralized configuration management to eliminate scattered magic values.

**Current problems:**
- Hardcoded paths in multiple loaders
- Magic dates ("2023-01-01") in 3 places
- Magic confidence values (0.5, 0.7)
- No way to override defaults without code changes

**Tasks:**
- [ ] Create `config.py` at root with `TranspilerConfig` dataclass:
  - Paths (sdtm_spec_dir, ct_dir)
  - Processing defaults (min_confidence, chunk_size)
  - Synthesis defaults (default_date, default_subject)
  - Constraints (max_xpt_label_length, max_xpt_variables)
- [ ] Implement `ConfigLoader.from_env()` for environment variable overrides
- [ ] Implement `ConfigLoader.from_toml()` for config file support
- [ ] Make config immutable (frozen=True)
- [ ] Write unit tests:
  - Test default values
  - Test environment variable overrides
  - Test TOML file loading
  - Test validation of paths
  - Test validation of numeric values

**Acceptance Criteria:**
- Configuration is centralized in one place
- Can override via environment variables
- Can override via TOML file (optional)
- All defaults are documented
- Config is immutable after creation

**Files to create:**
```
config.py                           # Main config
cdisc_transpiler.toml.example      # Example config file

tests/
└── unit/
    └── test_config.py
```

**Example usage:**
```python
# Default config
config = TranspilerConfig()

# From environment
config = TranspilerConfig.from_env()

# From TOML
config = ConfigLoader.load()
```

---

### INFRA-5: Extract Constants
**Priority:** P1  
**Effort:** 2 hours  
**Depends on:** INFRA-1  
**Description:** Move all magic values to a constants module for easy reference.

**Current scattered constants:**
- "2023-01-01" (default date)
- "SYNTH001" (default subject)
- 0.5 (min confidence)
- 1000 (chunk size)
- 200 (max XPT label length)
- 40 (max XPT variables)
- 8 (max QNAM length)

**Tasks:**
- [ ] Create `constants.py` with classes:
  - `Defaults` - default values
  - `Constraints` - system limits
  - `Patterns` - regex patterns
- [ ] Document each constant with SDTM reference
- [ ] Make constants type-safe (use literals where possible)
- [ ] Write unit tests for any computed constants

**Acceptance Criteria:**
- All magic values have descriptive names
- Each constant has docstring with SDTM reference
- No hardcoded magic values in services

**Files to create:**
```
constants.py

tests/
└── unit/
    └── test_constants.py
```

---

### INFRA-6: Implement Logger Interface
**Priority:** P2  
**Effort:** 3 hours  
**Depends on:** INFRA-1  
**Description:** Create logger interface for dependency injection, eliminating global logger state.

**Current problems:**
- Global `get_logger()` singleton
- Services directly import from `cli.logging_config`
- Hard to test with mocked logging
- Tight coupling to Rich console

**Tasks:**
- [ ] Create `application/ports/services.py` with `LoggerPort` protocol
- [ ] Move existing `SDTMLogger` to `infrastructure/logging/console_logger.py`
- [ ] Implement `LoggerPort` interface in `ConsoleLogger`
- [ ] Create `NullLogger` for testing (silent logger)
- [ ] Update dependency injection to pass logger
- [ ] Write unit tests:
  - Test LoggerPort interface compliance
  - Test ConsoleLogger behavior
  - Test NullLogger (no output)

**Acceptance Criteria:**
- Logger is injected, not globally accessed
- Easy to swap implementations (console vs file vs null)
- Services can be tested without console output
- Backward compatibility maintained for CLI

**Files to create:**
```
application/
└── ports/
    └── services.py             # LoggerPort protocol

infrastructure/
└── logging/
    ├── __init__.py
    ├── console_logger.py       # Renamed from logging_config
    └── null_logger.py          # For testing

tests/
└── unit/
    └── infrastructure/
        └── logging/
            └── test_loggers.py
```

---

## Epic 2: Domain Layer (Week 2)

### DOMAIN-1: Reorganize Domain Entities
**Priority:** P0 (Foundation for domain layer)  
**Effort:** 3 hours  
**Depends on:** INFRA-1  
**Description:** Move domain entities to new location without logic changes (pure refactoring).

**Tasks:**
- [ ] Move `domains_module/models.py` → `domain/entities/sdtm_domain.py`
- [ ] Move `domains_module/variable_builder.py` → `domain/entities/variable.py`
- [ ] Move `metadata_module/models.py` → `domain/entities/study_metadata.py`
- [ ] Move `mapping_module/models.py` → `domain/entities/mapping.py`
- [ ] Update all imports in moved files
- [ ] Update `__init__.py` files for clean imports
- [ ] Run existing functionality tests to ensure no breakage

**Acceptance Criteria:**
- All files moved successfully
- No logic changes (pure move)
- All imports updated
- Existing tests still pass (if any)

**Files affected:**
```
domain/
└── entities/
    ├── __init__.py
    ├── sdtm_domain.py      # From domains_module/models.py
    ├── variable.py         # From domains_module/variable_builder.py
    ├── study_metadata.py   # From metadata_module/models.py
    └── mapping.py          # From mapping_module/models.py
```

---

### DOMAIN-2: Create Transformer Interface
**Priority:** P0 (Foundation for transformations)  
**Effort:** 2 hours  
**Depends on:** DOMAIN-1  
**Description:** Define abstract interface for all data transformers.

**Tasks:**
- [ ] Create `transformations/base.py`
- [ ] Define `TransformerPort` protocol with methods:
  - `can_transform(df, domain) -> bool`
  - `transform(df, context) -> pd.DataFrame`
- [ ] Define `TransformationContext` dataclass for passing metadata
- [ ] Define `TransformationResult` dataclass for rich results
- [ ] Document interface with examples
- [ ] Write unit tests for context and result dataclasses

**Acceptance Criteria:**
- Clear interface definition
- Type hints on all methods
- Documentation with usage examples
- Tests for data structures

**Files to create:**
```
transformations/
├── __init__.py
└── base.py                 # TransformerPort protocol

tests/
└── unit/
    └── transformations/
        └── test_base.py
```

---

### DOMAIN-3: Implement Generic Wide-to-Long Transformer
**Priority:** P1  
**Effort:** 8 hours  
**Depends on:** DOMAIN-2  
**Description:** Extract common wide-to-long logic from VS and LB transformers (~60% duplicate code).

**Current duplication:**
- `study_orchestration_service.py:55-199` (VS transformation)
- `study_orchestration_service.py:201-406` (LB transformation)

**Tasks:**
- [ ] Create `transformations/findings/wide_to_long.py`
- [ ] Implement `TestColumnPattern` dataclass for matching
- [ ] Implement `WideToLongTransformer` base class:
  - Generic column discovery (regex patterns)
  - Generic row unpivoting logic
  - CT-based test code normalization
  - Configurable output column mapping
- [ ] Write comprehensive unit tests:
  - Test column pattern matching
  - Test row unpivoting
  - Test CT integration
  - Test empty data handling
  - Test mixed formats

**Acceptance Criteria:**
- Base transformer works for both VS and LB
- >90% test coverage
- Clear separation of generic vs domain-specific logic
- Performance not degraded vs current implementation

**Files to create:**
```
transformations/
└── findings/
    ├── __init__.py
    └── wide_to_long.py     # Base transformer

tests/
└── unit/
    └── transformations/
        └── findings/
            └── test_wide_to_long.py
```

---

### DOMAIN-4: Refactor VS Transformer
**Priority:** P1  
**Effort:** 4 hours  
**Depends on:** DOMAIN-3  
**Description:** Refactor VS transformation to use generic base, removing duplication.

**Tasks:**
- [ ] Create `transformations/findings/vs_transformer.py`
- [ ] Implement `VSTransformer(WideToLongTransformer)`:
  - Define VS-specific column patterns
  - Define VS-specific output mapping (VSTESTCD, VSTEST, etc.)
  - Implement VS-specific validations if any
- [ ] Replace usage in `domain_processing_coordinator.py`
- [ ] Write VS-specific unit tests
- [ ] Verify output matches current behavior (integration test)

**Acceptance Criteria:**
- VS transformer extends base class
- VS-specific logic clearly separated
- Output matches current implementation
- Tests cover VS-specific patterns

**Files to create:**
```
transformations/
└── findings/
    └── vs_transformer.py

tests/
└── unit/
    └── transformations/
        └── findings/
            └── test_vs_transformer.py
```

---

### DOMAIN-5: Refactor LB Transformer
**Priority:** P1  
**Effort:** 4 hours  
**Depends on:** DOMAIN-3  
**Description:** Refactor LB transformation to use generic base, removing duplication.

**Tasks:**
- [ ] Create `transformations/findings/lb_transformer.py`
- [ ] Implement `LBTransformer(WideToLongTransformer)`:
  - Define LB-specific column patterns
  - Define LB-specific output mapping (LBTESTCD, LBTEST, etc.)
  - Include normal range handling (LBORNRLO, LBORNRHI)
  - Implement LB-specific validations if any
- [ ] Replace usage in `domain_processing_coordinator.py`
- [ ] Write LB-specific unit tests
- [ ] Verify output matches current behavior (integration test)

**Acceptance Criteria:**
- LB transformer extends base class
- LB-specific logic clearly separated (normal ranges)
- Output matches current implementation
- Tests cover LB-specific patterns

**Files to create:**
```
transformations/
└── findings/
    └── lb_transformer.py

tests/
└── unit/
    └── transformations/
        └── findings/
            └── test_lb_transformer.py
```

---

### DOMAIN-6: Create Transformation Pipeline
**Priority:** P1  
**Effort:** 3 hours  
**Depends on:** DOMAIN-2, DOMAIN-4, DOMAIN-5  
**Description:** Implement pipeline for composing transformations in sequence.

**Tasks:**
- [ ] Create `transformations/pipeline.py`
- [ ] Implement `TransformationPipeline` class:
  - Register transformers in order
  - Execute applicable transformers sequentially
  - Collect transformation metadata
  - Handle errors gracefully
- [ ] Write unit tests:
  - Test single transformer execution
  - Test multiple transformer chain
  - Test error handling
  - Test transformer skipping (can_transform = False)

**Acceptance Criteria:**
- Pipeline can compose any transformers
- Order is explicit and configurable
- Errors don't break entire pipeline (optional fail-safe)
- Metadata collected for debugging

**Files to create:**
```
transformations/
└── pipeline.py

tests/
└── unit/
    └── transformations/
        └── test_pipeline.py
```

---

### DOMAIN-7: Extract Date Transformer
**Priority:** P2  
**Effort:** 4 hours  
**Depends on:** DOMAIN-2  
**Description:** Extract date transformation logic into separate transformer.

**Current location:** `xpt_module/builder.py` (mixed with other logic)

**Tasks:**
- [ ] Create `transformations/dates/iso_formatter.py`
- [ ] Implement `DateTransformer`:
  - ISO 8601 formatting for *DTC variables
  - Partial date handling (YYYY, YYYY-MM)
  - Timezone handling if needed
- [ ] Create `transformations/dates/study_day_calculator.py`
- [ ] Implement `StudyDayCalculator`:
  - Calculate *DY from *DTC and RFSTDTC
  - Handle dates before/after reference
  - Handle missing reference dates
- [ ] Write comprehensive unit tests
- [ ] Integration test with DomainFrameBuilder

**Acceptance Criteria:**
- Date formatting extracted and testable
- Study day calculation extracted and testable
- Same behavior as current implementation
- >95% test coverage

**Files to create:**
```
transformations/
└── dates/
    ├── __init__.py
    ├── iso_formatter.py
    └── study_day_calculator.py

tests/
└── unit/
    └── transformations/
        └── dates/
            ├── test_iso_formatter.py
            └── test_study_day_calculator.py
```

---

### DOMAIN-8: Extract Codelist Transformer
**Priority:** P2  
**Effort:** 3 hours  
**Depends on:** DOMAIN-2  
**Description:** Extract codelist mapping logic into separate transformer.

**Current location:** `xpt_module/builder.py` (mixed with other logic)

**Tasks:**
- [ ] Create `transformations/codelists/codelist_mapper.py`
- [ ] Implement `CodelistTransformer`:
  - Map *TERM to *DECOD using codelists
  - Handle missing codelists gracefully
  - Support both metadata and CT-based codelists
- [ ] Write unit tests:
  - Test successful mapping
  - Test missing codelist
  - Test unmapped terms
  - Test case-insensitive matching

**Acceptance Criteria:**
- Codelist logic extracted and testable
- Same behavior as current implementation
- Graceful fallbacks for missing data
- >90% test coverage

**Files to create:**
```
transformations/
└── codelists/
    ├── __init__.py
    └── codelist_mapper.py

tests/
└── unit/
    └── transformations/
        └── codelists/
            └── test_codelist_mapper.py
```

---

## Epic 3: Application Layer (Week 3)

### APP-1: Define Port Interfaces
**Priority:** P0 (Foundation for application layer)  
**Effort:** 3 hours  
**Depends on:** DOMAIN-1  
**Description:** Define abstract interfaces (protocols) for all external dependencies.

**Tasks:**
- [ ] Create `application/ports/repositories.py`:
  - `CTRepositoryPort` - Controlled Terminology access
  - `SDTMSpecRepositoryPort` - SDTM specification access
  - `StudyDataRepositoryPort` - Study data file access
- [ ] Create `application/ports/services.py`:
  - `LoggerPort` - Logging interface (from INFRA-6)
  - `FileGeneratorPort` - File generation interface
  - `TransformerPort` - Transformation interface (from DOMAIN-2)
- [ ] Document each interface with usage examples
- [ ] Write contract tests (tests that any implementation must pass)

**Acceptance Criteria:**
- All external dependencies have abstract interfaces
- Interfaces use Protocol (not ABC) for duck typing
- Clear documentation of contracts
- Contract tests defined

**Files to create:**
```
application/
└── ports/
    ├── __init__.py
    ├── repositories.py     # Data access interfaces
    └── services.py         # Service interfaces

tests/
└── unit/
    └── application/
        └── ports/
            ├── test_repository_contracts.py
            └── test_service_contracts.py
```

---

### APP-2: Create Study Processing Use Case
**Priority:** P1  
**Effort:** 8 hours  
**Depends on:** APP-1, INFRA-2, INFRA-3  
**Description:** Extract orchestration logic from `study_command()` into testable use case.

**Current:** `cli/commands/study.py:113-596` (483 lines)

**Target:** ~150 lines of pure orchestration, delegates all work

**Tasks:**
- [ ] Create `application/study_processing_use_case.py`
- [ ] Define `ProcessStudyRequest` dataclass (inputs)
- [ ] Define `ProcessStudyResponse` dataclass (outputs)
- [ ] Implement `StudyProcessingUseCase` class:
  - `execute(request) -> response` main method
  - Delegate to file discovery service
  - Delegate to domain processing use case (per domain)
  - Delegate to synthesis service
  - Delegate to Define-XML generator
  - Collect results and errors
- [ ] Write unit tests with mocked dependencies:
  - Test happy path (all domains succeed)
  - Test partial failure (some domains fail)
  - Test complete failure
  - Test synthesis triggering
  - Test Define-XML generation

**Acceptance Criteria:**
- Use case is pure orchestration (no business logic)
- All dependencies injected via constructor
- Testable without filesystem access
- Clear request/response DTOs
- >85% test coverage

**Files to create:**
```
application/
├── __init__.py
├── study_processing_use_case.py
└── models.py               # Request/Response DTOs

tests/
└── unit/
    └── application/
        └── test_study_processing_use_case.py
```

---

### APP-3: Create Domain Processing Use Case
**Priority:** P1  
**Effort:** 8 hours  
**Depends on:** APP-1, DOMAIN-6  
**Description:** Extract domain processing logic from `DomainProcessingCoordinator` into use case.

**Current:** `services/domain_processing_coordinator.py:55-621` (566 lines)

**Target:** ~200 lines with clear pipeline stages

**Tasks:**
- [ ] Create `application/domain_processing_use_case.py`
- [ ] Define `ProcessDomainRequest` dataclass
- [ ] Define `ProcessDomainResponse` dataclass
- [ ] Implement `DomainProcessingUseCase` class:
  - `execute(request) -> response` main method
  - Load files stage
  - Transform stage (use pipeline)
  - Map columns stage
  - Build domain dataframe stage
  - Generate outputs stage
- [ ] Write unit tests with mocked dependencies:
  - Test single file processing
  - Test multi-file merge
  - Test transformation pipeline
  - Test SUPPQUAL generation
  - Test file output generation

**Acceptance Criteria:**
- Use case has clear pipeline stages
- Each stage is testable independently
- All dependencies injected
- >85% test coverage

**Files to create:**
```
application/
└── domain_processing_use_case.py

tests/
└── unit/
    └── application/
        └── test_domain_processing_use_case.py
```

---

### APP-4: Implement Dependency Injection Container
**Priority:** P1  
**Effort:** 4 hours  
**Depends on:** APP-1, INFRA-2, INFRA-3, INFRA-6  
**Description:** Create factory for wiring up all dependencies.

**Tasks:**
- [ ] Create `infrastructure/container.py`
- [ ] Implement `DependencyContainer` class:
  - `create_csv_reader()` factory
  - `create_file_generator()` factory
  - `create_logger()` factory
  - `create_transformer_pipeline()` factory
  - `create_repositories()` factory
  - `create_study_processing_use_case()` factory
- [ ] Support configuration injection
- [ ] Write unit tests:
  - Test each factory method
  - Test configuration override
  - Test singleton vs transient

**Acceptance Criteria:**
- Single place to wire dependencies
- Easy to swap implementations
- Configuration-driven
- Testable

**Files to create:**
```
infrastructure/
└── container.py

tests/
└── unit/
    └── infrastructure/
        └── test_container.py
```

---

### APP-5: Add Integration Tests for Use Cases
**Priority:** P2  
**Effort:** 6 hours  
**Depends on:** APP-2, APP-3, APP-4  
**Description:** Create end-to-end integration tests with real dependencies.

**Tasks:**
- [ ] Create test fixtures (sample study data)
- [ ] Create `tests/integration/test_study_workflow.py`:
  - Test complete study processing
  - Test file discovery
  - Test domain processing
  - Test synthesis
  - Test Define-XML generation
- [ ] Create `tests/integration/test_domain_workflow.py`:
  - Test single domain processing
  - Test transformations
  - Test file generation
- [ ] Set up test data cleanup

**Acceptance Criteria:**
- Integration tests run end-to-end
- Tests use real file system (tmp_path)
- Tests verify output files
- Tests are repeatable and isolated

**Files to create:**
```
tests/
├── integration/
│   ├── __init__.py
│   ├── test_study_workflow.py
│   └── test_domain_workflow.py
└── fixtures/
    └── sample_study/
        ├── DM.csv
        ├── AE.csv
        └── Items.csv
```

---

## Epic 4: CLI Adapter Refactoring (Week 4)

### CLI-1: Simplify Study Command to Thin Adapter
**Priority:** P1  
**Effort:** 6 hours  
**Depends on:** APP-2, APP-4  
**Description:** Refactor `study.py` to be thin adapter that calls use cases.

**Current:** 483 lines with mixed concerns

**Target:** ~100 lines of argument parsing + use case delegation

**Tasks:**
- [ ] Refactor `cli/commands/study.py`:
  - Keep Click decorators for CLI interface
  - Parse arguments into `ProcessStudyRequest`
  - Call `StudyProcessingUseCase.execute()`
  - Format response into user output
  - Delegate all business logic to use case
- [ ] Update imports to use new paths
- [ ] Verify CLI still works: `cdisc-transpiler study ...`
- [ ] Write CLI integration tests

**Acceptance Criteria:**
- CLI interface unchanged (backward compatible)
- study.py is <150 lines
- No business logic in CLI layer
- All tests pass

**Files modified:**
```
cli/
└── commands/
    └── study.py            # MAJOR REFACTORING
```

---

### CLI-2: Extract Summary Presenter
**Priority:** P2  
**Effort:** 3 hours  
**Depends on:** CLI-1  
**Description:** Extract summary table formatting into presenter.

**Current:** Mixed with logic in `cli/helpers.py:116-279`

**Tasks:**
- [ ] Create `cli/presenters/summary.py`
- [ ] Implement `SummaryPresenter` class:
  - `present(results, errors) -> None`
  - Build Rich table
  - Format domain rows
  - Format supplemental rows
  - Format statistics
- [ ] Write unit tests (verify table structure)
- [ ] Update `study.py` to use presenter

**Acceptance Criteria:**
- Summary formatting extracted
- Testable without executing study
- Same visual output as before

**Files to create:**
```
cli/
└── presenters/
    ├── __init__.py
    └── summary.py

tests/
└── unit/
    └── cli/
        └── presenters/
            └── test_summary.py
```

---

### CLI-3: Extract Progress Presenter
**Priority:** P2  
**Effort:** 2 hours  
**Depends on:** CLI-1  
**Description:** Extract progress tracking into presenter.

**Current:** `cli/utils.py:13-54` (ProgressTracker)

**Tasks:**
- [ ] Move `cli/utils.py:ProgressTracker` → `cli/presenters/progress.py`
- [ ] Enhance with better Rich integration
- [ ] Write unit tests
- [ ] Update usage in refactored code

**Acceptance Criteria:**
- Progress tracking extracted
- Testable
- Same UX as before

**Files to create:**
```
cli/
└── presenters/
    └── progress.py

tests/
└── unit/
    └── cli/
        └── presenters/
            └── test_progress.py
```

---

### CLI-4: Add CLI Integration Tests
**Priority:** P2  
**Effort:** 4 hours  
**Depends on:** CLI-1, CLI-2, CLI-3  
**Description:** Add comprehensive CLI integration tests.

**Tasks:**
- [ ] Create `tests/integration/test_cli.py`
- [ ] Test CLI commands:
  - `cdisc-transpiler study <folder>`
  - `cdisc-transpiler study <folder> --format xpt`
  - `cdisc-transpiler study <folder> -vv`
  - Error cases (missing folder, invalid args)
- [ ] Test output files are created
- [ ] Test exit codes
- [ ] Test help text

**Acceptance Criteria:**
- CLI fully tested end-to-end
- Tests use CliRunner from Click
- Tests verify file outputs
- Tests cover error cases

**Files to create:**
```
tests/
└── integration/
    └── test_cli.py
```

---

## Epic 5: Testing & Documentation (Week 5)

### TEST-1: Create Comprehensive Unit Test Suite
**Priority:** P1  
**Effort:** 12 hours  
**Description:** Add unit tests for all new modules, targeting >80% coverage.

**Breakdown:**
- [ ] Infrastructure layer tests (from previous tickets)
- [ ] Domain layer tests (from previous tickets)
- [ ] Transformation tests (from previous tickets)
- [ ] Application layer tests (from previous tickets)
- [ ] CLI layer tests (from previous tickets)

**Additional tests needed:**
- [ ] Test error handling paths
- [ ] Test edge cases (empty data, malformed data)
- [ ] Test performance with large datasets
- [ ] Test concurrency safety (if applicable)

**Acceptance Criteria:**
- Overall test coverage >80%
- All critical paths covered
- Fast test execution (<30s for unit tests)
- Clear test names and documentation

---

### TEST-2: Create Integration Test Suite
**Priority:** P1  
**Effort:** 8 hours  
**Description:** Add end-to-end integration tests with real data.

**Tasks:**
- [ ] Create realistic test fixtures:
  - Small study (5 domains, 100 rows each)
  - Medium study (10 domains, 1000 rows each)
  - Study with variants (LBCC, LBHM)
  - Study with missing domains
- [ ] Test complete workflows:
  - End-to-end study processing
  - XPT generation
  - XML generation
  - Define-XML generation
  - SAS generation
- [ ] Test error recovery
- [ ] Test performance benchmarks

**Acceptance Criteria:**
- Integration tests cover main workflows
- Tests use real file I/O
- Tests verify output correctness
- Tests run in reasonable time (<5min)

**Files to create:**
```
tests/
├── integration/
│   ├── test_full_workflow.py
│   ├── test_xpt_generation.py
│   ├── test_xml_generation.py
│   └── test_define_generation.py
└── fixtures/
    ├── small_study/
    ├── medium_study/
    └── variant_study/
```

---

### TEST-3: Create Validation Test Suite
**Priority:** P2  
**Effort:** 6 hours  
**Description:** Add tests for SDTM compliance and file format validation.

**Tasks:**
- [ ] Create SDTM compliance tests:
  - Required variables present
  - Variable types correct
  - Variable lengths within limits
  - Controlled terminology compliance
- [ ] Create file format validation tests:
  - XPT files readable by SAS
  - XML files valid against schema
  - Define-XML valid against schema
- [ ] (Optional) Integrate Pinnacle 21 validator if available

**Acceptance Criteria:**
- Output files are SDTM compliant
- Output files pass format validation
- Tests catch common compliance issues

**Files to create:**
```
tests/
└── validation/
    ├── __init__.py
    ├── test_sdtm_compliance.py
    ├── test_xpt_format.py
    ├── test_xml_format.py
    └── test_define_xml_format.py
```

---

### DOC-1: Update README with New Architecture
**Priority:** P1  
**Effort:** 2 hours  
**Depends on:** All implementation tickets  
**Description:** Update README to reflect new architecture and usage.

**Tasks:**
- [ ] Update architecture overview section
- [ ] Update installation instructions
- [ ] Update usage examples
- [ ] Add link to detailed documentation
- [ ] Add contributing guidelines link
- [ ] Update badges (if applicable)

**Acceptance Criteria:**
- README is accurate and up-to-date
- Quick start guide works
- Links to detailed docs

---

### DOC-2: Finalize Migration Guide
**Priority:** P1  
**Effort:** 2 hours  
**Depends on:** All implementation tickets  
**Description:** Update MIGRATION.md with actual implementation details.

**Tasks:**
- [ ] Verify all breaking changes documented
- [ ] Add code migration examples for each change
- [ ] Test migration examples work
- [ ] Add FAQ entries based on implementation learnings
- [ ] Add rollback instructions

**Acceptance Criteria:**
- Migration guide is complete
- All examples tested
- Clear for users

---

### DOC-3: Create CONTRIBUTING Guide
**Priority:** P2  
**Effort:** 3 hours  
**Description:** Create guide for contributors explaining architecture.

**Tasks:**
- [ ] Create `CONTRIBUTING.md`
- [ ] Explain architectural layers
- [ ] Explain dependency injection pattern
- [ ] Provide code examples for common tasks:
  - Adding a new transformer
  - Adding a new domain
  - Adding a new output format
- [ ] Document testing requirements
- [ ] Document code style guidelines
- [ ] Add PR checklist

**Acceptance Criteria:**
- Clear contribution guidelines
- Easy for new contributors
- Examples are tested

**Files to create:**
```
CONTRIBUTING.md
```

---

### DOC-4: Add Inline Documentation
**Priority:** P2  
**Effort:** 4 hours  
**Description:** Add comprehensive docstrings to all new modules.

**Tasks:**
- [ ] Add module-level docstrings
- [ ] Add class-level docstrings
- [ ] Add method-level docstrings
- [ ] Add type hints to all functions
- [ ] Add usage examples in docstrings
- [ ] Run type checker (pyright)
- [ ] Run docstring linter (pydocstyle)

**Acceptance Criteria:**
- All public APIs documented
- Type hints complete
- Examples in docstrings
- Type checking passes

---

## Epic 6: Cleanup & Release (Week 6)

### CLEAN-1: Remove Old Code from Legacy Folder
**Priority:** P1  
**Effort:** 2 hours  
**Depends on:** All previous epics completed  
**Description:** Remove old implementations that have been replaced.

**Tasks:**
- [ ] Identify all replaced modules
- [ ] Verify all references updated to new modules
- [ ] Move old code to `legacy/` folder (for one release cycle)
- [ ] Update imports
- [ ] Run all tests to verify nothing breaks
- [ ] Plan for permanent deletion in next version

**Acceptance Criteria:**
- No references to old code
- All tests pass
- Old code in `legacy/` folder
- Deprecation warnings added

**Modules to move:**
- `services/domain_processing_coordinator.py`
- `services/domain_synthesis_coordinator.py`
- `services/study_orchestration_service.py`
- Old transformation code in `study_orchestration_service.py`

---

### CLEAN-2: Update All Import Paths
**Priority:** P1  
**Effort:** 3 hours  
**Depends on:** CLEAN-1  
**Description:** Update all remaining imports to use new paths.

**Tasks:**
- [ ] Find all imports from old paths: `grep -r "from cdisc_transpiler.services"`
- [ ] Update to new paths
- [ ] Run tests after each batch of updates
- [ ] Update examples in documentation
- [ ] Run type checker

**Acceptance Criteria:**
- No imports from old paths (except legacy)
- All tests pass
- Type checker passes

---

### CLEAN-3: Performance Benchmarking
**Priority:** P2  
**Effort:** 4 hours  
**Depends on:** CLEAN-2  
**Description:** Compare performance of new vs old implementation.

**Tasks:**
- [ ] Create `tests/benchmarks/` directory
- [ ] Implement benchmark tests:
  - Study processing time
  - Memory usage
  - File I/O performance
  - Transformation performance
- [ ] Run benchmarks on old implementation
- [ ] Run benchmarks on new implementation
- [ ] Document results
- [ ] Identify and fix any regressions

**Acceptance Criteria:**
- Performance is ≤ baseline (no significant regression)
- Benchmark results documented
- Any regressions fixed or explained

**Files to create:**
```
tests/
└── benchmarks/
    ├── __init__.py
    ├── test_study_processing_performance.py
    └── test_transformation_performance.py
```

---

### CLEAN-4: Full Validation Suite
**Priority:** P1  
**Effort:** 4 hours  
**Depends on:** CLEAN-3  
**Description:** Run full validation on sample studies to ensure quality.

**Tasks:**
- [ ] Process all sample studies in `mockdata/`
- [ ] Compare outputs with baseline (if available)
- [ ] Verify Define-XML validity
- [ ] Verify XPT file validity
- [ ] Check for any warnings or errors
- [ ] Document any behavioral changes
- [ ] Fix any critical issues found

**Acceptance Criteria:**
- All sample studies process successfully
- Output quality meets standards
- No critical issues
- All validation tests pass

---

### CLEAN-5: Prepare Release Notes
**Priority:** P1  
**Effort:** 2 hours  
**Depends on:** CLEAN-4  
**Description:** Document all changes for release.

**Tasks:**
- [ ] Create `CHANGELOG.md` for v1.0
- [ ] Document breaking changes
- [ ] Document new features
- [ ] Document bug fixes
- [ ] Document performance changes
- [ ] Link to migration guide
- [ ] Add upgrade instructions
- [ ] Thank contributors

**Acceptance Criteria:**
- Complete changelog
- Clear upgrade path
- Release notes ready

**Files to create:**
```
CHANGELOG.md
```

---

## Summary Statistics

### Effort Summary
| Epic | Tickets | Estimated Hours | Estimated Days |
|------|---------|----------------|----------------|
| 1. Infrastructure | 6 | 21 | 2.5 |
| 2. Domain Layer | 8 | 32 | 4 |
| 3. Application Layer | 5 | 29 | 3.5 |
| 4. CLI Adapter | 4 | 15 | 2 |
| 5. Testing & Docs | 8 | 37 | 4.5 |
| 6. Cleanup & Release | 5 | 15 | 2 |
| **Total** | **60** | **149** | **~6 weeks** |

**Assumptions:**
- Single developer working 5-6 hours/day on this project
- Minimal context switching and interruptions
- No major architectural pivots during implementation
- Code review feedback incorporated into estimates

**Buffer:** Consider adding 20-30% buffer for real-world factors (meetings, code review cycles, unexpected complexity, bug fixes in adjacent code).

### Priority Breakdown
- **P0 (Blockers):** 8 tickets
- **P1 (High):** 30 tickets
- **P2 (Medium):** 22 tickets

### Risk Assessment
- **Low Risk:** 40 tickets (pure refactoring, tests)
- **Medium Risk:** 15 tickets (new abstractions, transformations)
- **High Risk:** 5 tickets (major use case refactoring, CLI changes)

---

## Execution Strategy

### Recommended Order
1. **Week 1:** Complete all P0 tickets (blockers)
2. **Week 2:** Complete Domain Layer (enables transformations)
3. **Week 3:** Complete Application Layer (enables use cases)
4. **Week 4:** Complete CLI Adapter (user-facing)
5. **Week 5:** Complete Testing & Docs (quality)
6. **Week 6:** Complete Cleanup & Release (ship it)

### Daily Workflow
1. Pick next ticket in priority order
2. Write tests first (TDD)
3. Implement feature
4. Verify all tests pass
5. Commit with descriptive message
6. Update progress tracking

### Definition of Done (per ticket)
- [ ] Implementation complete
- [ ] Unit tests written and passing
- [ ] Integration tests updated (if applicable)
- [ ] Documentation updated
- [ ] Code reviewed (self-review)
- [ ] No type errors (pyright)
- [ ] Committed and pushed

---

**Document Status:** Ready for Implementation  
**Last Updated:** 2025-12-14  
**Next Action:** Start with INFRA-1
