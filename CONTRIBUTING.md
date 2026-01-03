# Contributing to Trial Submission Studio

Thank you for your interest in contributing to Trial Submission Studio. This project
is transitioning from a Python CLI to a Rust-first GUI. The Python implementation
remains as a reference until Rust parity is achieved.

## Table of Contents

- Code of Conduct
- Getting Started
- Development Setup
- Development Workflow
- Coding Standards
- Testing Guidelines
- Submitting Changes
- Issue Guidelines
- Pull Request Process
- Architecture Guidelines
- Documentation

## Code of Conduct

Please keep collaboration respectful and constructive:

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Assume good intentions

## Getting Started

### Prerequisites

- Rust toolchain (see `rust-toolchain.toml` for version)
- Git
- Basic familiarity with CDISC SDTM standards
- Python only if working on legacy code or parity tests

### Finding Issues to Work On

1. Check the GitHub issues page
2. Look for `good-first-issue` or `help-wanted`
3. Review `docs/REFRACTOR_PLAN.md` and `docs/RUST_CLI_TASKS.md`
4. Comment on an issue before starting work

## Development Setup

### 1. Fork and Clone

```bash
git clone https://github.com/YOUR_USERNAME/trial-submission-studio.git
cd trial-submission-studio

git remote add upstream https://github.com/rubentalstra/trial-submission-studio.git
```

### 2. Install Rust Toolchain

```bash
rustup show
rustup toolchain install 1.92
```

### 3. Build and Test

```bash
cargo build
cargo test
```

### 4. Legacy Python (Optional)

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e .[dev]
pytest
```

## Development Workflow

1. Create a branch off `main`
2. Implement changes in the appropriate crate
3. Add or update tests
4. Run `cargo fmt`, `cargo clippy`, and `cargo test`
5. Open a pull request with a clear description

## Coding Standards

### Rust Style

- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Prefer explicit types for public APIs
- Keep error messages actionable and user-facing

### Code Organization

Follow the workspace crate boundaries:

- `sdtm-cli`: CLI parsing and dependency wiring
- `sdtm-core`: use cases and orchestration
- `sdtm-map`: mapping engine and heuristics
- `sdtm-standards`: SDTM/CT loaders
- `sdtm-validate`: conformance checks
- `sdtm-report`: output writers and summaries

## Testing Guidelines

- Unit tests for mapping, transformers, and processors
- Integration tests for end-to-end study processing
- Golden tests for output parity
- Performance tests for regression detection

Run:

```bash
cargo test
cargo clippy
```

## Submitting Changes

Use conventional commits:

- `feat:` new features
- `fix:` bug fixes
- `docs:` documentation
- `test:` tests
- `refactor:` refactors
- `perf:` performance
- `chore:` maintenance

## Issue Guidelines

- Provide a clear, reproducible description
- Include sample inputs when possible
- Note expected vs actual behavior

## Pull Request Process

- Keep PRs focused and scoped
- Reference issues in the PR description
- Include tests or explain why not

## Architecture Guidelines

- Keep business logic out of CLI and I/O layers
- Maintain deterministic behavior and offline constraints
- Prefer pure functions in mapping and validation stages

## Documentation

- Update `docs/REFRACTOR_PLAN.md` for strategy changes
- Update `docs/RUST_CLI_TASKS.md` when adding or finishing tasks
