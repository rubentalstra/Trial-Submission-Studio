# Contributing to Trial Submission Studio

Thank you for your interest in contributing! This guide will help you get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Architecture](#project-architecture)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)

## Code of Conduct

Please keep collaboration respectful and constructive:

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Assume good intentions

## Security Best Practices

### Multi-Factor Authentication (MFA)

We strongly recommend enabling MFA on your GitHub account. SignPath Foundation
requires MFA for SignPath access and recommends it for source code repository
access as well.

To enable MFA on GitHub:
[GitHub Two-Factor Authentication](https://docs.github.com/en/authentication/securing-your-account-with-two-factor-authentication-2fa)

## Getting Started

### Prerequisites

- Rust 1.92+ (see `rust-toolchain.toml`)
- Git
- (Optional) Basic familiarity with CDISC SDTM standards

### Finding Issues

1. Check GitHub Issues
2. Look for `good-first-issue` or `help-wanted` labels
3. Comment on an issue before starting work

## Development Setup

### 1. Fork and Clone

```bash
git clone https://github.com/YOUR_USERNAME/trial-submission-studio.git
cd trial-submission-studio

git remote add upstream https://github.com/rubentalstra/trial-submission-studio.git
```

### 2. Install Rust

```bash
rustup show
rustup toolchain install 1.92
```

### 3. Build and Run

```bash
# Build all crates
cargo build

# Run the GUI application
cargo run --package tss-gui

# Run tests
cargo test

# Run lints
cargo clippy
```

### 4. Third-Party Licenses

When adding or updating dependencies, regenerate the third-party licenses file:

```bash
# Install cargo-about (one-time)
cargo install cargo-about

# Generate licenses (re-run when deps change)
cargo about generate about.hbs -o THIRD_PARTY_LICENSES.md
```

> **Note:** You may see a warning about `GPL-2.0` being a deprecated license identifier.
> This is from an upstream dependency and can be safely ignored—the file generates successfully.

This file is embedded in the application and displayed in Help > Third-Party Licenses.

## Project Architecture

Trial Submission Studio is organized as a 10-crate Rust workspace:

| Crate           | Purpose                                      |
|-----------------|----------------------------------------------|
| `tss-gui`       | Desktop GUI application (egui/eframe)        |
| `xport`         | XPT (SAS Transport) file I/O                 |
| `tss-validate`  | CDISC conformance validation                 |
| `tss-map`       | Fuzzy column mapping engine                  |
| `tss-normalization` | Data transformation rules                |
| `tss-ingest`    | CSV discovery and parsing                    |
| `tss-output`    | Multi-format export (XPT, XML)               |
| `tss-standards` | CDISC standards loader                       |
| `tss-model`     | Core domain types + Polars utilities         |
| `tss-updater`   | Auto-update functionality                    |

### Key Directories

- `crates/` - All Rust crates
- `standards/` - Embedded CDISC standards (SDTM, ADaM, SEND, CT)
- `mockdata/` - Test datasets
- `docs/` - Technical documentation

## Development Workflow

1. Create a branch from `main`
2. Make changes in the appropriate crate
3. Add or update tests
4. Run quality checks:
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   ```
5. Open a pull request

## Coding Standards

### Rust Style

- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Prefer explicit types for public APIs
- Write actionable, user-facing error messages

### Architecture Principles

- Keep business logic out of GUI and I/O layers
- Maintain deterministic, auditable behavior
- Prefer pure functions in mapping and validation
- Standards are embedded locally (no external API calls during validation)

## Testing

### Test Types

- **Unit tests**: Per-function/module tests
- **Integration tests**: End-to-end workflows
- **Snapshot tests**: Using `insta` for output stability
- **Property tests**: Using `proptest` for edge cases

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test --package xport

# With output
cargo test -- --nocapture
```

## Submitting Changes

### Commit Messages

Use conventional commits:

- `feat:` new features
- `fix:` bug fixes
- `docs:` documentation
- `test:` tests
- `refactor:` code refactoring
- `perf:` performance improvements
- `chore:` maintenance

### Pull Request Guidelines

- Keep PRs focused and scoped
- Reference related issues
- Include tests for new functionality
- Update documentation if needed

## Questions?

Open an issue or start a discussion on GitHub.