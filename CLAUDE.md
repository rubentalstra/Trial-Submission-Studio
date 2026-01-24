# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

---

## STOP. READ THIS FIRST.

### THE #1 RULE: MANDATORY DELIBERATION

Before writing, editing, or modifying ANY code - even a single character:

1. **Identify 2-3 different approaches**
2. **List pros and cons for each**
3. **Present your analysis**
4. **Wait for explicit approval**

**NO EXCEPTIONS.** Not for "obvious" fixes. Not for typos. NOTHING.

If you find yourself saying "Let me fix this..." and reaching for an edit tool - **STOP**.
You are about to violate this rule.

See `.claude/rules/00-deliberation.md` for the full process and real examples of failures.

---

## Critical Decisions - Ask First

IMPORTANT: Before making these changes, STOP and ask for approval:

- **Dependencies**: Adding, removing, or upgrading any crate in Cargo.toml
- **Standards/Validation**: Any changes to CDISC validation logic or standards interpretation
- **Architecture patterns**: Changes to Elm architecture (state/message/update/view)
- **Public APIs**: Breaking changes to function signatures or types
- **Persistence format**: Any changes to .tss file structure

---

## Workflow Expectations

- **DELIBERATE FIRST**: Before ANY code change, present 2-3 approaches with pros/cons
- **No quick fixes**: The "obvious" solution is often wrong. Always consider alternatives
- **Present options**: Even if you have a recommendation, show the alternatives
- **Wait for approval**: Never assume. Always ask "Which approach would you like?"
- **Minimal changes**: Only modify what's explicitly approved
- **Challenge yourself**: If your first instinct is to edit a file, ask "Is there a better architectural solution?"

---

## Available Tools & Plugins

Use these plugins proactively:

| Plugin                | When to Use                                               |
|-----------------------|-----------------------------------------------------------|
| `/feature-dev`        | New features - starts with requirements gathering         |
| `context7`            | Look up current docs for Iced, Polars, or any dependency  |
| `serena`              | Navigate codebase symbolically (find references, symbols) |
| `rust-analyzer-lsp`   | Get diagnostics and type information                      |
| `/code-simplifier`    | After implementation, simplify code                       |
| `/claude-md-improver` | Periodically audit CLAUDE.md                              |

For feature development, ALWAYS start with `/feature-dev` to gather requirements first.

---

## Project Overview

Trial Submission Studio transforms clinical trial source data (CSV) into FDA-compliant CDISC formats (SDTM, ADaM, SEND).
It's a cross-platform desktop application written in Rust using the Iced GUI framework.

**Status**: Alpha software (v0.0.4-alpha). Not for production regulatory submissions.

---

## Key Files

| File                                | Purpose                              |
|-------------------------------------|--------------------------------------|
| `crates/tss-gui/src/main.rs`        | Application entry point              |
| `crates/tss-gui/src/app.rs`         | Main App struct, update() and view() |
| `crates/tss-gui/src/state/mod.rs`   | AppState definition                  |
| `crates/tss-gui/src/message/mod.rs` | Message enum definitions             |
| `crates/tss-submit/src/lib.rs`      | Submission pipeline entry            |
| `crates/tss-standards/src/lib.rs`   | Standards registry entry             |

---

## Build Commands

```bash
# Build all crates
cargo build

# Run the GUI application
cargo run --package tss-gui

# Run all tests
cargo test

# Run tests for a specific crate
cargo test --package tss-submit

# Run lints
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Regenerate third-party licenses (when dependencies change)
cargo install cargo-about
cargo about generate about.hbs -o THIRD_PARTY_LICENSES.md
```

**Requirements**: Rust 1.92+ (managed by `rust-toolchain.toml`)

---

## Architecture

### Crate Structure

The workspace contains 7 crates with clear separation of concerns:

| Crate                | Purpose                                                 |
|----------------------|---------------------------------------------------------|
| `tss-gui`            | Desktop application (Iced 0.14.0 with Elm architecture) |
| `tss-submit`         | Mapping, normalization, validation, and export pipeline |
| `tss-ingest`         | CSV discovery and parsing                               |
| `tss-standards`      | CDISC standards loader (embedded, offline-first)        |
| `tss-persistence`    | Project file management (.tss format with rkyv)         |
| `tss-updater`        | Auto-update from GitHub releases                        |
| `tss-updater-helper` | macOS app bundle swap helper                            |

### Data Flow

```
CSV Input -> [tss-ingest] -> [tss-standards] -> [tss-submit] -> [tss-gui] -> Export (XPT/XML)
                                 |                                 |
                            CDISC/CT validation            [tss-persistence]
```

### GUI Architecture (tss-gui)

The GUI follows the **Elm architecture** (State -> Message -> Update -> View):

- **`state/`** - Application state (`AppState`, `ViewState`, `Settings`)
- **`message/`** - Message enums for state transitions
- **`handler/`** - Message handlers organized by feature (`HomeHandler`, `DomainEditorHandler`, etc.)
- **`view/`** - Pure view functions (rendering only, no state mutations)
- **`component/`** - Reusable UI components
- **`service/`** - Background task helpers (preview generation, validation)
- **`theme/`** - Clinical theme system with light/dark modes
- **`menu/`** - Native menu bar (macOS via muda, in-app for Windows/Linux)

Key patterns:

- All state changes happen in `update()` - views are pure functions
- Use `Task::perform` for async operations (no channels/polling)
- Handler pattern: each feature area has a dedicated handler implementing `MessageHandler` trait
- Multi-window support via daemon mode with dialog registry

### Standards System (tss-standards)

CDISC standards are embedded as CSV files in `standards/` for offline operation:

- SDTM-IG v3.4, ADaM-IG v1.3, SEND-IG v3.1.1
- Controlled Terminology (2024-2025 versions)
- Registry pattern for standard lookups

### Persistence (tss-persistence)

Project files use `.tss` format:

- Binary format: `TSS\x01` magic + version + rkyv payload
- Zero-copy deserialization with rkyv
- Auto-save with debounce, SHA-256 change detection
- Atomic writes for data integrity

---

## Key Dependencies

- **Iced 0.14.0** - GUI framework (Elm architecture)
- **Polars** - DataFrame operations (lazy evaluation, expressions)
- **rapidfuzz** - Fuzzy string matching for column mapping
- **rkyv** - Zero-copy serialization for project files
- **quick-xml** - Dataset-XML and Define-XML generation
- **xportrs** - SAS XPT v5/v8 read/write

---

## Coding Conventions

- Use conventional commits: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `perf:`, `chore:`
- Keep business logic out of GUI layer
- Prefer pure functions in mapping and validation
- Standards are embedded locally (no external API calls during validation)
- Address all `cargo clippy` warnings
- Prefer explicit types for public APIs

---

## Development Philosophy

This is **greenfield development** - we are building a new desktop application with no legacy constraints.

### NO LEGACY SUPPORT. EVER.

**ABSOLUTELY FORBIDDEN:**

- **Legacy wrappers** - NEVER wrap old code. Delete it.
- **Backwards compatibility** - NEVER maintain old behavior. Replace it.
- **Deprecation notices** - NEVER deprecate. Just remove.
- **Migration code** - NEVER write migration paths. Just rewrite.
- **Old API preservation** - NEVER keep old signatures "just in case". Delete them.
- **Compatibility shims** - NEVER. EVER.
- **Commented-out old code** - NEVER keep "for reference". Delete it.

**THE ONLY ACCEPTABLE APPROACH: FULL REWRITES.**

There are NO users depending on old behavior. There is NOTHING to be backwards compatible WITH.
If code needs to change, REWRITE IT. Don't patch. Don't wrap. Don't preserve.

### Key Principles

- **No backwards compatibility needed** - break anything that improves the codebase
- **Full rewrites encouraged** - don't patch bad code, replace it
- **Best practices only** - no legacy wrappers, no compatibility shims
- **Clean architecture** - if it's not the best design, change it
- **Zero technical debt** - fix issues properly, not with workarounds

### The Anti-Quick-Fix Rule

This project prioritizes **good decisions over fast fixes**. Claude must:

- Never jump to implementation without presenting alternatives
- Never assume the first solution is correct
- Always ask "Is there a better way?" before any edit
- Present architectural solutions alongside local fixes
- Let the user choose between quick fix and proper solution

**Real example of what NOT to do:**
> "Standards folder not bundled -> Let me modify packaging scripts"

This is wrong because there's a better solution (embedding in binary).
The quick fix creates technical debt. The user should choose.

### Anti-Over-Engineering Rules

- Don't add features beyond what was requested
- Don't refactor code outside the change scope
- Don't add "defensive" error handling for impossible cases
- Don't create abstractions for one-time operations
- If unsure whether something is needed, ask

### Error Handling

- Never use `.unwrap()` in production code (except after explicit validation)
- Use `total_cmp()` for float comparisons (NaN-safe)
- Propagate errors with `?` operator and custom error types
- Log best-effort operation failures with `tracing::warn!`

### Async Patterns

- All blocking I/O must use `tokio::task::spawn_blocking`
- Add timeouts to long-running operations
- Use `Task::perform` for Iced async operations

---

## Directory Structure

```
crates/          # Rust workspace crates
standards/       # Embedded CDISC standards (SDTM, ADaM, SEND, CT)
mockdata/        # Test datasets
docs/            # mdBook documentation
scripts/         # Build and utility scripts
resources/       # Asset files (icons, etc.)
```

---

## Gotchas

- **Iced 0.14 breaking changes**: Check `context7` for current Iced API before assuming patterns from older versions
- **Standards are embedded**: Don't try to fetch from external APIs during validation
- **No `cargo run` without `--package`**: Must specify `cargo run --package tss-gui`
- **rkyv versioning**: If persistence format changes, old .tss files may not load
- **macOS code signing**: Release builds require notarization for distribution