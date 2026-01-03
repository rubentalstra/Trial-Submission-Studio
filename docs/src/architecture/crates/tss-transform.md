# tss-transform

Data transformation crate for CDISC conversions.

## Overview

`tss-transform` applies transformations to convert source data to SDTM-compliant format.

## Responsibilities

- Apply column mappings
- Transform data values
- Derive computed variables
- Handle date conversions
- Apply controlled terminology mappings

## Dependencies

```toml
[dependencies]
polars = { version = "0.44", features = ["lazy"] }
chrono = "0.4"
tss-model = { path = "../tss-model" }
tss-standards = { path = "../tss-standards" }
```

## Architecture

### Module Structure

```
tss-transform/
├── src/
│   ├── lib.rs
│   ├── engine.rs        # Transformation orchestration
│   ├── transforms/
│   │   ├── mod.rs
│   │   ├── column.rs    # Column renaming/reordering
│   │   ├── value.rs     # Value transformations
│   │   ├── date.rs      # Date conversions
│   │   ├── derive.rs    # Computed variables
│   │   └── terminology.rs # CT mappings
│   └── pipeline.rs      # Transform pipeline
```

## Transformation Types

### Column Transforms

```rust
pub enum ColumnTransform {
    Rename { from: String, to: String },
    Drop { column: String },
    Reorder { order: Vec<String> },
}
```

### Value Transforms

```rust
pub enum ValueTransform {
    Map { column: String, mappings: HashMap<String, String> },
    Format { column: String, pattern: String },
    Uppercase { column: String },
    Trim { column: String },
}
```

### Date Transforms

```rust
pub enum DateTransform {
    ToIso8601 { column: String, input_format: String },
    ExtractDate { column: String },
    ExtractTime { column: String },
}
```

### Derivations

```rust
pub enum Derivation {
    Sequence { column: String, group_by: Vec<String> },
    Constant { column: String, value: String },
    Concat { column: String, sources: Vec<String>, sep: String },
}
```

## API

### Building a Pipeline

```rust
use tss_transform::{Pipeline, Transform};

let pipeline = Pipeline::new()
.add(Transform::rename("SUBJECT_ID", "USUBJID"))
.add(Transform::map_values("SEX", vec![
    ("Male", "M"),
    ("Female", "F"),
]))
.add(Transform::to_iso8601("VISIT_DATE", "%m/%d/%Y"))
.add(Transform::derive_sequence("AESEQ", & ["USUBJID"]));

let result = pipeline.apply( & dataframe) ?;
```

### Transformation Context

```rust
pub struct TransformContext<'a> {
    pub source_data: &'a DataFrame,
    pub mappings: &'a [Mapping],
    pub domain: &'a DomainDefinition,
    pub options: TransformOptions,
}
```

## Date Handling

### Supported Input Formats

| Format    | Example             |
|-----------|---------------------|
| ISO 8601  | 2024-01-15          |
| US        | 01/15/2024          |
| EU        | 15-01-2024          |
| With time | 2024-01-15T09:30:00 |

### Output Format

Always ISO 8601:

- Full: `YYYY-MM-DDTHH:MM:SS`
- Date only: `YYYY-MM-DD`
- Partial: `YYYY-MM` or `YYYY`

## Testing

```bash
cargo test --package tss-transform
```

### Test Strategy

- Unit tests for each transform type
- Integration tests with sample data
- Snapshot tests for output consistency

## See Also

- [Column Mapping](../../user-guide/column-mapping.md) - Mapping workflow
- [tss-map](tss-map.md) - Mapping engine
- [tss-validate](tss-validate.md) - Validation after transform
