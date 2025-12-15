# CDISC Transpiler

[![Python 3.10+](https://img.shields.io/badge/python-3.10+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests](https://img.shields.io/badge/tests-485%20passing-brightgreen.svg)](tests/)
[![Coverage](https://img.shields.io/badge/coverage-76%25-green.svg)](tests/)

A modern Python tool for transpiling clinical trial data to CDISC SDTM format with support for multiple output formats (XPT, Dataset-XML, Define-XML, and SAS).

## ✨ Features

- 🔄 **Multiple Output Formats**: Generate XPT, Dataset-XML, Define-XML 2.1, and SAS programs
- 📊 **SDTM Compliance**: Automatic transformation to SDTM 3.2/3.4 standards
- 🏗️ **Clean Architecture**: Ports & Adapters (Hexagonal) architecture for maintainability
- ⚡ **High Performance**: Process studies with 18+ domains in ~2 seconds
- 🧪 **Comprehensive Testing**: 485+ tests with 76% code coverage
- ✅ **Validation Suite**: 42 tests for SDTM compliance and file format validation
- 📈 **Performance Benchmarks**: Track and prevent performance regressions
- 🎯 **Domain Synthesis**: Automatic generation of supplemental and variant domains

## 🏗️ Architecture

This project follows **Ports & Adapters (Hexagonal Architecture)** for clean separation of concerns:

```
cdisc_transpiler/
├── cli/                    # CLI Layer (Adapters)
│   ├── commands/          # Thin CLI commands (argument parsing only)
│   └── presenters/        # Output formatting (SummaryPresenter, ProgressPresenter)
├── application/           # Application Layer (Use Cases)
│   ├── use_cases/        # Business workflows (StudyProcessingUseCase)
│   └── ports/            # Interfaces/abstractions
├── domain/               # Domain Layer (Business Logic)
│   ├── entities/         # Domain models (Study, Domain, Variable)
│   └── services/         # Domain services (DomainProcessor, Synthesizer)
└── infrastructure/       # Infrastructure Layer (I/O, External Systems)
    ├── repositories/     # Data access
    ├── file_generators/  # XPT, XML, SAS generation
    └── transformers/     # Data transformation pipeline
```

**Benefits:**
- ✅ **Testability**: Business logic isolated from I/O and CLI
- ✅ **Maintainability**: Clear boundaries and single responsibility
- ✅ **Flexibility**: Easy to swap implementations (e.g., different file formats)
- ✅ **Scalability**: Can add new features without touching core logic

## 📦 Installation

### Prerequisites

- Python 3.10 or higher
- pip package manager

### Standard Installation

```bash
pip install cdisc-transpiler
```

### Development Installation

```bash
# Clone the repository
git clone https://github.com/rubentalstra/cdisc-transpiler.git
cd cdisc-transpiler

# Create and activate virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install with development dependencies
pip install -e .[dev]
```

## 🚀 Usage

### Quick Start

Process a study folder to generate all output formats:

```bash
# Activate virtual environment
source .venv/bin/activate

# Process study with default settings (XPT + Dataset-XML + Define-XML)
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/

# Verbose output for debugging
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/ -vv
```

### Output Formats

```bash
# Generate only XPT files
cdisc-transpiler study mockdata/DEMO_CF1234_NL_20250120_104838/ --format xpt

# Generate only Dataset-XML
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/ --format xml

# Generate both XPT and XML
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/ --format both

# Include SAS programs
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/ --sas

# Generate Define-XML 2.1
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/ --define-xml
```

### List Supported Domains

```bash
cdisc-transpiler domains
```

### Example Output

```
📊 Study Processing Summary
┏━━━━━━━━━┳━━━━━━━━━━━┳━━━━━━━┳━━━━━━━━━━━━━━━┳━━━━━━━┓
┃ Domain  ┃   Records ┃  XPT  ┃  Dataset-XML  ┃  SAS  ┃
┡━━━━━━━━━╇━━━━━━━━━━━╇━━━━━━━╇━━━━━━━━━━━━━━━╇━━━━━━━┩
│ AE      │         8 │   ✓   │       ✓       │   ✓   │
│ DM      │        10 │   ✓   │       ✓       │   ✓   │
│ EX      │        15 │   ✓   │       ✓       │   ✓   │
│ LB      │        42 │   ✓   │       ✓       │   ✓   │
│ VS      │        38 │   ✓   │       ✓       │   ✓   │
└─────────┴───────────┴───────┴───────────────┴───────┘
✓ 5 domains processed successfully
```

## 🧪 Testing

The project has comprehensive test coverage across multiple test suites:

### Test Suites

| Suite | Tests | Coverage | Purpose |
|-------|-------|----------|---------|
| **Unit Tests** | 440+ | 76% | Core business logic, transformations, presenters |
| **Integration Tests** | 40+ | - | End-to-end workflows with real data |
| **Validation Tests** | 42 | - | SDTM compliance, XPT/XML/Define-XML format validation |
| **Performance Benchmarks** | 3 | - | Track and prevent performance regressions |

### Running Tests

```bash
# Run all tests
pytest

# Run only unit tests (fast)
pytest tests/unit/

# Run integration tests
pytest tests/integration/

# Run validation tests (SDTM compliance, file formats)
pytest -m validation

# Run performance benchmarks
pytest -m benchmark --benchmark-only

# Run with coverage report
pytest --cov=cdisc_transpiler --cov-report=html

# Run specific test file
pytest tests/unit/cli/presenters/test_summary.py -v
```

### Test Markers

```bash
# Skip slow tests
pytest -m "not slow"

# Only validation tests
pytest -m validation

# Only benchmark tests
pytest -m benchmark
```

### Test Organization

```
tests/
├── unit/                  # Unit tests (440+ tests)
│   ├── application/      # Use case tests
│   ├── cli/              # Presenter and command tests
│   ├── domain/           # Domain logic tests
│   └── infrastructure/   # File generation, transformation tests
├── integration/          # Integration tests (40+ tests)
│   ├── test_cli.py       # CLI end-to-end tests
│   ├── test_study_workflow.py
│   ├── test_domain_workflow.py
│   └── test_performance_benchmarks.py
└── validation/           # Validation tests (42 tests)
    ├── test_sdtm_compliance.py      # SDTM standards validation
    ├── test_xpt_format.py           # XPT format validation
    ├── test_xml_format.py           # Dataset-XML validation
    └── test_define_xml_format.py    # Define-XML validation
```

## 💻 Development

### Setup Development Environment

```bash
# Install development dependencies
pip install -e .[dev]

# Install pre-commit hooks (optional)
pre-commit install
```

### Code Quality Tools

```bash
# Type checking with pyright
pyright

# Linting with ruff
ruff check .

# Format code with ruff
ruff format .

# Run all quality checks
pyright && ruff check . && pytest
```

### Development Workflow

1. **Write tests first** (TDD approach)
2. **Implement feature** in appropriate layer
3. **Run tests** to verify
4. **Check code quality** with pyright and ruff
5. **Commit changes** with descriptive message

### Performance Benchmarking

```bash
# Run benchmarks and save baseline
pytest -m benchmark --benchmark-only --benchmark-save=baseline

# Compare against baseline
pytest -m benchmark --benchmark-only --benchmark-compare=baseline

# Fail if >10% slower
pytest -m benchmark --benchmark-only --benchmark-compare=baseline --benchmark-compare-fail=mean:10%
```

## 📁 Project Structure

```
cdisc-transpiler/
├── cdisc_transpiler/           # Main package
│   ├── __init__.py
│   ├── cli/                    # CLI layer (Ports & Adapters)
│   │   ├── commands/          # Click commands (study, domains)
│   │   │   ├── study.py       # Study processing command (thin adapter)
│   │   │   └── domains.py     # List domains command
│   │   ├── presenters/        # Output formatting
│   │   │   ├── summary.py     # SummaryPresenter (table formatting)
│   │   │   └── progress.py    # ProgressPresenter (progress tracking)
│   │   └── helpers.py         # CLI utilities
│   ├── application/           # Application layer (Use Cases)
│   │   ├── use_cases/        # Business workflows
│   │   │   ├── study_processing_use_case.py
│   │   │   └── domain_processing_use_case.py
│   │   ├── ports/            # Interfaces
│   │   └── models/           # DTOs (ProcessStudyRequest/Response)
│   ├── domain/               # Domain layer (Business Logic)
│   │   ├── entities/         # Domain models
│   │   │   ├── study.py
│   │   │   ├── domain.py
│   │   │   └── variable.py
│   │   └── services/         # Domain services
│   │       ├── domain_processor.py
│   │       ├── synthesizer.py
│   │       └── discovery_service.py
│   ├── infrastructure/       # Infrastructure layer (I/O)
│   │   ├── dependency_container.py  # DI container
│   │   ├── csv_reader.py
│   │   ├── file_generators/  # Output generation
│   │   │   ├── xpt_generator.py
│   │   │   ├── xml_generator.py
│   │   │   ├── define_xml_generator.py
│   │   │   └── sas_generator.py
│   │   └── transformers/     # Data transformation pipeline
│   │       ├── base_transformer.py
│   │       ├── date_formatter.py
│   │       └── codelist_mapper.py
│   └── metadata_module/      # CDISC metadata and standards
├── tests/                    # Test suites
├── mockdata/                 # Test data (DEMO studies)
├── pyproject.toml           # Project configuration
└── README.md                # This file
```

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. **Report bugs** via GitHub Issues
2. **Suggest features** or improvements
3. **Submit pull requests** with bug fixes or new features
4. **Improve documentation**
5. **Add test coverage**

### Contribution Guidelines

- Follow the existing code style (ruff formatting)
- Write tests for new features
- Ensure all tests pass (`pytest`)
- Run type checking (`pyright`)
- Update documentation as needed

See [implementation_tickets.md](implementation_tickets.md) for planned features and tasks.

## 📚 Documentation

- **CDISC SDTM Standards**: https://library.cdisc.org/browser/#/mdr/sdtmig/3-4
- **Test Coverage Report**: [TEST_COVERAGE_REPORT.md](TEST_COVERAGE_REPORT.md)
- **Integration Test Report**: [TEST_INTEGRATION_REPORT.md](TEST_INTEGRATION_REPORT.md)
- **Performance Benchmarks**: [tests/integration/BENCHMARK_README.md](tests/integration/BENCHMARK_README.md)

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🔗 Links

- **Repository**: https://github.com/rubentalstra/cdisc-transpiler
- **Issues**: https://github.com/rubentalstra/cdisc-transpiler/issues
- **CDISC Library**: https://www.cdisc.org/standards/foundational/sdtm

---

**Built with ❤️ for the clinical research community**
