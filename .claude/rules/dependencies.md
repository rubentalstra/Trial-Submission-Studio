---
paths:
  - "**/Cargo.toml"
---

# Dependency Management Rules

## MANDATORY: Deliberation First

When a dependency-related change seems needed:

1. **Explain the actual need** - What problem are we solving?
2. **Consider alternatives:**
   - Can existing dependencies solve this?
   - Can we implement it ourselves (if small)?
   - Are there lighter-weight alternatives?
   - Is the dependency even necessary?
3. **Present trade-offs** for each option
4. **Recommend one** with clear reasoning
5. **Wait for explicit approval**

Never say "I need to add crate X" and start editing Cargo.toml.
**Always present options first.**

---

## ALWAYS Ask Before

- Adding new dependencies
- Removing dependencies
- Upgrading dependency versions
- Changing feature flags

---

## Process

1. Explain why the dependency is needed
2. Check if existing dependencies can solve the problem
3. Consider bundle size and compile time impact
4. Get explicit approval before modifying Cargo.toml

---

## After Changes

Run:

```bash
cargo build
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

---

## Common Questions to Consider

1. **Do we really need this?** Many crates add unnecessary dependencies.
2. **Is there a lighter alternative?** Check crates.io for minimal implementations.
3. **What's the maintenance status?** Avoid unmaintained crates.
4. **What's the license?** Ensure compatibility with our project.