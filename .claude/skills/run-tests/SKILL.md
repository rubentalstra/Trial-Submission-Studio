---
name: run-tests
description: Run tests for a specific crate or domain
---
# Run Domain Tests

Execute targeted test suites for specific crates or functionality.

## Usage
- `/run-tests` - Run all tests
- `/run-tests <crate>` - Run tests for specific crate (e.g., `/run-tests tss-submit`)
- `/run-tests <crate> <filter>` - Run filtered tests (e.g., `/run-tests tss-submit validate`)

## Commands
```bash
# All tests
cargo test

# Specific crate
cargo test --package tss-submit
cargo test --package tss-standards
cargo test --package tss-ingest
cargo test --package tss-gui

# Filtered tests
cargo test --package tss-submit validate
cargo test --package tss-submit export
cargo test --package tss-submit normalize
```

## Test Categories
- **Unit tests**: Pure function tests (inline with `#[cfg(test)]`)
- **Integration tests**: Full pipeline with mock CSV data
- **Validation tests**: CDISC compliance checking
