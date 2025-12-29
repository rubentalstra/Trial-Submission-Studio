---
name: release-check
description: Run pre-release validation checklist before merging or releasing. Use when preparing for PR merge, creating releases, or validating code quality.
---

# Release Check Skill

## Purpose

This skill automates the pre-release quality checklist to ensure code is ready for merge or release.

## When to Use

- Before creating a pull request
- Before merging to main branch
- Before cutting a release
- After significant refactoring
- When verifying code quality

## Pre-Release Checklist

### 1. Code Formatting
```bash
cargo fmt --all -- --check
```
Fix any issues:
```bash
cargo fmt --all
```

### 2. Linting
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
Address all clippy warnings before proceeding.

### 3. Build (Debug & Release)
```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release
```

### 4. Run All Tests
```bash
# All tests across all crates
cargo test --all

# With verbose output
cargo test --all -- --nocapture
```

### 5. Run Tests Per Crate
```bash
cargo test --package sdtm-model
cargo test --package sdtm-core
cargo test --package sdtm-ingest
cargo test --package sdtm-map
cargo test --package sdtm-validate
cargo test --package sdtm-standards
cargo test --package sdtm-xpt
cargo test --package sdtm-report
cargo test --package sdtm-cli
```

### 6. Documentation Build
```bash
cargo doc --no-deps --all
```

### 7. Check for TODOs/FIXMEs
```bash
# Find remaining TODOs
rg "TODO|FIXME" --type rust

# Or using grep
grep -r "TODO\|FIXME" crates/ --include="*.rs"
```

### 8. Dependency Check
```bash
# Check for outdated dependencies
cargo outdated

# Audit for security vulnerabilities
cargo audit
```

### 9. MSRV Compliance
Ensure minimum supported Rust version (MSRV: 1.92)
```bash
rustc --version
```

### 10. Integration Test
```bash
# Run full pipeline on test data
cargo run -- -s test_data/sample_study -o output/test_run/
```

## Quick Release Check

Run all critical checks in sequence:
```bash
cargo fmt --all --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --all && \
cargo build --release
```

## Common Issues & Fixes

### Formatting Issues
```bash
# Auto-fix formatting
cargo fmt --all
```

### Clippy Warnings
- Read warning message carefully
- Fix root cause, don't just suppress
- Use `#[allow(clippy::...)]` only when justified

### Test Failures
- Run specific test: `cargo test test_name`
- Use `-- --nocapture` to see println output
- Check test data in `test_data/` directory

### Build Failures
- Check dependency versions in `Cargo.toml`
- Ensure MSRV compatibility (1.92+)
- Review recent changes for breaking syntax

## Pre-PR Checklist

Before creating a pull request:

- [ ] All tests pass (`cargo test --all`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation builds (`cargo doc`)
- [ ] Commit messages follow conventional commits
- [ ] CLAUDE.md updated if architecture changed
- [ ] No debug/console output in production code

## Pre-Release Checklist

Before cutting a release:

- [ ] Version bumped in all affected `Cargo.toml` files
- [ ] CHANGELOG.md updated with changes
- [ ] All tests pass on main branch
- [ ] Integration tests pass with real data
- [ ] Documentation is up to date
- [ ] No security vulnerabilities (`cargo audit`)
- [ ] Git tags created appropriately

## Commit Style Verification

Check recent commits follow conventional format:
```bash
git log --oneline -10
```

Expected format: `type(scope): subject`
- Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`
- Examples:
  - `feat(dm): add AGE derivation from BRTHDTC`
  - `fix(validate): handle missing CT codelist gracefully`
  - `docs: update CLAUDE.md with new crate structure`

## Parallel Execution

Run independent checks in parallel for speed:
```bash
# Terminal 1
cargo test --all

# Terminal 2
cargo clippy --all-targets --all-features

# Terminal 3
cargo doc --no-deps --all
```

## Critical vs Non-Critical

### Critical (Must Pass)
- All tests pass
- No clippy errors
- Code formatted
- Builds successfully (debug + release)

### Non-Critical (Should Address)
- Documentation warnings
- Outdated dependencies
- TODOs/FIXMEs

## Related Commands

```bash
# Clean build artifacts
cargo clean

# Check without building
cargo check --all

# Build with all features
cargo build --all-features

# Run benchmarks (if available)
cargo bench
```

## Best Practices

1. **Run checks frequently** - Don't wait until PR time
2. **Fix issues immediately** - Address problems as they appear
3. **Automated CI** - These checks should run in CI/CD pipeline
4. **Document decisions** - Comment why clippy warnings are allowed
5. **Test on clean checkout** - Verify build works from fresh clone

## Exit Criteria

Code is ready for merge when:
- ✅ All tests pass
- ✅ Zero clippy warnings
- ✅ Code formatted correctly
- ✅ Documentation builds
- ✅ Integration test succeeds
- ✅ No security vulnerabilities
- ✅ Conventional commits used

## Notes

- MSRV is Rust 1.92 - verify compatibility
- Use `--release` builds for performance testing
- Integration tests use real SDTM standards from `standards/`
- Validation gating should work correctly (errors block XPT)
