# Migration Guide: CLI Layer Refactoring & Test Infrastructure

**Last Updated:** 2025-12-15  
**Release:** CLI Refactoring (Epic 4 & Epic 5)  
**Breaking Changes:** No - 100% Backward Compatible ✅

---

## Overview

This guide documents the **incremental refactoring** completed in this release, focusing on CLI layer improvements and comprehensive test infrastructure. This is **not a breaking release** - all existing code continues to work unchanged.

### What Changed in This Release?

✅ **CLI Layer**: Refactored to thin adapter pattern with presenters  
✅ **Test Suite**: Added 485+ tests (unit, integration, validation, benchmarks)  
✅ **Documentation**: Updated README with architecture overview  
✅ **Backward Compatibility**: 100% maintained via aliases and unchanged APIs

### Who Should Read This?

- **CLI Users**: No changes required. CLI interface is identical. ✅
- **Library Users**: Optional new presenter classes available. Old imports still work. ✅
- **Contributors**: New test infrastructure and coding patterns to follow. 📖
- **Maintainers**: New architecture enables future refactoring with confidence. 🎯

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

### What's New Under the Hood?

While the CLI interface is unchanged, the implementation now uses:
- **Thin Adapter Pattern**: CLI code reduced from 595 to 225 lines
- **Presenters**: `SummaryPresenter` and `ProgressPresenter` for formatted output
- **Dependency Injection**: Better testability and maintainability

**You don't need to know or care about these changes** - they're internal improvements that make the codebase easier to test and maintain.

---

## Library Users: Optional New Features

If you're using `cdisc_transpiler` as a library in your own Python code, you have access to new presenter classes. **All old imports continue to work.**

### New Presenters Available

#### 1. SummaryPresenter

Format study processing results as a Rich table (what you see in CLI output):

```python
from cdisc_transpiler.cli.presenters import SummaryPresenter
from rich.console import Console

console = Console()
presenter = SummaryPresenter(console=console, results=domain_results)
presenter.present(
    study_id="STUDY001",
    output_dir=Path("output"),
    formats={"xpt", "xml"},
    generate_sas=False,
)
```

#### 2. ProgressPresenter

Track domain processing progress with emoji indicators:

```python
from cdisc_transpiler.cli.presenters import ProgressPresenter

progress = ProgressPresenter(total_domains=10, console=console)

# Track each domain
progress.increment(success=True)   # ✓
progress.increment(error=True)     # ✗  
progress.increment(warning=True)   # ⚠

# Show summary
progress.print_summary()

# Get progress percentage
percentage = progress.progress_percentage  # 30.0

# Check if complete
if progress.is_complete:
    print("All domains processed!")
```

### Backward Compatibility: ProgressTracker

The old `ProgressTracker` name still works as an alias:

```python
# Old code still works:
from cdisc_transpiler.cli.utils import ProgressTracker

tracker = ProgressTracker(total_domains=10)
tracker.increment()
# ... continues to work exactly as before
```

---

## Deprecated Services (Moved to Legacy)

⚠️ **IMPORTANT**: The following service classes have been moved to the `legacy` package and are deprecated. They will be removed in the next major version.

### Deprecated Services

The following coordinators have been replaced by the new architecture:

| Deprecated Service | Replacement | Status |
|-------------------|-------------|---------|
| `DomainProcessingCoordinator` | `application.domain_processing_use_case.DomainProcessingUseCase` | ⚠️ Deprecated |
| `DomainSynthesisCoordinator` | `application.study_processing_use_case.StudyProcessingUseCase` | ⚠️ Deprecated |
| `StudyOrchestrationService` | `application.study_processing_use_case.StudyProcessingUseCase` | ⚠️ Deprecated |

### What You Should Do

**If you're using these services:**

1. **Short term (current release)**: Your code continues to work unchanged, but you'll see deprecation warnings:
   ```python
   from cdisc_transpiler.services import DomainProcessingCoordinator  # ⚠️ Shows deprecation warning
   
   coordinator = DomainProcessingCoordinator()
   # ... your code works as before
   ```

2. **Long term (before next major version)**: Migrate to the new use case architecture:
   ```python
   # NEW: Use the application layer use cases
   from cdisc_transpiler.application import StudyProcessingUseCase, DomainProcessingUseCase
   from cdisc_transpiler.application.models import ProcessStudyRequest, ProcessDomainRequest
   from cdisc_transpiler.infrastructure.container import DependencyContainer
   
   # Create dependencies
   container = DependencyContainer()
   logger = container.create_logger()
   
   # Use study processing use case
   use_case = StudyProcessingUseCase(logger=logger)
   request = ProcessStudyRequest(study_folder=Path("path/to/study"))
   response = use_case.execute(request)
   ```

### Why Were They Deprecated?

These services mixed multiple concerns and had tight coupling, making them difficult to test and maintain. The new use case architecture provides:

- ✅ Clear separation of concerns
- ✅ Dependency injection for testability
- ✅ Better error handling and reporting
- ✅ Cleaner API with explicit request/response DTOs
- ✅ Easier to extend and modify

### Migration Timeline

- **Current Release (v0.0.1)**: Services moved to `legacy/`, deprecation warnings added
- **Next Release (v1.0.0)**: Legacy services will be permanently removed

### Suppressing Deprecation Warnings (Not Recommended)

If you need to suppress warnings temporarily during migration:

```python
import warnings
warnings.filterwarnings('ignore', category=DeprecationWarning)

from cdisc_transpiler.services import DomainProcessingCoordinator
# No warnings shown (but still deprecated!)
```

**Note**: We recommend fixing the deprecation warnings rather than suppressing them.

---

## Testing Infrastructure (NEW)

This release adds comprehensive testing infrastructure with **485+ tests**:

### Test Suites

#### 1. Unit Tests (440+ tests, 76% coverage)
```bash
# Run all unit tests
pytest tests/unit/

# Run specific module tests
pytest tests/unit/cli/presenters/
pytest tests/unit/application/
pytest tests/unit/transformations/
```

**Coverage by layer:**
- CLI Layer: 95%+ ✅
- Application Layer: 85%+ ✅
- Infrastructure Layer: 85%+ ✅
- Transformation Layer: 91-100% ✅

#### 2. Integration Tests (40+ tests)
```bash
# Run all integration tests
pytest tests/integration/

# Run fast tests only (skip slow)
pytest tests/integration/ -m "not slow"

# Run with specific dataset
pytest tests/integration/test_study_workflow.py
```

**Test files:**
- `test_cli.py`: 20 CLI end-to-end tests
- `test_study_workflow.py`: Study processing workflows
- `test_domain_workflow.py`: Domain processing tests

#### 3. Validation Tests (42 tests)
```bash
# Run all validation tests
pytest -m validation

# Run specific validation suite
pytest tests/validation/test_sdtm_compliance.py    # 12 tests
pytest tests/validation/test_xpt_format.py          # 9 tests
pytest tests/validation/test_xml_format.py          # 9 tests
pytest tests/validation/test_define_xml_format.py   # 12 tests
```

**What's validated:**
- SDTM compliance (required variables, types, controlled terminology)
- XPT format (SAS readability, metadata, data integrity)
- XML format (well-formedness, structure, encoding)
- Define-XML (ODM structure, metadata elements, attributes)

#### 4. Performance Benchmarks (3 tests)
```bash
# Run benchmarks
pytest -m benchmark --benchmark-only

# Save baseline
pytest -m benchmark --benchmark-only --benchmark-save=baseline

# Compare against baseline (fail if >10% slower)
pytest -m benchmark --benchmark-only --benchmark-compare=baseline --benchmark-compare-fail=mean:10%
```

**What's benchmarked:**
- Small study processing (DEMO_CF: 11 domains) → <5s baseline
- Large study processing (DEMO_GDISC: 18 domains) → <20s baseline
- DataFrame operations (1000 rows) → ~2ms baseline

### Test Documentation

- `TEST_COVERAGE_REPORT.md`: Coverage analysis and gaps
- `TEST_INTEGRATION_REPORT.md`: Integration test inventory
- `tests/integration/BENCHMARK_README.md`: Benchmark usage guide

---

## What's NOT Changed (Important!)

The following components **remain unchanged** in this release. They will be refactored in future releases:

❌ **Not Changed:**
- Core business logic (study/domain processing)
- Infrastructure layer (file I/O, repositories)
- Domain models (`SDTMDomain`, `SDTMVariable`)
- Configuration file format
- Import paths for most modules
- File output formats (XPT, XML, Define-XML)

✅ **Changed:**
- CLI command implementation (internal only)
- New presenter classes (optional)
- Test infrastructure (new)
- Documentation (enhanced)

**Bottom line**: If your code doesn't import CLI-specific modules, you won't see any changes.

---

## Common Migration Scenarios

### Scenario 1: You Use the CLI Only

**Action Required:** None ✅

```bash
# Just keep using it as before:
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/
```

### Scenario 2: You Want to Use New Presenters

**Action:** Optional - use new presenter classes for formatted output

```python
from cdisc_transpiler.cli.presenters import SummaryPresenter, ProgressPresenter
from rich.console import Console

console = Console()

# Use SummaryPresenter for study results
presenter = SummaryPresenter(console=console, results=results)
presenter.present(study_id="STUDY001", output_dir=Path("output"), ...)

# Use ProgressPresenter for tracking
progress = ProgressPresenter(total_domains=10, console=console)
progress.increment(success=True)
progress.print_summary()
```

### Scenario 3: You Use ProgressTracker

**Action Required:** None ✅ (backward compatible alias exists)

```python
# Old code continues to work:
from cdisc_transpiler.cli.utils import ProgressTracker

tracker = ProgressTracker(total_domains=10)
tracker.increment()
tracker.print_summary()
```

### Scenario 4: You Want to Run Tests

**Action:** Use new test infrastructure

```bash
# Install dev dependencies
pip install -e ".[dev]"

# Run all tests
pytest

# Run specific test suites
pytest tests/unit/
pytest tests/integration/
pytest -m validation
pytest -m benchmark --benchmark-only
```

---

## Rollback Plan

Since this release is **100% backward compatible**, rollback is not necessary. However, if you encounter issues:

### Option 1: Continue Using Current Version
No rollback needed - all old code continues to work.

### Option 2: Report Issue
File an issue at: https://github.com/rubentalstra/cdisc-transpiler/issues

Include:
- Your use case
- Error messages (if any)
- Whether you're using CLI or library
- Sample code (if applicable)

---

## Getting Help

### Documentation
- **README.md**: Quick start and architecture overview
- **TEST_COVERAGE_REPORT.md**: Test coverage analysis
- **TEST_INTEGRATION_REPORT.md**: Integration test documentation
- **tests/integration/BENCHMARK_README.md**: Performance benchmarking guide
- **Architecture Docs**: `docs/target_architecture.md` (future roadmap)

### Community
- **GitHub Issues**: Report bugs and request features
- **GitHub Discussions**: Ask questions and share use cases

---

## Checklist for Migration

Use this checklist to track your migration progress:

### CLI Users
- [ ] Verify existing commands still work (they should!)
- [ ] (Optional) Explore new test suites for validation

### Library Users
- [ ] No action required - old imports still work
- [ ] (Optional) Explore new `SummaryPresenter` and `ProgressPresenter` classes
- [ ] (Optional) Review test infrastructure for your own testing needs

### Contributors
- [ ] Read updated `README.md` 
- [ ] Review test infrastructure: `pytest tests/`
- [ ] Run tests locally to verify setup: `pytest`
- [ ] Explore validation tests: `pytest -m validation`
- [ ] Try performance benchmarks: `pytest -m benchmark --benchmark-only`

---

## FAQ

### Q: Will this break my existing scripts?
**A:** No! This release is 100% backward compatible. CLI commands work identically, and all existing library imports continue to work.

### Q: Do I need to update my data files?
**A:** No. The input CSV format is unchanged.

### Q: Will output files be different?
**A:** No. XPT, XML, and Define-XML files are identical to before (except minor metadata like timestamps).

### Q: What's new in this release?
**A:** Internal improvements (CLI refactoring, test infrastructure) that make the codebase easier to maintain and extend. You get better quality without any changes to your code.

### Q: Can I use the new presenter classes?
**A:** Yes! `SummaryPresenter` and `ProgressPresenter` are available for optional use. See "Library Users" section above.

### Q: Do I need to write tests for my code now?
**A:** No requirement, but you now have a comprehensive test suite to learn from. See `tests/` directory for examples.

### Q: What about validation tests?
**A:** New validation tests (42 tests) verify SDTM compliance, XPT format, XML format, and Define-XML structure. Run with: `pytest -m validation`

### Q: How do I run performance benchmarks?
**A:** Run: `pytest -m benchmark --benchmark-only`. See `tests/integration/BENCHMARK_README.md` for details.

### Q: Is this the final architecture?
**A:** No, this is an incremental step. Future releases will continue refactoring toward full Ports & Adapters architecture (see `docs/target_architecture.md`).

### Q: How do I report a bug?
**A:** File an issue at https://github.com/rubentalstra/cdisc-transpiler/issues with:
- Version info: `cdisc-transpiler --version`
- Error message and stack trace (if any)
- Steps to reproduce
- Whether you're using CLI or library

### Q: Can I contribute to the project?
**A:** Yes! Check open issues and read the test infrastructure documentation. The new test suite makes it easier to contribute with confidence.

---

**Last Updated:** 2025-12-15  
**Next Review:** After next major refactoring phase
