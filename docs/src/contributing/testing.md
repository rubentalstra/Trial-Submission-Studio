# Testing

Testing guidelines for Trial Submission Studio contributions.

## Test Types

### Unit Tests

Test individual functions and methods:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_column_name_removes_spaces() {
        let result = normalize_column_name("Patient Age");
        assert_eq!(result, "PATIENT_AGE");
    }
}
```

### Integration Tests

Test interactions between modules:

```rust
// tests/integration_test.rs
use tss_ingest::CsvReader;
use tss_validate::Validator;

#[test]
fn validate_imported_data() {
    let data = CsvReader::read("tests/data/sample.csv").unwrap();
    let results = Validator::validate(&data, "DM").unwrap();
    assert!(results.errors().is_empty());
}
```

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Crate

```bash
cargo test --package tss-output
```

### Specific Test

```bash
cargo test test_name
```

### With Output

```bash
cargo test -- --nocapture
```

### Release Mode

```bash
cargo test --release
```

## Test Organization

### File Structure

```
crates/tss-validate/
├── src/
│   ├── lib.rs
│   └── rules/
│       └── structural.rs
└── tests/
    ├── structural_rules_test.rs
    └── data/
        └── sample_dm.csv
```

### Inline Tests

For simple unit tests:

```rust
// src/normalize.rs

pub fn normalize(s: &str) -> String {
    s.trim().to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("  hello  "), "HELLO");
    }
}
```

### External Tests

For integration tests:

```rust
// tests/validation_integration.rs

use tss_validate::*;

#[test]
fn full_validation_workflow() {
    // Integration test code
}
```

## Test Data

### Location

Test data files are in:

- `mockdata/` - Shared test datasets
- `crates/*/tests/data/` - Crate-specific test data

### Sample Data

```csv
STUDYID,DOMAIN,USUBJID,SUBJID,AGE,SEX
ABC123,DM,ABC123-001,001,45,M
ABC123,DM,ABC123-002,002,38,F
```

### Sensitive Data

Never commit real clinical trial data. Use:

- Synthetic/mock data only
- Anonymized examples
- Generated test cases

## Writing Good Tests

### Structure (AAA Pattern)

```rust
#[test]
fn test_validation_rule() {
    // Arrange - set up test data
    let data = create_test_dataframe();
    let validator = Validator::new();

    // Act - perform the operation
    let results = validator.validate(&data);

    // Assert - verify results
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].severity, Severity::Error);
}
```

### Descriptive Names

```rust
// Good
#[test]
fn returns_error_when_usubjid_is_missing() { ... }

#[test]
fn accepts_valid_iso8601_date_format() { ... }

// Avoid
#[test]
fn test1() { ... }

#[test]
fn it_works() { ... }
```

### Test Edge Cases

```rust
#[test]
fn handles_empty_dataframe() { ... }

#[test]
fn handles_null_values() { ... }

#[test]
fn handles_unicode_characters() { ... }

#[test]
fn handles_maximum_length_values() { ... }
```

### Test Error Conditions

```rust
#[test]
fn returns_error_for_invalid_input() {
    let result = process_file("nonexistent.csv");
    assert!(result.is_err());
}

#[test]
fn error_contains_helpful_message() {
    let err = process_file("bad.csv").unwrap_err();
    assert!(err.to_string().contains("parse error"));
}
```

## CI Testing

### Automated Checks

Every PR runs:

1. `cargo test` - All tests
2. `cargo clippy` - Linting
3. `cargo fmt --check` - Formatting

### Test Matrix

Tests run on:

- Ubuntu (primary)
- macOS (future)
- Windows (future)

## Test Coverage

### Goal

Aim for high coverage on critical paths:

- Validation rules
- Data transformations
- File I/O

### Not Required

100% coverage isn't required. Focus on:

- Business logic
- Error handling
- Edge cases

## Next Steps

- [Pull Requests](pull-requests.md) - Submit your changes
- [Coding Standards](coding-standards.md) - Code style
