# Building from Source

For developers who want to compile Trial Submission Studio from source code.

## Prerequisites

### Required

- **Rust 1.92+** - Install via [rustup](https://rustup.rs/)
- **Git** - For cloning the repository

### Platform-Specific Dependencies

#### macOS

No additional dependencies required.

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get install libgtk-3-dev libxdo-dev
```

#### Windows

No additional dependencies required.

## Clone the Repository

```bash
git clone https://github.com/rubentalstra/trial-submission-studio.git
cd trial-submission-studio
```

## Verify Rust Version

```bash
rustup show
```

Ensure you have Rust 1.92 or higher. To update:

```bash
rustup update stable
```

## Build

### Debug Build (faster compilation)

```bash
cargo build
```

### Release Build (optimized, slower compilation)

```bash
cargo build --release
```

## Run

### Debug

```bash
cargo run --package tss-gui
```

### Release

```bash
cargo run --release --package tss-gui
```

Or run the compiled binary directly:

```bash
./target/release/tss-gui        # macOS/Linux
.\target\release\tss-gui.exe    # Windows
```

## Run Tests

```bash
# All tests
cargo test

# Specific crate
cargo test --package tss-xpt

# With output
cargo test -- --nocapture
```

## Run Lints

```bash
# Format check
cargo fmt --check

# Clippy lints
cargo clippy -- -D warnings
```

## Project Structure

Trial Submission Studio is organized as a Rust workspace with multiple crates:

```
trial-submission-studio/
├── crates/
│   ├── tss-gui/          # Desktop application
│   ├── tss-xpt/          # XPT file I/O
│   ├── tss-validate/     # CDISC validation
│   ├── tss-map/          # Column mapping
│   ├── tss-transform/    # Data transformations
│   ├── tss-ingest/       # CSV loading
│   ├── tss-output/       # Multi-format export
│   ├── tss-standards/    # CDISC standards loader
│   ├── tss-model/        # Core types
│   ├── tss-common/       # Shared utilities
│   └── tss-updater/      # Update mechanism
├── standards/            # Embedded CDISC standards
├── mockdata/             # Test datasets
└── docs/                 # Documentation (this site)
```

## Third-Party Licenses

When adding or updating dependencies, regenerate the licenses file:

```bash
# Install cargo-about (one-time)
cargo install cargo-about

# Generate licenses
cargo about generate about.hbs -o THIRD_PARTY_LICENSES.md
```

## IDE Setup

### RustRover / IntelliJ IDEA

1. Open the project folder
2. The Rust plugin will detect the workspace automatically

### VS Code

1. Install the `rust-analyzer` extension
2. Open the project folder

## Next Steps

- [Contributing Guide](../contributing/getting-started.md) - How to contribute
- [Architecture Overview](../architecture/overview.md) - Understand the codebase
