# tss-model

Core domain types crate.

## Overview

`tss-model` defines the fundamental data structures used across all crates.

## Responsibilities

- Define core data types
- Provide serialization/deserialization
- Ensure type consistency across crates

## Dependencies

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Architecture

### Module Structure

```
tss-model/
├── src/
│   ├── lib.rs
│   ├── domain.rs        # Domain types
│   ├── variable.rs      # Variable types
│   ├── mapping.rs       # Mapping types
│   ├── validation.rs    # Validation types
│   └── metadata.rs      # Metadata types
```

## Core Types

### Domain Types

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DomainClass {
    SpecialPurpose,
    Interventions,
    Events,
    Findings,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub code: String,
    pub name: String,
    pub class: DomainClass,
    pub description: String,
}
```

### Variable Types

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Char,
    Num,
    Date,
    DateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Core {
    Required,
    Expected,
    Permissible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub label: String,
    pub data_type: DataType,
    pub length: Option<usize>,
    pub core: Core,
    pub codelist: Option<String>,
}
```

### Mapping Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    pub source_column: String,
    pub target_variable: String,
    pub confidence: f64,
    pub transform: Option<Transform>,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transform {
    Rename,
    ValueMap(HashMap<String, String>),
    DateFormat(String),
    Uppercase,
    Trim,
}
```

### Validation Types

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub location: Option<Location>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub row: Option<usize>,
    pub column: Option<String>,
}
```

### Metadata Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub name: String,
    pub label: String,
    pub domain: String,
    pub structure: String,
    pub variables: Vec<VariableMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableMetadata {
    pub name: String,
    pub label: String,
    pub data_type: DataType,
    pub length: usize,
    pub origin: Origin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Origin {
    Crf,
    Derived,
    Assigned,
    Protocol,
}
```

## Design Principles

### Immutability

Types are designed to be cloned rather than mutated:

```rust
let updated = Mapping {
confirmed: true,
..original
};
```

### Serialization

All types derive `Serialize` and `Deserialize` for:

- Configuration storage
- State persistence
- Debug output

### Equality

Types implement `PartialEq` for:

- Testing
- Deduplication
- Change detection

## Testing

```bash
cargo test --package tss-model
```

### Test Focus

- Serialization roundtrip
- Type conversions
- Default values

## See Also

- [Architecture Overview](../overview.md) - System design
- Other crate documentation for usage examples
