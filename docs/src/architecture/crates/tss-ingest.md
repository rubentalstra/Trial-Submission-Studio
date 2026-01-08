# tss-ingest

CSV ingestion and schema detection crate.
 
## Overview

`tss-ingest` handles loading source data files and detecting their schema.

## Responsibilities

- CSV file parsing
- Schema detection (types, formats)
- Domain suggestion
- Data preview generation

## Dependencies

```toml
[dependencies]
csv = "1.3"
polars = { version = "0.44", features = ["lazy", "csv"] }
encoding_rs = "0.8"
tss-model = { path = "../tss-model" }
```

## Architecture

### Module Structure

```
tss-ingest/
├── src/
│   ├── lib.rs
│   ├── reader.rs        # CSV reading
│   ├── schema.rs        # Schema detection
│   ├── types.rs         # Type inference
│   ├── domain.rs        # Domain suggestion
│   └── preview.rs       # Data preview
```

## Schema Detection

### Type Inference

```rust
pub enum InferredType {
    Integer,
    Float,
    Date(String),      // With format pattern
    DateTime(String),
    Boolean,
    Text,
}
```

### Detection Algorithm

1. Sample first N rows
2. For each column:
    - Try parsing as integer
    - Try parsing as float
    - Try common date formats
    - Default to text

### Date Format Detection

| Pattern             | Example             |
|---------------------|---------------------|
| `%Y-%m-%d`          | 2024-01-15          |
| `%m/%d/%Y`          | 01/15/2024          |
| `%d-%m-%Y`          | 15-01-2024          |
| `%Y-%m-%dT%H:%M:%S` | 2024-01-15T09:30:00 |

## API

### Loading a File

```rust
use tss_ingest::{CsvReader, IngestOptions};

let options = IngestOptions {
encoding: Some("utf-8"),
sample_rows: 1000,
..Default::default ()
};

let result = CsvReader::read("data.csv", options) ?;
println!("Rows: {}", result.row_count);
println!("Columns: {:?}", result.schema.columns);
```

### Schema Result

```rust
pub struct IngestResult {
    pub data: DataFrame,
    pub schema: DetectedSchema,
    pub suggested_domain: Option<String>,
    pub warnings: Vec<IngestWarning>,
}

pub struct DetectedSchema {
    pub columns: Vec<ColumnInfo>,
}

pub struct ColumnInfo {
    pub name: String,
    pub inferred_type: InferredType,
    pub null_count: usize,
    pub sample_values: Vec<String>,
}
```

## Domain Suggestion

Based on column names, suggest likely SDTM domain:

| Column Patterns   | Suggested Domain |
|-------------------|------------------|
| USUBJID, AGE, SEX | DM               |
| AETERM, AESTDTC   | AE               |
| VSTESTCD, VSORRES | VS               |
| LBTESTCD, LBORRES | LB               |

```rust
pub fn suggest_domain(columns: &[String]) -> Option<String> {
    // Pattern matching logic
}
```

## Error Handling

### Common Issues

| Issue          | Handling                  |
|----------------|---------------------------|
| Encoding error | Try alternative encodings |
| Parse error    | Mark as text, warn user   |
| Empty file     | Return error              |
| No header      | Require user action       |

## Testing

```bash
cargo test --package tss-ingest
```

### Test Files

Located in `mockdata/`:

- Various CSV formats
- Different encodings
- Edge cases

## See Also

- [Importing Data](../../user-guide/importing-data.md) - User guide
- [tss-normalization](tss-normalization.md) - Data transformation
