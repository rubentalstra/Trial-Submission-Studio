# tss-validate

CDISC conformance validation crate.

## Overview

`tss-validate` checks data against SDTM implementation guide rules and controlled terminology.

## Responsibilities

- Structural validation (required variables, types)
- Content validation (controlled terminology, formats)
- Cross-record validation (relationships, duplicates)
- Generate validation reports

## Dependencies

```toml
[dependencies]
tss-standards = { path = "../tss-standards" }
tss-model = { path = "../tss-model" }
regex = "1"
chrono = "0.4"
```

## Architecture

### Module Structure

```
tss-validate/
├── src/
│   ├── lib.rs
│   ├── engine.rs        # Validation orchestration
│   ├── rules/
│   │   ├── mod.rs
│   │   ├── structural.rs   # Structure rules
│   │   ├── content.rs      # Value rules
│   │   ├── terminology.rs  # CT validation
│   │   └── cross_record.rs # Relationship rules
│   ├── result.rs        # Validation results
│   └── report.rs        # Report generation
```

## Validation Engine

### Rule Interface

```rust
pub trait ValidationRule {
    fn id(&self) -> &str;
    fn severity(&self) -> Severity;
    fn validate(&self, context: &ValidationContext) -> Vec<ValidationResult>;
}
```

### Severity Levels

```rust
pub enum Severity {
    Error,    // Blocks export
    Warning,  // Should review
    Info,     // Informational
}
```

### Validation Context

```rust
pub struct ValidationContext<'a> {
    pub domain: &'a str,
    pub data: &'a DataFrame,
    pub mappings: &'a [Mapping],
    pub standards: &'a Standards,
}
```

## Built-in Rules

### Structural Rules (SD*)

| Rule   | Description               |
|--------|---------------------------|
| SD0001 | Required variable missing |
| SD0002 | Invalid variable name     |
| SD0003 | Variable length exceeded  |
| SD0004 | Invalid data type         |

### Terminology Rules (CT*)

| Rule   | Description           |
|--------|-----------------------|
| CT0001 | Value not in codelist |
| CT0002 | Invalid date format   |
| CT0003 | Date out of range     |

### Cross-Record Rules (XR*)

| Rule   | Description          |
|--------|----------------------|
| XR0001 | USUBJID not in DM    |
| XR0002 | Duplicate key values |

## API

### Running Validation

```rust
use tss_validate::{Validator, ValidationContext};

let validator = Validator::new( & standards);
let results = validator.validate( & context) ?;

for result in results.errors() {
    println!("[{:?}] {}", result.severity(), result.message());
}
```

### Custom Rules

```rust
struct MyCustomRule;

impl ValidationRule for MyCustomRule {
    fn id(&self) -> &str { "CUSTOM001" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn validate(&self, ctx: &ValidationContext) -> Vec<ValidationResult> {
        // Custom logic
    }
}
```

## Testing

```bash
cargo test --package tss-validate
```

### Test Strategy

- Unit tests for each rule
- Integration tests with sample data
- Property tests for edge cases

## See Also

- [Validation](../../user-guide/validation.md) - User guide
- [Validation Rules](../../cdisc-standards/sdtm/validation-rules.md) - Rule reference
- [tss-standards](tss-standards.md) - Standards data
