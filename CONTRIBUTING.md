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
