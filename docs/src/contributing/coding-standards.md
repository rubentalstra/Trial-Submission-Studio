# Coding Standards

Code style and quality guidelines for Trial Submission Studio.

## Rust Style

### Formatting

Use `rustfmt` for all code formatting:

```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt
```

### Linting

All code must pass Clippy with no warnings:

```bash
cargo clippy -- -D warnings
```

## Naming Conventions

### Crates

- Lowercase with hyphens: `tss-xpt`, `tss-validate`
- Prefix with `tss-` for project crates

### Modules

- Lowercase with underscores: `column_mapping.rs`
- Keep names short but descriptive

### Functions

```rust
// Good - descriptive, snake_case
fn calculate_similarity(source: &str, target: &str) -> f64

// Good - verb-noun pattern
fn validate_domain(data: &DataFrame) -> Vec<ValidationResult>

// Avoid - too abbreviated
fn calc_sim(s: &str, t: &str) -> f64
```

### Types

```rust
// Good - PascalCase, descriptive
struct ValidationResult {
    ...
}
enum DomainClass {...}

// Good - clear trait naming
trait ValidationRule { ... }
```

### Constants

```rust
// Good - SCREAMING_SNAKE_CASE
const MAX_VARIABLE_LENGTH: usize = 8;
const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.8;
```

## Code Organization

### File Structure

```rust
// 1. Module documentation
//! Module description

// 2. Imports (grouped)
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::Variable;

// 3. Constants
const DEFAULT_VALUE: i32 = 0;

// 4. Type definitions
pub struct MyStruct {
    ...
}

// 5. Implementations
impl MyStruct { ... }

// 6. Functions
pub fn my_function() { ... }

// 7. Tests (at bottom or in separate file)
#[cfg(test)]
mod tests {
    ...
}
```

### Import Organization

Group imports in this order:

1. Standard library
2. External crates
3. Internal crates
4. Current crate modules

```rust
use std::path::Path;

use polars::prelude::*;
use serde::Serialize;

use tss_model::Variable;

use crate::mapping::Mapping;
```

## Error Handling

### Use Result Types

```rust
// Good - explicit error handling
pub fn parse_file(path: &Path) -> Result<Data, ParseError> {
    let content = std::fs::read_to_string(path)?;
    parse_content(&content)
}

// Avoid - panicking on errors
pub fn parse_file(path: &Path) -> Data {
    let content = std::fs::read_to_string(path).unwrap(); // Don't do this
    parse_content(&content).expect("parse failed") // Or this
}
```

### Custom Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Missing required variable: {0}")]
    MissingVariable(String),

    #[error("Invalid value '{value}' for {variable}")]
    InvalidValue { variable: String, value: String },
}
```

### Error Context

```rust
// Good - add context to errors
fs::read_to_string(path)
.map_err( | e| ParseError::FileRead {
path: path.to_path_buf(),
source: e,
}) ?;
```

## Documentation

### Public Items

All public items must be documented:

```rust
/// Validates data against SDTM rules.
///
/// # Arguments
///
/// * `data` - The DataFrame to validate
/// * `domain` - Target SDTM domain code
///
/// # Returns
///
/// Vector of validation results
///
/// # Example
///
/// ```
/// let results = validate(&data, "DM")?;
/// ```
pub fn validate(data: &DataFrame, domain: &str) -> Result<Vec<ValidationResult>> {
    // ...
}
```

### Module Documentation

```rust
//! CSV ingestion and schema detection.
//!
//! This module provides functionality for loading CSV files
//! and automatically detecting their schema.
```

## Testing

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() {
        // Arrange
        let input = "test";

        // Act
        let result = process(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_edge_case() {
        // ...
    }
}
```

### Test Naming

```rust
// Good - descriptive test names
#[test]
fn parse_iso8601_date_returns_correct_value() { ... }

#[test]
fn validate_returns_error_for_missing_usubjid() { ... }

// Avoid - vague names
#[test]
fn test1() { ... }
```

## Architecture Principles

### Separation of Concerns

- Keep business logic out of GUI code
- I/O operations separate from data processing
- Validation rules independent of data loading

### Pure Functions

Prefer pure functions where possible:

```rust
// Good - pure function, easy to test
pub fn calculate_confidence(source: &str, target: &str) -> f64 {
    // No side effects, deterministic
}

// Use sparingly - side effects
pub fn log_and_calculate(source: &str, target: &str) -> f64 {
    tracing::info!("Calculating..."); // Side effect
    calculate_confidence(source, target)
}
```

### Determinism

Output must be reproducible:

```rust
// Good - deterministic output
pub fn derive_sequence(data: &DataFrame, group_by: &[&str]) -> Vec<i32> {
    // Same input always produces same output
}

// Avoid - non-deterministic
pub fn derive_sequence_random(data: &DataFrame) -> Vec<i32> {
    // Uses random ordering - bad for regulatory compliance
}
```

## Performance

### Avoid Premature Optimization

Write clear code first, optimize if needed based on profiling.

### Use Appropriate Data Structures

```rust
// Good - HashMap for lookups
let lookup: HashMap<String, Variable> =...;

// Good - Vec for ordered data
let results: Vec<ValidationResult> =...;
```

### Lazy Evaluation

Use Polars lazy evaluation for large datasets:

```rust
let result = df.lazy()
.filter(col("value").gt(lit(0)))
.collect() ?;
```

## Next Steps

- [Testing](testing.md) - Testing guidelines
- [Pull Requests](pull-requests.md) - PR process
