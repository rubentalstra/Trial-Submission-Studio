# Migration Guide: v0.x → v1.0 Refactored Architecture

**Last Updated:** 2025-12-14  
**Target Version:** 1.0.0  
**Breaking Changes:** Yes

---

## Overview

This guide documents the breaking changes introduced in the v1.0 refactoring and provides migration paths for code that depends on the CDISC Transpiler.

### Who Should Read This?

- **CLI Users**: Good news! The CLI interface is **unchanged**. No migration needed.
- **Library Users**: If you import modules directly from `cdisc_transpiler`, you'll need to update import paths.
- **Contributors**: If you're developing features or fixing bugs, read the new architecture documentation.

---

## CLI Users: No Changes Required ✅

If you only use the command-line interface, **no changes are required**. All existing commands work identically:

```bash
# All these commands work unchanged
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/ --format xpt
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/ -vv
cdisc-transpiler domains
```

### Optional: New Configuration File

You can now optionally create a `cdisc_transpiler.toml` file in your working directory to set defaults:

```toml
# cdisc_transpiler.toml (optional)
[default]
min_confidence = 0.7
output_format = "xpt"
generate_define = true
generate_sas = true

[paths]
sdtm_spec_dir = "docs/SDTMIG_v3.4"
ct_dir = "docs/Controlled_Terminology"
```

This is **completely optional** and backward compatible.

---

## Library Users: Import Path Changes

If you're using `cdisc_transpiler` as a library in your own Python code, you'll need to update import statements.

### Module Reorganization

#### domains_module → domain.entities

**Old:**
```python
from cdisc_transpiler.domains_module import get_domain, list_domains, SDTMDomain
from cdisc_transpiler.domains_module.models import SDTMVariable
```

**New:**
```python
from cdisc_transpiler.domain.entities import get_domain, list_domains, SDTMDomain
from cdisc_transpiler.domain.entities import SDTMVariable
```

#### io_module → infrastructure.io

**Old:**
```python
from cdisc_transpiler.io_module import load_input_dataset, ParseError
```

**New:**
```python
from cdisc_transpiler.infrastructure.io import CSVReader, CSVReadOptions
from cdisc_transpiler.infrastructure.io import DataParseError

# Usage changed:
reader = CSVReader()
df = reader.read(path, options=CSVReadOptions(normalize_headers=True))
```

#### metadata_module → domain.entities

**Old:**
```python
from cdisc_transpiler.metadata_module import load_study_metadata, StudyMetadata
```

**New:**
```python
from cdisc_transpiler.domain.entities import StudyMetadata
from cdisc_transpiler.infrastructure.repositories import StudyMetadataRepository

# Usage changed:
repo = StudyMetadataRepository()
metadata = repo.load(study_folder)
```

#### services → application

**Old:**
```python
from cdisc_transpiler.services import (
    DomainProcessingCoordinator,
    DomainSynthesisCoordinator,
    StudyOrchestrationService,
)

coordinator = DomainProcessingCoordinator()
result = coordinator.process_and_merge_domain(...)
```

**New:**
```python
from cdisc_transpiler.application import DomainProcessingUseCase
from cdisc_transpiler.infrastructure import create_default_dependencies

# Dependency injection pattern:
deps = create_default_dependencies()
use_case = DomainProcessingUseCase(
    logger=deps.logger,
    file_generator=deps.file_generator,
    csv_reader=deps.csv_reader,
    transformer_pipeline=deps.transformer_pipeline,
)

result = use_case.execute(request)
```

#### logging → infrastructure.logging

**Old:**
```python
from cdisc_transpiler.cli.logging_config import get_logger, create_logger

logger = get_logger()  # Global singleton
logger.info("Processing...")
```

**New:**
```python
from cdisc_transpiler.infrastructure.logging import Logger, ConsoleLogger

# Inject logger as dependency:
logger = ConsoleLogger(verbosity=1)

class MyService:
    def __init__(self, logger: Logger):
        self._logger = logger
    
    def process(self):
        self._logger.info("Processing...")
```

### API Signature Changes

#### File Writing

**Old:**
```python
from cdisc_transpiler.xpt_module import write_xpt_file
from cdisc_transpiler.xml_module.dataset_module import write_dataset_xml

write_xpt_file(dataframe, domain_code, output_path)
write_dataset_xml(dataframe, domain_code, config, output_path)
```

**New:**
```python
from cdisc_transpiler.infrastructure.io import FileGenerator, OutputRequest

generator = FileGenerator(logger=logger)
result = generator.generate(OutputRequest(
    dataframe=dataframe,
    domain_code=domain_code,
    config=config,
    output_dirs=OutputDirs(xpt_dir=xpt_dir, xml_dir=xml_dir),
    formats={"xpt", "xml"},
))

# Result contains paths to all generated files
print(result.xpt_path)
print(result.xml_path)
```

#### Transformations

**Old:**
```python
from cdisc_transpiler.services import StudyOrchestrationService

service = StudyOrchestrationService()
vs_long = service.reshape_vs_to_long(vs_wide, study_id)
lb_long = service.reshape_lb_to_long(lb_wide, study_id)
```

**New:**
```python
from cdisc_transpiler.transformations.findings import VSTransformer, LBTransformer
from cdisc_transpiler.infrastructure.repositories import CDISCCTRepository

ct_repo = CDISCCTRepository()
vs_transformer = VSTransformer(ct_repository=ct_repo)
lb_transformer = LBTransformer(ct_repository=ct_repo)

context = TransformationContext(study_id=study_id, domain_code="VS")
vs_long = vs_transformer.transform(vs_wide, context)

context = TransformationContext(study_id=study_id, domain_code="LB")
lb_long = lb_transformer.transform(lb_wide, context)
```

---

## Configuration Changes

### Hardcoded Paths Now Configurable

**Old:** Paths were hardcoded in loaders
```python
# In code:
domain_dir = Path("docs/SDTMIG_v3.4")  # Hardcoded
ct_dir = Path("docs/Controlled_Terminology")  # Hardcoded
```

**New:** Paths are configurable via environment variables or config file

```bash
# Environment variables:
export SDTM_SPEC_DIR=/custom/path/to/sdtm
export CT_DIR=/custom/path/to/ct

cdisc-transpiler study mockdata/...
```

Or via `cdisc_transpiler.toml`:
```toml
[paths]
sdtm_spec_dir = "/custom/path/to/sdtm"
ct_dir = "/custom/path/to/ct"
```

Or programmatically:
```python
from cdisc_transpiler.config import TranspilerConfig

config = TranspilerConfig(
    sdtm_spec_dir=Path("/custom/path/to/sdtm"),
    ct_dir=Path("/custom/path/to/ct"),
)
```

### Default Values Now Centralized

**Old:** Magic values scattered throughout code

**New:** Defaults in `constants.py`
```python
from cdisc_transpiler.constants import Defaults, Constraints

# Instead of hardcoded "2023-01-01":
default_date = Defaults.DATE

# Instead of hardcoded "0.5":
min_confidence = Defaults.MIN_CONFIDENCE

# Instead of hardcoded "200":
max_label_length = Constraints.XPT_MAX_LABEL_LENGTH
```

---

## Error Handling Changes

### New Exception Hierarchy

**Old:** Mix of built-in exceptions and custom exceptions

**New:** Consistent exception hierarchy

```python
from cdisc_transpiler.domain.exceptions import (
    TranspilerError,          # Base exception
    DataSourceError,          # File I/O errors
    DataParseError,          # Parsing errors
    ValidationError,         # SDTM validation errors
    TransformationError,     # Transformation errors
    ConfigurationError,      # Configuration errors
)

# Usage:
try:
    df = csv_reader.read(path)
except DataSourceError as e:
    logger.error(f"Failed to read file: {e}")
except DataParseError as e:
    logger.error(f"Failed to parse CSV: {e}")
```

---

## Testing Changes

### New Test Structure

**Old:** No tests in repository

**New:** Comprehensive test suite

```
tests/
├── unit/
│   ├── test_csv_reader.py
│   ├── test_transformers.py
│   └── test_mapping_engine.py
├── integration/
│   └── test_study_workflow.py
└── fixtures/
    └── sample_study/
```

### Testing Your Integration

If you're using `cdisc_transpiler` as a library, you can now write better tests:

**Old:** Hard to test due to global state
```python
# Had to test against real files and global logger
def test_my_integration():
    result = process_domain(real_file_path)
    assert result  # Hard to verify behavior
```

**New:** Easy to test with mocks
```python
from unittest.mock import Mock

def test_my_integration():
    # Mock dependencies
    mock_logger = Mock()
    mock_file_generator = Mock()
    
    use_case = DomainProcessingUseCase(
        logger=mock_logger,
        file_generator=mock_file_generator,
    )
    
    result = use_case.execute(request)
    
    # Verify behavior
    assert result.success
    mock_logger.info.assert_called()
    mock_file_generator.generate.assert_called_once()
```

---

## Performance Changes

### Expected Performance Impact

| Operation | Before | After | Change |
|-----------|--------|-------|--------|
| Study processing | Baseline | ~Same | No significant change expected |
| Memory usage | Baseline | ~5% lower | Better GC due to less global state |
| Startup time | Baseline | ~Same | Config loading is lazy |

### Optimization Opportunities

The refactored code is easier to optimize:
- Transformations can be parallelized (not in v1.0)
- File I/O can be batched (not in v1.0)
- Caching is now explicit and tunable

---

## Deprecation Schedule

### Removed in v1.0

The following modules and functions are **removed** (not deprecated):

- `services.domain_processing_coordinator` → Use `application.DomainProcessingUseCase`
- `services.domain_synthesis_coordinator` → Use `application.DomainSynthesisUseCase`
- `services.study_orchestration_service` → Use `application.StudyProcessingUseCase`
- `cli.logging_config.get_logger()` global function → Inject logger dependency

### Removed in Future Versions

None planned. All breaking changes are in v1.0.

---

## Common Migration Scenarios

### Scenario 1: Processing a Single Domain

**Old:**
```python
from cdisc_transpiler.services import DomainProcessingCoordinator

coordinator = DomainProcessingCoordinator()
result = coordinator.process_and_merge_domain(
    files_for_domain=[(Path("DM.csv"), "DM")],
    domain_code="DM",
    study_id="STUDY001",
    output_format="xpt",
    xpt_dir=Path("output/xpt"),
    xml_dir=None,
    sas_dir=None,
    min_confidence=0.5,
    streaming=False,
    chunk_size=1000,
    generate_sas=False,
    verbose=False,
)
```

**New:**
```python
from cdisc_transpiler.application import DomainProcessingUseCase, ProcessDomainRequest
from cdisc_transpiler.infrastructure import create_default_dependencies

deps = create_default_dependencies()
use_case = DomainProcessingUseCase(
    logger=deps.logger,
    file_generator=deps.file_generator,
    csv_reader=deps.csv_reader,
    transformer_pipeline=deps.transformer_pipeline,
)

result = use_case.execute(ProcessDomainRequest(
    files=[(Path("DM.csv"), "DM")],
    domain_code="DM",
    study_id="STUDY001",
    output_dirs=OutputDirs(xpt_dir=Path("output/xpt")),
    formats={"xpt"},
    options=ProcessingOptions(
        min_confidence=0.5,
        generate_sas=False,
    ),
))
```

### Scenario 2: Custom Transformation

**Old:** Modify `study_orchestration_service.py` directly

**New:** Implement transformer interface
```python
from cdisc_transpiler.transformations.base import TransformerPort

class MyCustomTransformer:
    """Custom transformation for my special domain."""
    
    def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
        return domain == "MY_DOMAIN"
    
    def transform(
        self,
        df: pd.DataFrame,
        context: TransformationContext,
    ) -> pd.DataFrame:
        # Your custom logic here
        return transformed_df

# Register with pipeline:
pipeline = TransformationPipeline([
    MyCustomTransformer(),
    VSTransformer(ct_repo),
    LBTransformer(ct_repo),
])
```

### Scenario 3: Custom File Writer

**Old:** Duplicate file writing logic

**New:** Implement writer interface
```python
from cdisc_transpiler.infrastructure.io import FileWriterPort

class MyCustomWriter:
    """Write files in custom format."""
    
    def write(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        output_path: Path,
    ) -> None:
        # Your custom writing logic
        pass

# Register with file generator:
file_generator = FileGenerator(
    xpt_writer=XPTWriter(),
    xml_writer=XMLWriter(),
    custom_writer=MyCustomWriter(),  # Add your writer
    logger=logger,
)
```

---

## Rollback Plan

If you encounter critical issues with v1.0:

### Option 1: Pin to v0.x
```bash
pip install cdisc-transpiler==0.0.1
```

### Option 2: Use Legacy Branch
```bash
git checkout legacy-v0.x
pip install -e .
```

### Option 3: Report Issue
File an issue at: https://github.com/rubentalstra/cdisc-transpiler/issues

Include:
- Your use case
- Error messages
- Sample data (if possible)

---

## Getting Help

### Documentation
- **Architecture Overview**: `docs/target_architecture.md`
- **As-Is Documentation**: `docs/study_command_flow.md`
- **API Reference**: (Coming in v1.1)

### Community
- **GitHub Issues**: Report bugs and request features
- **Discussions**: Ask questions and share use cases

### Support Timeline
- **v0.x**: Security fixes only (12 months after v1.0 release)
- **v1.x**: Active development and support

---

## Checklist for Migration

Use this checklist to track your migration progress:

### CLI Users
- [ ] Verify existing commands still work
- [ ] (Optional) Create `cdisc_transpiler.toml` for custom defaults

### Library Users
- [ ] Update all import statements to new paths
- [ ] Replace direct service instantiation with dependency injection
- [ ] Update error handling to use new exception hierarchy
- [ ] Refactor global state access (e.g., `get_logger()`)
- [ ] Update tests to use mocked dependencies
- [ ] Verify output files match expected format

### Contributors
- [ ] Read `docs/target_architecture.md`
- [ ] Set up development environment with test suite
- [ ] Run existing test suite: `pytest tests/`
- [ ] Review contribution guidelines (coming soon)

---

## FAQ

### Q: Will this break my existing scripts?
**A:** If you only use the CLI (`cdisc-transpiler study ...`), no changes needed. If you import modules directly, see import path changes above.

### Q: Do I need to update my data files?
**A:** No. The input CSV format is unchanged.

### Q: Will output files be different?
**A:** No. XPT, XML, and Define-XML files should be identical (except for minor metadata like timestamps).

### Q: Can I use both old and new code during migration?
**A:** Yes, but not recommended. If needed, you can import from `cdisc_transpiler.legacy.*` during the transition period (until v1.1 removes legacy code).

### Q: How do I report a bug in the new version?
**A:** File an issue at https://github.com/rubentalstra/cdisc-transpiler/issues with:
- Version info: `cdisc-transpiler --version`
- Error message and stack trace
- Steps to reproduce
- (Optional) Sample data

### Q: Is the new version faster?
**A:** Performance should be similar. The main benefits are maintainability and testability, not speed.

### Q: Can I contribute to the refactoring?
**A:** Yes! See open issues labeled `refactoring` and `help-wanted`. Read `docs/target_architecture.md` first.

---

**Last Updated:** 2025-12-14  
**Next Review:** 2025-03-14 (3 months after v1.0 release)
