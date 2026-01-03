# tss-common

Shared utilities crate.

## Overview

`tss-common` provides utilities and helpers used across multiple crates.

## Responsibilities

- Logging and tracing setup
- Error handling utilities
- Common string operations
- File system helpers

## Dependencies

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "2"
```

## Architecture

### Module Structure

```
tss-common/
├── src/
│   ├── lib.rs
│   ├── logging.rs       # Logging setup
│   ├── error.rs         # Error types
│   ├── string.rs        # String utilities
│   └── fs.rs            # File system helpers
```

## Logging

### Setup

```rust
use tss_common::logging;

pub fn init_logging() {
    logging::init(logging::Level::Info);
}
```

### Usage

```rust
use tracing::{info, warn, error, debug};

info!("Processing file: {}", filename);
warn!("Large dataset detected: {} rows", row_count);
error!("Failed to parse: {}", error);
debug!("Internal state: {:?}", state);
```

### Log Levels

| Level | Usage                             |
|-------|-----------------------------------|
| Error | Failures that need user attention |
| Warn  | Potential issues, recoverable     |
| Info  | Normal operation progress         |
| Debug | Development/troubleshooting       |
| Trace | Detailed internal state           |

## Error Handling

### Error Type

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TssError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

### Result Type

```rust
pub type TssResult<T> = Result<T, TssError>;
```

### Usage

```rust
use tss_common::{TssError, TssResult};

fn process_file(path: &Path) -> TssResult<Data> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| TssError::FileNotFound(path.display().to_string()))?;

    parse_content(&content)
}
```

## String Utilities

### Normalization

```rust
pub fn normalize_column_name(name: &str) -> String {
    name.trim()
        .to_uppercase()
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}
```

### Truncation

```rust
pub fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
```

## File System Helpers

### Path Operations

```rust
pub fn ensure_extension(path: &Path, ext: &str) -> PathBuf {
    let mut path = path.to_path_buf();
    path.set_extension(ext);
    path
}

pub fn unique_filename(dir: &Path, base: &str, ext: &str) -> PathBuf {
    // Generate unique filename if exists
}
```

### Safe Operations

```rust
pub fn safe_write(path: &Path, content: &[u8]) -> TssResult<()> {
    // Write to temp file, then rename (atomic)
}
```

## Testing

```bash
cargo test --package tss-common
```

## See Also

- [Architecture Overview](../overview.md) - System design
- Individual crate docs for usage context
