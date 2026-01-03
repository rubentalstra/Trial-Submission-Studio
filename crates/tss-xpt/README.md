# sdtm-xpt

A Rust library for reading and writing SAS Transport (XPT) files, commonly used for SDTM datasets in FDA regulatory
submissions.

## Features

- **Full V5 and V8 format support** - Read and write both SAS Transport V5 (legacy) and V8 (extended) formats
- **FDA compliance validation** - Built-in validation rules with collect-all-errors pattern
- **Streaming I/O** - Process large files (>1GB) with constant memory usage
- **IEEE ↔ IBM float conversion** - Accurate floating-point conversion for mainframe compatibility
- **All 28 SAS missing value codes** - Support for `.`, `._`, and `.A` through `.Z`
- **Builder pattern with validation** - Validate before write to catch all errors upfront
- **Optional Polars integration** - Direct DataFrame conversion with the `polars` feature

## Format Versions

| Feature        | V5 Limit | V8 Limit  |
|----------------|----------|-----------|
| Variable name  | 8 chars  | 32 chars  |
| Variable label | 40 chars | 256 chars |
| Format name    | 8 chars  | 32 chars  |
| Dataset name   | 8 chars  | 32 chars  |

By default, files are written in V5 format for maximum compatibility with FDA submissions.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sdtm-xpt = "0.1"

# Optional: Enable Polars DataFrame integration
sdtm-xpt = { version = "0.1", features = ["polars"] }
```

## Quick Start

### Reading an XPT File

```rust
use std::path::Path;
use sdtm_xpt::read_xpt;

let dataset = read_xpt(Path::new("dm.xpt")) ?;

println!("Dataset: {} ({} rows)", dataset.name, dataset.num_rows());

for column in & dataset.columns {
println!("  {} ({:?})", column.name, column.data_type);
}
```

### Writing an XPT File

```rust
use std::path::Path;
use sdtm_xpt::{XptDataset, XptColumn, XptValue, write_xpt};

let mut dataset = XptDataset::with_columns("DM", vec![
    XptColumn::character("USUBJID", 20).with_label("Unique Subject ID"),
    XptColumn::numeric("AGE").with_label("Age in Years"),
]);

dataset.add_row(vec![
    XptValue::character("STUDY-001"),
    XptValue::numeric(35.0),
]);

write_xpt(Path::new("dm.xpt"), & dataset) ?;
```

### FDA-Compliant Writing with Validation

```rust
use std::path::Path;
use sdtm_xpt::{XptDataset, XptColumn, XptWriterBuilder};

let dataset = XptDataset::with_columns("DM", vec![
    XptColumn::character("USUBJID", 20),
    XptColumn::numeric("AGE"),
]);

let writer = XptWriterBuilder::new()
.fda_compliant()  // Enforces V5 format + FDA rules
.validate( & dataset);

if writer.is_valid() {
writer.write_to_file(Path::new("dm.xpt"), & dataset) ?;
} else {
for error in writer.errors() {
eprintln ! ("Validation error: {}", error);
}
}
```

### Streaming Large Files

```rust
use std::path::Path;
use sdtm_xpt::read_xpt_streaming;

let mut reader = read_xpt_streaming(Path::new("large_dataset.xpt")) ?;

println!("Dataset: {}", reader.dataset_name());
println!("Columns: {}", reader.num_columns());

// Process observations one at a time (constant memory)
for observation in reader.observations() {
let obs = observation ?;
// Process each row...
}
```

### Working with Missing Values

SAS supports 28 different missing value codes:

```rust
use sdtm_xpt::{XptValue, MissingValue};

// Standard missing (.)
let missing = XptValue::numeric_missing();

// Special missing codes (.A through .Z)
let missing_a = XptValue::numeric_missing_with(MissingValue::Special('A'));
let missing_z = XptValue::numeric_missing_with(MissingValue::Special('Z'));

// Underscore missing (._)
let missing_underscore = XptValue::numeric_missing_with(MissingValue::Underscore);

// Check for missing
assert!(missing.is_missing());
```

### Polars DataFrame Integration

With the `polars` feature enabled:

```rust
use std::path::Path;
use sdtm_xpt::{read_xpt_to_dataframe, write_dataframe_to_xpt};
use polars::prelude::*;

// Read XPT directly to DataFrame
let df = read_xpt_to_dataframe(Path::new("dm.xpt")) ?;
println!("{}", df);

// Write DataFrame to XPT
let df = df! {
    "USUBJID" => &["001", "002", "003"],
    "AGE" => &[25i64, 30, 35],
}?;

write_dataframe_to_xpt(Path::new("output.xpt"), & df, "DM") ?;
```

## API Overview

### Core Types

| Type           | Description                                        |
|----------------|----------------------------------------------------|
| `XptDataset`   | A complete dataset with columns and rows           |
| `XptColumn`    | Column metadata (name, type, label, format)        |
| `XptValue`     | A single value (numeric or character)              |
| `NumericValue` | Numeric value with missing value support           |
| `MissingValue` | Missing value type (Standard, Underscore, Special) |
| `XptVersion`   | Format version (V5 or V8)                          |

### Reader Functions

| Function                               | Description                             |
|----------------------------------------|-----------------------------------------|
| `read_xpt(path)`                       | Read entire XPT file into memory        |
| `read_xpt_streaming(path)`             | Create streaming reader for large files |
| `read_xpt_with_options(path, options)` | Read with custom options                |

### Writer Functions

| Function                                         | Description                            |
|--------------------------------------------------|----------------------------------------|
| `write_xpt(path, dataset)`                       | Write dataset to XPT file (V5 default) |
| `write_xpt_with_options(path, dataset, options)` | Write with custom options              |
| `XptWriterBuilder::new()`                        | Create builder for validated writing   |

### Validation

The library provides comprehensive validation:

```rust
use sdtm_xpt::{XptDataset, XptVersion};
use sdtm_xpt::validation::{Validator, ValidationMode};

let dataset = XptDataset::new("DM");
let validator = Validator::new(XptVersion::V5)
.with_mode(ValidationMode::FdaCompliant);

let result = validator.validate( & dataset);

for error in & result.errors {
println!("Error [{}]: {}", error.code, error.message);
}

for warning in & result.warnings {
println!("Warning: {}", warning.message);
}
```

## Error Handling

All errors are consolidated into a single `XptError` type:

```rust
use sdtm_xpt::{read_xpt, XptError};

match read_xpt(Path::new("file.xpt")) {
Ok(dataset) => println ! ("Loaded {} rows", dataset.num_rows()),
Err(XptError::FileNotFound { path }) => {
eprintln ! ("File not found: {}", path.display());
}
Err(XptError::InvalidHeader { expected }) => {
eprintln ! ("Invalid header: expected {}", expected);
}
Err(XptError::Validation(errors)) => {
for e in errors {
eprintln ! ("Validation: {}", e);
}
}
Err(e) => eprintln ! ("Error: {}", e),
}
```

## FDA Submission Requirements

For FDA regulatory submissions, use the FDA-compliant mode:

```rust
let writer = XptWriterBuilder::new()
.fda_compliant()
.validate( & dataset);
```

This enforces:

- V5 format (required by FDA)
- 8-character variable names
- 40-character labels
- ASCII-only data
- Single dataset per file

## Testing

The crate includes comprehensive tests:

```bash
# Run all tests
cargo test -p sdtm-xpt

# Run with Polars feature
cargo test -p sdtm-xpt --all-features

# Run only roundtrip integration tests
cargo test -p sdtm-xpt --test roundtrip
```

## License

MIT
