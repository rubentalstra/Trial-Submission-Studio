# CDISC Transpiler

[![Python 3.14+](https://img.shields.io/badge/python-3.14+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests](https://img.shields.io/badge/tests-485%20passing-brightgreen.svg)](tests/)
[![Coverage](https://img.shields.io/badge/coverage-76%25-green.svg)](tests/)

A modern Python tool for transpiling clinical trial data to CDISC SDTM format
with support for multiple output formats (XPT, Dataset-XML, Define-XML, and
SAS).

## ✨ Features

## 📦 Installation

### Prerequisites

- Python 3.14 or higher
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

| Suite                      | Purpose                                               |
| -------------------------- | ----------------------------------------------------- |
| **Unit Tests**             | Core business logic, normalization, presenters        |
| **Integration Tests**      | End-to-end workflows with real data                   |
| **Validation Tests**       | SDTM compliance, XPT/XML/Define-XML format validation |
| **Performance Benchmarks** | Track and prevent performance regressions             |

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

## 📚 Documentation

- **CDISC SDTM Standards**: https://library.cdisc.org/browser/#/mdr/sdtmig/3-4
- **Performance Benchmarks**:
  [tests/integration/BENCHMARK_README.md](tests/integration/BENCHMARK_README.md)

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for
details.

## 🔗 Links

- **Repository**: https://github.com/rubentalstra/cdisc-transpiler
- **Issues**: https://github.com/rubentalstra/cdisc-transpiler/issues
- **CDISC Library**: https://www.cdisc.org/standards/foundational/sdtm

---

**Built with ❤️ for the clinical research community**
