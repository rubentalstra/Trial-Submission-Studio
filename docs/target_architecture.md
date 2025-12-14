# Target Architecture & Refactoring Plan

**Date:** 2025-12-14  
**Repository:** rubentalstra/cdisc-transpiler  
**Status:** Design Proposal (Breaking Changes Allowed)

---

## Executive Summary

This document proposes a comprehensive refactoring of the CDISC Transpiler codebase to eliminate duplication, improve maintainability, and establish clear architectural boundaries. **Breaking changes are acceptable** - backward compatibility is not required.

### Key Goals
1. **Reduce complexity**: Break down 500+ line functions into composable units
2. **Eliminate duplication**: Establish single sources of truth for common patterns
3. **Improve testability**: Enable unit testing through dependency injection
4. **Clarify responsibilities**: Enforce clear module boundaries (domain/services/adapters)
5. **Enable extensibility**: Make it easy to add new domains and transformations

### Scope
- Full refactoring of study command workflow (study.py)
- Consolidation of 3 service coordinators into unified pipeline
- Extraction of 60% duplicate code in transformations
- Creation of comprehensive test suite
- Centralized configuration management

---

## 1. Target Architecture

### 1.1 Architectural Layers

**Why Ports & Adapters (Hexagonal Architecture)?**

We chose this pattern over alternatives (layered, clean architecture) because:
- **Testability**: Business logic isolated from infrastructure (no mocks for core domain)
- **Flexibility**: Easy to swap implementations (console logger → file logger)
- **Clear boundaries**: Explicit interfaces prevent accidental coupling
- **Domain focus**: Core logic doesn't depend on frameworks or CLI
- **Better than layered**: Avoids database-centric design (we're file-based)
- **Better than clean architecture**: Less ceremony, clearer for small team

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Layer (Adapters)                      │
│  - Argument parsing & validation                            │
│  - User interaction & progress display                      │
│  - Error presentation                                       │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                Application Layer (Use Cases)                 │
│  - StudyProcessingUseCase: Main workflow orchestration      │
│  - DomainProcessingUseCase: Single domain processing        │
│  - File generation orchestration                            │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                   Domain Layer (Core)                        │
│  - Domain entities: SDTMDomain, Variable, StudyMetadata     │
│  - Business rules: Validation, transformation rules         │
│  - Interfaces: Repositories, services (abstract)            │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│              Infrastructure Layer (Adapters)                 │
│  - File I/O: CSV readers, XPT/XML writers                   │
│  - External data: CDISC CT loader, SDTM spec loader         │
│  - Logging, monitoring                                      │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Proposed Folder Structure

```
cdisc_transpiler/
├── __init__.py
├── config.py                          # NEW: Centralized configuration
├── constants.py                       # NEW: Magic values and defaults
│
├── cli/                               # Adapter layer
│   ├── __init__.py
│   ├── __main__.py
│   ├── commands/
│   │   ├── __init__.py
│   │   ├── study.py                  # SIMPLIFIED: Thin adapter only
│   │   └── domains.py
│   ├── presenters/                   # NEW: Output formatting
│   │   ├── __init__.py
│   │   ├── progress.py
│   │   └── summary.py
│   └── validation.py                 # NEW: CLI argument validation
│
├── application/                       # NEW: Use case layer
│   ├── __init__.py
│   ├── study_processing_use_case.py  # Main workflow
│   ├── domain_processing_use_case.py # Single domain workflow
│   └── ports/                        # Interface definitions
│       ├── __init__.py
│       ├── repositories.py           # Abstract repositories
│       └── services.py               # Abstract services
│
├── domain/                           # Core domain logic
│   ├── __init__.py
│   ├── entities/                     # Domain entities
│   │   ├── __init__.py
│   │   ├── sdtm_domain.py           # Renamed from domains_module
│   │   ├── variable.py
│   │   ├── study_metadata.py
│   │   └── mapping.py
│   ├── services/                     # Domain services
│   │   ├── __init__.py
│   │   ├── transformation_service.py
│   │   ├── mapping_service.py
│   │   └── validation_service.py
│   └── specifications/               # Business rules
│       ├── __init__.py
│       ├── sdtm_rules.py
│       └── ct_rules.py
│
├── infrastructure/                   # NEW: Infrastructure adapters
│   ├── __init__.py
│   ├── io/                          # File I/O (consolidated)
│   │   ├── __init__.py
│   │   ├── csv_reader.py           # Single source for CSV reading
│   │   ├── xpt_writer.py
│   │   ├── xml_writer.py
│   │   └── sas_writer.py
│   ├── repositories/                # Data access implementations
│   │   ├── __init__.py
│   │   ├── cdisc_ct_repository.py
│   │   ├── sdtm_spec_repository.py
│   │   └── study_data_repository.py
│   ├── logging/                     # Logging infrastructure
│   │   ├── __init__.py
│   │   ├── logger.py
│   │   └── console_handler.py
│   └── caching/                     # NEW: Explicit caching layer
│       ├── __init__.py
│       └── memory_cache.py
│
├── transformations/                  # NEW: Transformation framework
│   ├── __init__.py
│   ├── base.py                      # Base transformer interface
│   ├── findings/                    # Findings class transformers
│   │   ├── __init__.py
│   │   ├── wide_to_long.py         # Generic wide-to-long
│   │   ├── vs_transformer.py       # VS-specific logic
│   │   └── lb_transformer.py       # LB-specific logic
│   ├── dates/
│   │   ├── __init__.py
│   │   ├── iso_formatter.py
│   │   └── study_day_calculator.py
│   └── codelists/
│       ├── __init__.py
│       └── codelist_mapper.py
│
├── synthesis/                        # Domain synthesis (simplified)
│   ├── __init__.py
│   ├── base.py                      # Base synthesizer
│   ├── trial_design.py              # TS, TA, TE, SE, DS
│   ├── observations.py              # AE, LB, VS, EX
│   └── relationships.py             # RELREC
│
└── legacy/                          # TEMPORARY: Old code during migration
    └── (old modules moved here during refactoring)
```

### 1.3 Key Design Principles

#### 1. Dependency Injection
**Before:**
```python
from ..cli.logging_config import get_logger

class DomainProcessor:
    def process(self):
        logger = get_logger()  # Global state access
        logger.info("Processing...")
```

**After:**
```python
from application.ports.services import LoggerPort

class DomainProcessor:
    def __init__(self, logger: LoggerPort):
        self._logger = logger  # Injected dependency
    
    def process(self):
        self._logger.info("Processing...")
```

#### 2. Single Responsibility
**Before:** `study_command()` - 483 lines, 10 responsibilities

**After:** Decomposed into:
```python
class StudyProcessingUseCase:
    def execute(self, request: ProcessStudyRequest) -> ProcessStudyResponse:
        # Orchestration only, delegates to:
        # - FileDiscoveryService
        # - DomainProcessingUseCase (per domain)
        # - SynthesisService
        # - DefineXMLGenerator
```

#### 3. Interface Segregation
```python
# application/ports/repositories.py
class CTRepositoryPort(Protocol):
    """Interface for controlled terminology access."""
    def get_test_codes(self, domain: str) -> list[str]: ...
    def normalize_test_code(self, domain: str, code: str) -> str: ...

class SDTMSpecRepositoryPort(Protocol):
    """Interface for SDTM specification access."""
    def get_domain(self, code: str) -> SDTMDomain: ...
    def list_domains(self) -> list[str]: ...
```

#### 4. Explicit Configuration
```python
# config.py
@dataclass(frozen=True)
class TranspilerConfig:
    """Immutable configuration for the transpiler."""
    
    # Paths (configurable via env vars)
    sdtm_spec_dir: Path = field(default_factory=lambda: Path("docs/SDTMIG_v3.4"))
    ct_dir: Path = field(default_factory=lambda: Path("docs/Controlled_Terminology"))
    
    # Processing defaults
    min_confidence: float = 0.5
    chunk_size: int = 1000
    
    # Defaults for synthesis
    default_date: str = "2023-01-01"
    default_subject: str = "SYNTH001"
    
    # XPT constraints
    xpt_max_label_length: int = 200
    xpt_max_variables: int = 40
    
    @classmethod
    def from_env(cls) -> "TranspilerConfig":
        """Load configuration from environment variables."""
        return cls(
            sdtm_spec_dir=Path(os.getenv("SDTM_SPEC_DIR", "docs/SDTMIG_v3.4")),
            # ... other env-based overrides
        )
```

---

## 2. Single Source of Truth Components

### 2.1 CSV Reading (io/csv_reader.py)
**Replaces:** 3 different CSV reading patterns

```python
@dataclass
class CSVReadOptions:
    """Options for CSV reading."""
    normalize_headers: bool = True
    strict_na_handling: bool = True
    dtype: str | dict | None = str
    encoding: str = "utf-8"

class CSVReader:
    """Unified CSV reader with consistent behavior."""
    
    def read(
        self,
        path: Path,
        options: CSVReadOptions | None = None,
    ) -> pd.DataFrame:
        """Read CSV with standard normalization and error handling."""
        options = options or CSVReadOptions()
        
        try:
            df = pd.read_csv(
                path,
                dtype=options.dtype,
                keep_default_na=False if options.strict_na_handling else True,
                na_values=[""] if options.strict_na_handling else None,
                encoding=options.encoding,
            )
        except FileNotFoundError as e:
            raise DataSourceNotFoundError(f"File not found: {path}") from e
        except pd.errors.ParserError as e:
            raise DataParseError(f"Failed to parse {path}: {e}") from e
        
        if options.normalize_headers:
            df.columns = [col.strip() for col in df.columns]
        
        return df
```

### 2.2 File Generation (io/file_generator.py)
**Replaces:** 3 copies of file generation logic

```python
@dataclass
class OutputRequest:
    """Request for file generation."""
    dataframe: pd.DataFrame
    domain_code: str
    config: MappingConfig
    output_dirs: OutputDirs
    formats: set[str]  # {"xpt", "xml", "sas"}

@dataclass
class OutputResult:
    """Result of file generation."""
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    errors: list[str] = field(default_factory=list)

class FileGenerator:
    """Centralized file generation for all formats."""
    
    def __init__(
        self,
        xpt_writer: XPTWriter,
        xml_writer: XMLWriter,
        sas_writer: SASWriter,
        logger: LoggerPort,
    ):
        self._xpt_writer = xpt_writer
        self._xml_writer = xml_writer
        self._sas_writer = sas_writer
        self._logger = logger
    
    def generate(self, request: OutputRequest) -> OutputResult:
        """Generate all requested output files."""
        result = OutputResult()
        
        if "xpt" in request.formats and request.output_dirs.xpt_dir:
            result.xpt_path = self._generate_xpt(request)
        
        if "xml" in request.formats and request.output_dirs.xml_dir:
            result.xml_path = self._generate_xml(request)
        
        if "sas" in request.formats and request.output_dirs.sas_dir:
            result.sas_path = self._generate_sas(request)
        
        return result
```

### 2.3 Transformation Framework (transformations/base.py)
**Replaces:** Duplicate VS/LB transformation logic

```python
class TransformerPort(Protocol):
    """Interface for data transformers."""
    
    def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
        """Check if this transformer applies to the data."""
        ...
    
    def transform(
        self,
        df: pd.DataFrame,
        context: TransformationContext,
    ) -> pd.DataFrame:
        """Apply transformation and return result."""
        ...

class TransformationPipeline:
    """Pipeline for applying transformations in order."""
    
    def __init__(self, transformers: list[TransformerPort]):
        self._transformers = transformers
    
    def execute(
        self,
        df: pd.DataFrame,
        context: TransformationContext,
    ) -> pd.DataFrame:
        """Apply all applicable transformers in sequence."""
        result = df
        for transformer in self._transformers:
            if transformer.can_transform(result, context.domain_code):
                result = transformer.transform(result, context)
        return result
```

### 2.4 Wide-to-Long Transformer (transformations/findings/wide_to_long.py)
**Replaces:** Duplicate VS and LB reshaping logic

```python
@dataclass
class TestColumnPattern:
    """Pattern for matching test-related columns."""
    orres: str  # Result column pattern (e.g., "ORRES_{test}")
    orresu: str | None = None  # Unit column pattern
    test: str | None = None  # Test label pattern
    nrlo: str | None = None  # Normal range low
    nrhi: str | None = None  # Normal range high

class WideToLongTransformer:
    """Generic wide-to-long transformer for Findings domains."""
    
    def __init__(
        self,
        domain_code: str,
        ct_repository: CTRepositoryPort,
        patterns: list[TestColumnPattern],
    ):
        self._domain_code = domain_code
        self._ct_repository = ct_repository
        self._patterns = patterns
    
    def transform(
        self,
        df: pd.DataFrame,
        context: TransformationContext,
    ) -> pd.DataFrame:
        """Transform wide format to SDTM long format."""
        # Common implementation for VS, LB, QS, etc.
        test_defs = self._discover_tests(df)
        records = []
        
        for _, row in df.iterrows():
            for test_code, test_def in test_defs.items():
                # Normalize test code using CT
                std_code = self._ct_repository.normalize_test_code(
                    self._domain_code, test_code
                )
                if not std_code:
                    continue
                
                record = self._build_record(row, test_code, test_def, std_code)
                if record:
                    records.append(record)
        
        return pd.DataFrame(records) if records else pd.DataFrame()
```

### 2.5 Configuration Management (config.py)
**Replaces:** Scattered hardcoded values

```python
class ConfigLoader:
    """Load configuration from multiple sources."""
    
    @staticmethod
    def load() -> TranspilerConfig:
        """Load configuration with precedence: CLI > Env > Defaults."""
        # 1. Start with defaults
        config = TranspilerConfig()
        
        # 2. Override from environment variables
        config = TranspilerConfig.from_env()
        
        # 3. Override from config file if exists
        config_file = Path("cdisc_transpiler.toml")
        if config_file.exists():
            config = ConfigLoader._load_from_toml(config_file, config)
        
        return config
```

---

## 3. Breaking Changes & Migration

### 3.1 Breaking Changes List

#### API Changes

| Old | New | Breaking? |
|-----|-----|-----------|
| `from cdisc_transpiler.domains_module import get_domain` | `from cdisc_transpiler.domain.entities import get_domain` | Yes - Import path |
| `from cdisc_transpiler.io_module import load_input_dataset` | `from cdisc_transpiler.infrastructure.io import CSVReader` | Yes - API change |
| `from cdisc_transpiler.cli.logging_config import get_logger` | Inject `LoggerPort` via constructor | Yes - Pattern change |
| `MappingConfig` dataclass | `MappingConfig` frozen dataclass | Maybe - If code mutates it |
| Study metadata in study folder | Study metadata in study folder | No |

#### CLI Changes (None - CLI interface stays the same)

```bash
# All existing CLI commands work unchanged
cdisc-transpiler study <folder> [OPTIONS]  # ✓ Works
```

#### Configuration Changes

| Old | New | Migration |
|-----|-----|-----------|
| Hardcoded `docs/SDTMIG_v3.4` | `SDTM_SPEC_DIR` env var (with default) | Optional - only if custom paths needed |
| Hardcoded `0.5` min confidence | `cdisc_transpiler.toml` config file | Optional - defaults remain |
| No config file | Optional `cdisc_transpiler.toml` | Backwards compatible |

#### Internal API Changes (for library users)

```python
# OLD (direct imports)
from cdisc_transpiler.services import DomainProcessingCoordinator
coordinator = DomainProcessingCoordinator()

# NEW (dependency injection)
from cdisc_transpiler.application import DomainProcessingUseCase
from cdisc_transpiler.infrastructure import create_default_dependencies

deps = create_default_dependencies()
use_case = DomainProcessingUseCase(
    logger=deps.logger,
    file_generator=deps.file_generator,
    transformer=deps.transformer,
)
```

### 3.2 Migration Steps

#### Phase 1: Infrastructure Setup (Week 1)
1. Create new folder structure (without deleting old code)
2. Implement new `infrastructure/io/` modules
   - CSVReader (replace 3 implementations)
   - FileGenerator (consolidate file writing)
3. Implement `config.py` and `constants.py`
4. Add unit tests for new infrastructure

#### Phase 2: Core Domain Extraction (Week 2)
1. Move domain entities to `domain/entities/`
   - No logic changes, just reorganization
2. Extract transformation framework
   - Create `transformations/base.py`
   - Implement `WideToLongTransformer`
3. Refactor VS and LB transformers to use common framework
4. Add unit tests for transformations

#### Phase 3: Application Layer (Week 3)
1. Create use case classes
   - `StudyProcessingUseCase`
   - `DomainProcessingUseCase`
2. Define ports (interfaces)
   - `LoggerPort`, `RepositoryPort`, etc.
3. Implement dependency injection container
4. Add integration tests

#### Phase 4: CLI Adapter Simplification (Week 4)
1. Refactor `study.py` to be thin adapter
   - Call use cases instead of direct service calls
2. Extract presenters for output formatting
3. Add CLI integration tests with mock data

#### Phase 5: Testing & Validation (Week 5)
1. Create comprehensive test suite
   - Unit tests: >80% coverage target
   - Integration tests: End-to-end workflows
   - Validation tests: SDTM compliance checks
2. Test with real study data
3. Performance benchmarking

#### Phase 6: Cleanup (Week 6)
1. Remove old code from `legacy/` folder
2. Update documentation
3. Create migration guide for library users

### 3.3 Testing Strategy

#### Unit Tests
```python
# tests/unit/infrastructure/io/test_csv_reader.py
def test_csv_reader_normalizes_headers():
    """Verify CSV reader normalizes column headers."""
    reader = CSVReader()
    df = reader.read(fixture_path("messy_headers.csv"))
    
    assert "USUBJID" in df.columns
    assert "usubjid" not in df.columns
    assert " USUBJID " not in df.columns

# tests/unit/transformations/test_wide_to_long.py
def test_wide_to_long_vs_transformation():
    """Verify VS wide-to-long transformation."""
    # Given: Wide format VS data
    input_df = pd.DataFrame({
        "USUBJID": ["001"],
        "ORRES_HR": [72],
        "ORRES_SYSBP": [120],
    })
    
    # When: Transform
    transformer = VSTransformer(ct_repository=mock_ct_repo)
    result = transformer.transform(input_df, context)
    
    # Then: Long format with 2 records
    assert len(result) == 2
    assert "VSTESTCD" in result.columns
    assert set(result["VSTESTCD"]) == {"HR", "SYSBP"}
```

#### Integration Tests
```python
# tests/integration/test_study_workflow.py
def test_end_to_end_study_processing(tmp_path):
    """Verify complete study processing workflow."""
    # Given: Sample study folder
    study_folder = setup_fixture_study(tmp_path)
    
    # When: Process study
    use_case = StudyProcessingUseCase(dependencies)
    result = use_case.execute(ProcessStudyRequest(
        study_folder=study_folder,
        output_dir=tmp_path / "output",
    ))
    
    # Then: All expected files generated
    assert result.success
    assert (tmp_path / "output" / "xpt" / "dm.xpt").exists()
    assert (tmp_path / "output" / "define.xml").exists()
```

#### Validation Tests
```python
# tests/validation/test_sdtm_compliance.py
def test_generated_xpt_passes_pinnacle21():
    """Verify generated XPT files pass Pinnacle 21 validation."""
    # This would call Pinnacle 21 validator via subprocess
    # or use their API if available
```

### 3.4 Rollback Plan

Since this is a greenfield refactor in a feature branch:
1. **Risk:** Low - no production users to impact
2. **Rollback:** Simply don't merge the refactor branch
3. **Partial adoption:** Cherry-pick specific improvements to main

---

## 4. Implementation Tickets

### Epic 1: Infrastructure Layer
- [ ] **INFRA-1:** Create new folder structure and move docs
- [ ] **INFRA-2:** Implement `CSVReader` with unit tests (replace 3 implementations)
- [ ] **INFRA-3:** Implement `FileGenerator` with unit tests (consolidate file writing)
- [ ] **INFRA-4:** Create `TranspilerConfig` dataclass and `ConfigLoader`
- [ ] **INFRA-5:** Extract constants to `constants.py` module
- [ ] **INFRA-6:** Implement `LoggerPort` interface and adapter

### Epic 2: Domain Layer
- [ ] **DOMAIN-1:** Move domain entities to `domain/entities/` (no logic change)
- [ ] **DOMAIN-2:** Create `TransformerPort` interface
- [ ] **DOMAIN-3:** Implement `WideToLongTransformer` base class
- [ ] **DOMAIN-4:** Refactor VS transformer to use base (remove duplication)
- [ ] **DOMAIN-5:** Refactor LB transformer to use base (remove duplication)
- [ ] **DOMAIN-6:** Create transformation pipeline with unit tests
- [ ] **DOMAIN-7:** Extract date transformation logic to `DateTransformer`
- [ ] **DOMAIN-8:** Extract codelist logic to `CodelistTransformer`

### Epic 3: Application Layer
- [ ] **APP-1:** Define port interfaces (`LoggerPort`, `RepositoryPort`, etc.)
- [ ] **APP-2:** Create `StudyProcessingUseCase` (orchestration only)
- [ ] **APP-3:** Create `DomainProcessingUseCase` (single domain workflow)
- [ ] **APP-4:** Implement dependency injection container
- [ ] **APP-5:** Add integration tests for use cases

### Epic 4: CLI Adapter Refactoring
- [ ] **CLI-1:** Simplify `study.py` to call use cases (remove business logic)
- [ ] **CLI-2:** Extract presenters for summary display
- [ ] **CLI-3:** Extract presenters for progress tracking
- [ ] **CLI-4:** Add CLI integration tests with fixtures

### Epic 5: Testing & Documentation
- [ ] **TEST-1:** Create unit test suite (target >80% coverage)
  - [ ] Infrastructure layer tests
  - [ ] Domain layer tests
  - [ ] Transformation tests
- [ ] **TEST-2:** Create integration test suite
  - [ ] End-to-end workflow tests
  - [ ] Error handling tests
- [ ] **TEST-3:** Create validation test suite
  - [ ] SDTM compliance tests
  - [ ] File format validation tests
- [ ] **DOC-1:** Update README with new architecture
- [ ] **DOC-2:** Create MIGRATION.md guide
- [ ] **DOC-3:** Create CONTRIBUTING.md with architecture explanation
- [ ] **DOC-4:** Add inline documentation to new modules

### Epic 6: Cleanup & Release
- [ ] **CLEAN-1:** Remove old code from `legacy/` folder
- [ ] **CLEAN-2:** Update all import paths in remaining code
- [ ] **CLEAN-3:** Performance benchmarking vs old implementation
- [ ] **CLEAN-4:** Run full validation suite on sample studies
- [ ] **CLEAN-5:** Prepare release notes documenting breaking changes

---

## 5. Success Metrics

### Code Quality Metrics
| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Avg function length | ~150 lines* | <50 lines | CodeClimate |
| Cyclomatic complexity | Max 40* | Max 10 | Radon |
| Test coverage | 0%* | >80% | pytest-cov |
| Duplicate code | ~30%* | <5% | pylint |
| Import coupling | High* | Low | Dependency graph analysis |

*Note: Current values are estimates from manual code review. Before implementation, run baseline measurements using the tools listed to establish precise starting points.

### Performance Metrics
| Metric | Baseline | Target | Test Case |
|--------|----------|--------|-----------|
| Study processing time | TBD | ≤ baseline | DEMO_GDISC study |
| Memory usage | TBD | ≤ baseline | Large study (1M rows) |
| Startup time | TBD | ≤ baseline | `cdisc-transpiler --help` |

### Maintainability Metrics
| Metric | Current | Target |
|--------|---------|--------|
| Time to add new domain | ~4 hours | <1 hour |
| Time to add new transformation | ~2 hours | <30 min |
| Time to fix common bug | ~1 hour | <15 min |

---

## 6. Risk Assessment

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Performance regression | Medium | High | Benchmark early and often; optimize hot paths |
| Breaking existing integrations | Low | High | No known external users; document all breaking changes |
| Incomplete refactoring | Medium | Medium | Incremental approach; working code at each phase |
| Test coverage gaps | Medium | Medium | TDD approach; require tests for new code |
| Over-engineering | Low | Medium | Keep it simple; avoid premature abstractions |

### Schedule Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Underestimated complexity | Medium | Medium | Buffer time in estimates; prioritize must-haves |
| Scope creep | Low | Low | Strict ticket-based approach; defer nice-to-haves |
| Resource availability | Low | Low | Single developer; predictable schedule |

---

## 7. Decision Log

### ADR-001: Use Ports & Adapters (Hexagonal) Architecture
**Status:** Accepted  
**Context:** Need clear boundaries between business logic and infrastructure  
**Decision:** Adopt ports & adapters pattern with explicit interfaces  
**Consequences:** 
- ✅ Better testability (can mock adapters)
- ✅ Clear dependency flow
- ⚠️ More boilerplate for interfaces

### ADR-002: Allow Breaking Changes
**Status:** Accepted  
**Context:** No known external users; codebase needs significant restructuring  
**Decision:** Breaking changes are acceptable if they improve design  
**Consequences:**
- ✅ Freedom to make optimal design decisions
- ✅ No backward compatibility burden
- ⚠️ Must document all breaking changes

### ADR-003: Extract Transformation Framework
**Status:** Accepted  
**Context:** 60% code duplication between VS and LB transformations  
**Decision:** Create generic `WideToLongTransformer` base class  
**Consequences:**
- ✅ Eliminates duplication
- ✅ Easy to add new Findings domains
- ⚠️ Requires careful abstraction design

### ADR-004: Dependency Injection over Singletons
**Status:** Accepted  
**Context:** Global logger and registry singletons cause tight coupling  
**Decision:** Use constructor injection for dependencies  
**Consequences:**
- ✅ Testable without global state
- ✅ Explicit dependency graph
- ⚠️ More verbose initialization code

### ADR-005: Incremental Migration Strategy
**Status:** Accepted  
**Context:** Large refactoring; need to maintain working code  
**Decision:** Refactor in phases; keep old code in `legacy/` during migration  
**Consequences:**
- ✅ Always have working code
- ✅ Can merge incrementally
- ⚠️ Temporary code duplication

---

## 8. Open Questions

1. **Q:** Should we support streaming mode for large datasets?
   **A:** Defer to Phase 2; focus on correctness first

2. **Q:** How to handle domain-specific validation rules?
   **A:** Create `ValidationRuleRegistry` with pluggable rules per domain

3. **Q:** Should we support custom transformations via plugins?
   **A:** Not in v1.0; document extension points for future

4. **Q:** How to handle CDISC CT updates?
   **A:** Make CT directory configurable; document update process

5. **Q:** Should we cache transformed data between runs?
   **A:** Not in v1.0; focus on correctness and maintainability first

---

## Appendix A: Comparison Matrix

### Before vs After: Study Command

| Aspect | Before (study.py) | After (StudyProcessingUseCase) |
|--------|------------------|-------------------------------|
| Lines of code | 483 | ~150 (orchestration only) |
| Responsibilities | 10+ | 1 (orchestrate workflow) |
| Direct imports | 10 modules | 3 ports (injected) |
| Testability | Low (global state) | High (mocked dependencies) |
| Error handling | Mixed with logic | Centralized in use case |

### Before vs After: Transformations

| Aspect | Before (VS + LB) | After (WideToLongTransformer) |
|--------|-----------------|-------------------------------|
| Lines of code | 351 (combined) | ~180 (shared base) |
| Duplication | ~60% | <5% |
| Extensibility | Copy-paste | Inherit from base |
| Test coverage | 0% | >80% |

---

**Document Version:** 1.0  
**Status:** Ready for Implementation  
**Approval Required:** Repository Owner

**Next Steps:**
1. Review and approve this design document
2. Create GitHub issues from ticket list
3. Start with Epic 1 (Infrastructure Layer)
4. Establish test-first workflow
