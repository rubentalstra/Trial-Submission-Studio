---
paths:
  - "**/Cargo.toml"
---

# Dependency Management Rules

## ALWAYS Ask Before

- Adding new dependencies
- Removing dependencies
- Upgrading dependency versions
- Changing feature flags

## Process

1. Explain why the dependency is needed
2. Check if existing dependencies can solve the problem
3. Consider bundle size and compile time impact
4. Get explicit approval before modifying Cargo.toml

## After Changes

Run:
```bash
cargo build
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```