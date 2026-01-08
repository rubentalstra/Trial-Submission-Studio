# xport

XPT (SAS Transport) file I/O crate. Designed for standalone use and publishing to crates.io.

## Overview

`xport` provides reading and writing of XPT V5 and V8 format files. It's designed to be used independently of the Trial Submission Studio application for general SAS Transport file handling.

## Features

- Read XPT V5 and V8 format files
- Write XPT V5 and V8 format files
- Handle IBM mainframe to IEEE floating-point conversion
- Support all 28 SAS missing value codes
- Optional Polars DataFrame integration (`polars` feature)
- Optional serde serialization (`serde` feature)

## Dependencies

```toml
[dependencies]
xport = { version = "0.1", features = ["polars"] }  # With DataFrame support
# or
xport = "0.1"  # Core functionality only
```

## Architecture

### Module Structure

```
xport/
├── src/
│   ├── lib.rs
│   ├── reader/       # XPT file reading (streaming)
│   ├── writer/       # XPT file writing (streaming)
│   ├── header/       # Header parsing
│   ├── types/        # Core types (column, value, missing)
│   ├── error/        # Error handling
│   └── version.rs    # V5/V8 version handling
```

## XPT Format Details

### File Structure

```
┌─────────────────────────────────────┐
│ Library Header (80 bytes × 2)       │
├─────────────────────────────────────┤
│ Member Header (80 bytes × 3)        │
├─────────────────────────────────────┤
│ Namestr Records (140 bytes each)    │
│ (one per variable)                  │
├─────────────────────────────────────┤
│ Observation Header (80 bytes)       │
├─────────────────────────────────────┤
│ Data Records                        │
│ (fixed-width, packed)               │
└─────────────────────────────────────┘
```

### Numeric Handling

IBM mainframe to IEEE conversion:

```rust
pub fn ibm_to_ieee(ibm_bytes: [u8; 8]) -> f64 {
    // Convert IBM 370 floating point to IEEE 754
}

pub fn ieee_to_ibm(value: f64) -> [u8; 8] {
    // Convert IEEE 754 to IBM 370 floating point
}
```

### Missing Values

Support for all 28 SAS missing codes:

```rust
pub enum MissingValue {
    Standard,           // .
    Special(char),      // .A through .Z
    Underscore,         // ._
}
```

## API

### Reading

```rust
use xport::{read_xpt, XptDataset};

let dataset: XptDataset = read_xpt("dm.xpt")?;
println!("Variables: {}", dataset.columns.len());
println!("Observations: {}", dataset.rows.len());
```

### Writing

```rust
use xport::{write_xpt, XptDataset, XptColumn, XptVersion};

let dataset = XptDataset {
    name: "DM".to_string(),
    label: Some("Demographics".to_string()),
    columns: vec![
        XptColumn::character("USUBJID", 20).with_label("Unique Subject ID"),
        XptColumn::numeric("AGE").with_label("Age"),
    ],
    rows: vec![/* data rows */],
    ..Default::default()
};

write_xpt("dm.xpt", &dataset)?;
```

### With Polars (optional feature)

```rust
use xport::polars::{read_xpt_to_dataframe, write_dataframe_to_xpt};
use polars::prelude::*;

// Read to DataFrame
let df = read_xpt_to_dataframe("dm.xpt")?;

// Write from DataFrame
write_dataframe_to_xpt(&df, "output.xpt", XptVersion::V5)?;
```

## Testing

```bash
cargo test --package xport
cargo test --package xport --features polars
```

### Test Categories

- Header parsing
- Numeric conversion accuracy
- Missing value roundtrip
- Large file handling
- V5/V8 compatibility

## See Also

- [XPT Format](../../output-formats/xpt-format.md) - User documentation
- [tss-output](tss-output.md) - Export integration
