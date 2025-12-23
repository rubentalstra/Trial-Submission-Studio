# Contributing to CDISC Transpiler

Thank you for your interest in contributing to CDISC Transpiler! This guide will
help you get started with contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Submitting Changes](#submitting-changes)
- [Issue Guidelines](#issue-guidelines)
- [Pull Request Process](#pull-request-process)
- [Architecture Guidelines](#architecture-guidelines)
- [Documentation](#documentation)

## Code of Conduct

This project follows a professional code of conduct:

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Assume good intentions
- Respect different viewpoints and experiences

## Getting Started

### Prerequisites

- Python 3.10 or higher
- Git
- Basic understanding of CDISC SDTM standards (helpful but not required)
- Familiarity with pytest for testing

### Finding Issues to Work On

1. Check the [Issues](https://github.com/rubentalstra/cdisc-transpiler/issues)
   page
2. Look for issues labeled `good-first-issue` or `help-wanted`
3. Review `IMPLEMENTATION_PROGRESS.md` for planned work
4. Comment on an issue to express interest before starting work

## Development Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/cdisc-transpiler.git
cd cdisc-transpiler

# Add upstream remote
git remote add upstream https://github.com/rubentalstra/cdisc-transpiler.git
```

### 2. Create Virtual Environment

```bash
# Create virtual environment
python -m venv .venv

# Activate (Linux/Mac)
source .venv/bin/activate

# Activate (Windows)
.venv\Scripts\activate
```

### 3. Install Dependencies

```bash
# Install project with development dependencies
pip install -e ".[dev]"

# Verify installation
cdisc-transpiler --help
```

### 4. Verify Setup

```bash
# Run tests to ensure everything works
pytest

# Expected: 485+ tests passing, 14 skipped
# Execution time: ~75 seconds
```

## Development Workflow

### 1. Create a Branch

```bash
# Update your fork
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name
# Or for bug fixes
git checkout -b fix/issue-description
```

### 2. Make Changes

Follow the [Coding Standards](#coding-standards) and
[Architecture Guidelines](#architecture-guidelines) below.

### 3. Write Tests

All code changes must include tests:

- **New features**: Add unit tests and integration tests
- **Bug fixes**: Add regression tests
- **Refactoring**: Ensure existing tests pass

See [Testing Guidelines](#testing-guidelines) for details.

### 4. Run Tests Locally

```bash
# Run all tests
pytest

# Run specific test suites
pytest tests/unit/                    # Unit tests only
pytest tests/integration/             # Integration tests only
pytest -m validation                  # Validation tests only
pytest -m benchmark --benchmark-only  # Performance benchmarks only

# Run tests with coverage
pytest --cov=cdisc_transpiler --cov-report=html
```

### 5. Check Code Quality

```bash
# Type checking
pyright

# Linting
pyflakes cdisc_transpiler/

# Run all quality checks
pyright && pyflakes cdisc_transpiler/ && pytest
```

### 6. Commit Changes

Follow conventional commit format:

```bash
git add .
git commit -m "feat: add new domain processor for custom domains"
# Or
git commit -m "fix: correct variable type mapping in DM domain"
# Or
git commit -m "docs: update README with new usage examples"
# Or
git commit -m "test: add integration tests for XML generation"
```

**Commit message format:**

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `test:` Adding or updating tests
- `refactor:` Code refactoring
- `perf:` Performance improvements
- `chore:` Maintenance tasks

## Coding Standards

### Python Style

- **Python Version**: 3.10+ (use modern Python features)
- **Line Length**: Maximum 88 characters (Black default)
- **Imports**: Group by standard library, third-party, local (sorted
  alphabetically)
- **Type Hints**: Use type hints for all function signatures
- **Docstrings**: Use Google-style docstrings

### Code Organization

Follow the **Ports & Adapters (Hexagonal Architecture)**:

```
cdisc_transpiler/
├── cli/                    # CLI Layer (thin adapters only)
│   ├── commands/          # Click commands (argument parsing only)
│   └── presenters/        # Output formatting (no business logic)
├── application/           # Application Layer (use cases)
│   ├── ports/            # Interface definitions
│   ├── models.py         # DTOs (request/response objects)
│   ├── study_processing_use_case.py
│   └── domain_processing_use_case.py
├── domain/               # Domain Layer (core business logic)
│   ├── entities/         # Domain models
│   └── services/         # Domain services + normalization logic
└── infrastructure/       # Infrastructure Layer (I/O, external systems)
    ├── container.py      # Composition root
    ├── io/               # Dataset output + writers
    ├── repositories/     # Data access
    ├── logging/          # Logger adapters
    └── sdtm_spec/         # SDTM spec registry
```

### Key Principles

1. **Separation of Concerns**: Keep CLI, business logic, and infrastructure
   separate
2. **Dependency Injection**: Use `DependencyContainer` for wiring dependencies
3. **Single Responsibility**: Each class/function should have one clear purpose
4. **Testability**: Write code that's easy to test (avoid tight coupling)
5. **Immutability**: Prefer immutable data structures where possible

### Example: Good vs. Bad

**❌ Bad (mixed concerns in CLI):**

```python
@click.command()
def study(study_folder: str):
    # DON'T: Business logic in CLI command
    files = os.listdir(study_folder)
    data = pd.read_csv(files[0])
    transformed = transform_data(data)
    output = generate_xpt(transformed)
```

**✅ Good (thin adapter pattern):**

```python
@click.command()
def study(study_folder: str):
    # DO: CLI only parses args and delegates
    container = DependencyContainer()
    use_case = container.create_study_processing_use_case()
    
    request = ProcessStudyRequest(study_folder=study_folder)
    response = use_case.execute(request)
    
    presenter = SummaryPresenter(console)
    presenter.present(response)
```

## Testing Guidelines

### Test Structure

We maintain 4 types of tests:

#### 1. Unit Tests (`tests/unit/`)

- **Purpose**: Test individual classes/functions in isolation
- **Characteristics**: Fast (<1ms per test), no I/O, use mocks/stubs
- **Coverage Target**: >90% for new code
- **Location**: Mirror source structure

**Example:**

```python
def test_summary_presenter_formats_table():
    presenter = SummaryPresenter(Console())
    results = [ProcessedDomain(domain="DM", records=10)]
    
    table = presenter._build_summary_table(results)
    
    assert table.title == "📊 Study Processing Summary"
    assert len(table.rows) == 1
```

#### 2. Integration Tests (`tests/integration/`)

- **Purpose**: Test multiple components working together
- **Characteristics**: Use real file I/O, test workflows end-to-end
- **Data**: Use mockdata (DEMO_GDISC, DEMO_CF)
- **Execution**: ~40 seconds total

**Example:**

```python
def test_study_processing_generates_xpt_files(tmp_path):
    study_folder = Path("mockdata/DEMO_GDISC")
    output_dir = tmp_path / "output"
    
    container = DependencyContainer()
    use_case = container.create_study_processing_use_case()
    
    request = ProcessStudyRequest(
        study_folder=study_folder,
        output_dir=output_dir,
        output_formats=["xpt"]
    )
    response = use_case.execute(request)
    
    assert response.success
    assert (output_dir / "xpt" / "dm.xpt").exists()
```

#### 3. Validation Tests (`tests/validation/`)

- **Purpose**: Validate SDTM compliance and file format correctness
- **Characteristics**: Use pyreadstat for XPT validation, XML parsers
- **Scope**: SDTM standards, file formats, controlled terminology
- **Marker**: `@pytest.mark.validation`

**Example:**

```python
@pytest.mark.validation
def test_dm_domain_has_required_variables(processed_study):
    dm_file = processed_study / "xpt" / "dm.xpt"
    df, meta = pyreadstat.read_xport(dm_file)
    
    required_vars = ["STUDYID", "DOMAIN", "USUBJID", "SUBJID"]
    for var in required_vars:
        assert var in df.columns, f"Missing required variable: {var}"
```

#### 4. Performance Benchmarks (`tests/integration/test_performance_benchmarks.py`)

- **Purpose**: Track performance and detect regressions
- **Tool**: pytest-benchmark
- **Marker**: `@pytest.mark.benchmark`
- **Usage**: `pytest -m benchmark --benchmark-only`

**Example:**

```python
@pytest.mark.benchmark
def test_benchmark_large_study_processing(benchmark):
    def process_study():
        # ... processing code ...
        return response
    
    result = benchmark(process_study)
    assert result.success
```

### Writing Tests

**Guidelines:**

1. **Arrange-Act-Assert**: Structure tests clearly
2. **One Assertion Per Test**: Test one thing at a time
3. **Descriptive Names**: `test_<what>_<when>_<expected>`
4. **Use Fixtures**: Share setup code via pytest fixtures
5. **Mock External Dependencies**: Don't call real APIs or databases in unit
   tests
6. **Clean Up**: Use `tmp_path` fixture for file operations

**Test Coverage:**

- All new code must have tests
- Bug fixes must include regression tests
- Aim for >90% coverage on new code
- Current project coverage: 76% (target: >80%)

### Running Tests

```bash
# All tests
pytest

# Specific suites
pytest tests/unit/                    # Fast unit tests (~30s)
pytest tests/integration/             # Integration tests (~40s)
pytest -m validation                  # Validation tests (~10s)
pytest -m "not slow"                  # Skip slow tests

# With coverage
pytest --cov=cdisc_transpiler --cov-report=html

# Performance benchmarks
pytest -m benchmark --benchmark-only
pytest -m benchmark --benchmark-only --benchmark-save=baseline
pytest -m benchmark --benchmark-compare=baseline
```

## Submitting Changes

### Before Submitting

**Checklist:**

- [ ] All tests pass locally
- [ ] Code follows style guidelines
- [ ] Type checking passes (pyright)
- [ ] Linting passes (pyflakes)
- [ ] Added tests for new functionality
- [ ] Updated documentation if needed
- [ ] Commit messages follow conventional format
- [ ] No unrelated changes included

### Creating a Pull Request

1. **Push your branch:**
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create PR on GitHub:**
   - Use a clear, descriptive title
   - Reference related issues: "Fixes #123" or "Relates to #456"
   - Fill out the PR template completely
   - Add relevant labels (feature, bug, documentation, etc.)

3. **PR Description should include:**
   - **Summary**: What does this PR do?
   - **Motivation**: Why is this change needed?
   - **Changes**: List of specific changes made
   - **Testing**: How was this tested?
   - **Checklist**: Mark completed items

### Example PR Description

```markdown
## Summary

Add support for custom domain processors via plugin system.

## Motivation

Users need ability to process custom domains not in SDTM standard.

## Changes

- Added `PluginRegistry` class in `domain/plugins/`
- Modified `DomainProcessingUseCase` to check for plugins
- Added integration tests for plugin loading
- Updated documentation with plugin development guide

## Testing

- Added 12 unit tests for `PluginRegistry`
- Added 3 integration tests with sample plugin
- All existing tests pass
- Manually tested with custom domain "ZZ"

## Checklist

- [x] Tests added and passing
- [x] Documentation updated
- [x] Type hints added
- [x] No breaking changes
```

## Issue Guidelines

### Reporting Bugs

**Include:**

1. **Description**: Clear description of the bug
2. **Steps to Reproduce**: Exact steps to reproduce the issue
3. **Expected Behavior**: What should happen
4. **Actual Behavior**: What actually happens
5. **Environment**: Python version, OS, package version
6. **Code Sample**: Minimal code to reproduce (if applicable)
7. **Error Messages**: Full error message and stack trace

### Feature Requests

**Include:**

1. **Use Case**: Why is this feature needed?
2. **Proposed Solution**: How should it work?
3. **Alternatives**: Other approaches considered
4. **Examples**: Example usage/code if applicable

### Asking Questions

- Check existing issues and documentation first
- Use discussions for general questions
- Be specific and provide context
- Include what you've already tried

## Pull Request Process

### Review Process

1. **Automated Checks**: CI runs tests, linting, type checking
2. **Code Review**: Maintainers review code quality and design
3. **Testing**: Reviewer may test functionality manually
4. **Feedback**: Address feedback and push updates
5. **Approval**: Once approved, maintainer will merge

### Review Criteria

Reviewers check for:

- **Correctness**: Does it solve the problem?
- **Tests**: Are there adequate tests?
- **Design**: Does it follow architecture patterns?
- **Documentation**: Is it well documented?
- **Style**: Does it follow coding standards?
- **Performance**: Are there performance concerns?
- **API Stability**: Any breaking changes? If yes, include migration notes and
  avoid adding compatibility shims/aliases—prefer updating call sites.

### After Merge

- Your changes will be in the next release
- You'll be added to contributors list
- Close any related issues

## Architecture Guidelines

### Ports & Adapters Pattern

When adding features, respect the architecture layers:

#### CLI Layer (`cdisc_transpiler/cli/`)

**Responsibilities:**

- Parse command-line arguments
- Validate basic input
- Create request DTOs
- Call use cases
- Format output

**DON'T:**

- Put business logic here
- Access infrastructure directly
- Perform complex calculations

#### Application Layer (`cdisc_transpiler/application/`)

**Responsibilities:**

- Orchestrate workflows
- Call domain services in correct order
- Handle cross-cutting concerns (logging, errors)
- Return structured responses

**DON'T:**

- Know about CLI or file formats
- Contain domain business rules
- Access infrastructure directly

#### Domain Layer (`cdisc_transpiler/domain/`)

**Responsibilities:**

- Core business logic
- Domain rules and validations
- Domain entities and services

**DON'T:**

- Depend on infrastructure
- Know about CLI or use cases
- Perform I/O operations

#### Infrastructure Layer (`cdisc_transpiler/infrastructure/`)

**Responsibilities:**

- File I/O (reading/writing)
- External system integration
- Output generation adapters
- Repository implementations

**DON'T:**

- Contain business logic
- Know about CLI or use cases

### Adding New Features

**Example: Adding a new domain processor**

1. **Domain Layer**: Create domain service
   ```python
   # domain/services/custom_domain_processor.py
   class CustomDomainProcessor:
       def process(self, data: DomainData) -> ProcessedDomain:
           # Business logic here
   ```

2. **Application Layer**: Update use case
   ```python
   # application/use_cases/study_processing_use_case.py
   class StudyProcessingUseCase:
       def __init__(self, custom_processor: CustomDomainProcessor):
           self.custom_processor = custom_processor
   ```

3. **Infrastructure**: Add data access if needed
   ```python
   # infrastructure/repositories/custom_domain_repository.py
   class CustomDomainRepository:
       def load(self, path: Path) -> DomainData:
           # I/O operations here
   ```

4. **CLI**: No changes needed (thin adapter pattern)

5. **Tests**: Add at all layers
   - Unit tests for `CustomDomainProcessor`
   - Integration test for full workflow
   - Validation test for output format

## Documentation

### Code Documentation

- **Docstrings**: All public classes, methods, functions
- **Type Hints**: All function signatures
- **Comments**: Explain _why_, not _what_
- **Examples**: Include usage examples in docstrings

**Example:**

```python
def process_domain(
    domain_data: DomainData,
    sdtm_version: str = "3.2",
) -> ProcessedDomain:
    """Process a single SDTM domain with SDTM normalization.
    
    Applies required normalization including variable mapping,
    controlled terminology, and data standardization.
    
    Args:
        domain_data: Raw domain data to process
        sdtm_version: SDTM version to target (default: "3.2")
    
    Returns:
        Processed domain with SDTM normalization applied
    
    Raises:
        ValidationError: If domain data fails SDTM validation
    
    Example:
        >>> processor = DomainProcessor()
        >>> data = DomainData(domain="DM", records=[...])
        >>> result = processor.process_domain(data)
        >>> print(result.record_count)
        42
    """
```

### Project Documentation

When adding features, update relevant docs:

- **README.md**: Usage examples, new features
- **docs/ARCHITECTURE.md**: Architecture, boundaries, migration guidance
- **Test Reports**: Update coverage reports if significant changes
- **Architecture Docs**: New patterns or components

## Getting Help

- **Questions**: Use GitHub Discussions
- **Bugs**: Open an issue with the bug report template
- **Features**: Open an issue with the feature request template
- **Chat**: (Add Discord/Slack link if available)

## Recognition

Contributors are recognized in:

- GitHub contributors page
- Release notes
- `CITATION.cff` file (for significant contributions)

## License

By contributing, you agree that your contributions will be licensed under the
MIT License.

---

**Happy Contributing! 🎉**

Questions? Open an issue or discussion - we're here to help!
