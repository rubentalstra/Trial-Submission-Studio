# Development Setup

Set up your development environment for contributing to Trial Submission Studio.

## Prerequisites

### Required

| Tool | Version    | Purpose              |
|------|------------|----------------------|
| Rust | 1.92+      | Programming language |
| Git  | Any recent | Version control      |

### Optional

| Tool        | Purpose                 |
|-------------|-------------------------|
| cargo-about | License generation      |
| cargo-watch | Auto-rebuild on changes |

## Step 1: Fork and Clone

### Fork on GitHub

1. Go to [Trial Submission Studio](https://github.com/rubentalstra/Trial-Submission-Studio)
2. Click "Fork" in the top right
3. Select your account

### Clone Your Fork

```bash
git clone https://github.com/YOUR_USERNAME/trial-submission-studio.git
cd trial-submission-studio
```

### Add Upstream Remote

```bash
git remote add upstream https://github.com/rubentalstra/Trial-Submission-Studio.git
```

## Step 2: Install Rust

### Using rustup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Verify Installation

```bash
rustup show
```

Expected output should show Rust 1.92 or higher.

### Install Required Toolchain

```bash
rustup toolchain install stable
rustup component add rustfmt clippy
```

## Step 3: Platform Dependencies

### macOS

No additional dependencies required.

### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y libgtk-3-dev libxdo-dev
```

### Windows

No additional dependencies required.

## Step 4: Build the Project

### Debug Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

### Check Build

```bash
cargo check
```

## Step 5: Run the Application

```bash
cargo run --package tss-gui
```

## Step 6: Run Tests

```bash
# All tests
cargo test

# Specific crate
cargo test --package xport

# With output
cargo test -- --nocapture
```

## Step 7: Run Lints

```bash
# Format check
cargo fmt --check

# Apply formatting
cargo fmt

# Clippy lints
cargo clippy -- -D warnings
```

## IDE Setup

### RustRover / IntelliJ IDEA

1. Open the project folder
2. Rust plugin auto-detects workspace
3. Configure run configuration for `tss-gui`

### VS Code

1. Install `rust-analyzer` extension
2. Open the project folder
3. Extension auto-configures

### Recommended VS Code Extensions

- rust-analyzer
- Even Better TOML
- Error Lens
- GitLens

## Project Structure

```
trial-submission-studio/
├── Cargo.toml              # Workspace config
├── crates/                 # All crates
│   ├── tss-gui/           # Main application
│   ├── xport/             # XPT I/O
│   ├── tss-validate/      # Validation
│   └── ...                # Other crates
├── standards/             # Embedded CDISC data
├── mockdata/              # Test data
└── docs/                  # Documentation
```

## Development Workflow

### Create Feature Branch

```bash
git checkout main
git pull upstream main
git checkout -b feature/my-feature
```

### Make Changes

1. Edit code
2. Run tests: `cargo test`
3. Run lints: `cargo clippy`
4. Format: `cargo fmt`

### Commit Changes

```bash
git add .
git commit -m "feat: add my feature"
```

### Push and Create PR

```bash
git push origin feature/my-feature
```

Then create PR on GitHub.

## Useful Commands

| Command                  | Purpose             |
|--------------------------|---------------------|
| `cargo build`            | Build debug         |
| `cargo build --release`  | Build release       |
| `cargo test`             | Run all tests       |
| `cargo test --package X` | Test specific crate |
| `cargo clippy`           | Run linter          |
| `cargo fmt`              | Format code         |
| `cargo doc --open`       | Generate docs       |
| `cargo run -p tss-gui`   | Run application     |

## Troubleshooting

### Build Fails

1. Ensure Rust 1.92+: `rustup update stable`
2. Clean build: `cargo clean && cargo build`
3. Check dependencies: `cargo fetch`

### Tests Fail

1. Run with output: `cargo test -- --nocapture`
2. Run specific test: `cargo test test_name`
3. Check test data in `mockdata/`

### GUI Won't Start

1. Check platform dependencies installed
2. Try release build: `cargo run --release -p tss-gui`
3. Check logs for errors

## Next Steps

- [Coding Standards](coding-standards.md) - Style guide
- [Testing](testing.md) - Testing guide
- [Architecture](../architecture/overview.md) - Understand the codebase
