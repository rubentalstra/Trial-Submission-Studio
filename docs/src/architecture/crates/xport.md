# xportrs

XPT (SAS Transport) file I/O library. Trial Submission Studio uses the [xportrs](https://crates.io/crates/xportrs) crate from crates.io.

## Overview

`xportrs` provides reading and writing of XPT V5 and V8 format files. It's a standalone crate designed for general SAS Transport file handling.

## Features

- Read XPT V5 and V8 format files
- Write XPT V5 and V8 format files
- Handle IBM mainframe to IEEE floating-point conversion
- Support all 28 SAS missing value codes
- Optional Polars DataFrame integration (`polars` feature)
- Optional serde serialization (`serde` feature)

## Installation

```toml
[dependencies]
xportrs = { version = "0.0.6", features = ["polars"] }  # With DataFrame support
# or
xportrs = "0.0.6"  # Core functionality only
```

## XPT Format Details

### File Structure

```
+-------------------------------------+
| Library Header (80 bytes x 2)       |
+-------------------------------------+
| Member Header (80 bytes x 3)        |
+-------------------------------------+
| Namestr Records (140 bytes each)    |
| (one per variable)                  |
+-------------------------------------+
| Observation Header (80 bytes)       |
+-------------------------------------+
| Data Records                        |
| (fixed-width, packed)               |
+-------------------------------------+
```

### Numeric Handling

IBM mainframe to IEEE conversion is handled automatically when reading/writing XPT files.

## API Examples

### Reading

```rust
use xportrs::Xpt;

let datasets = Xpt::reader()
    .read_path("dm.xpt")?;

for dataset in datasets {
    println!("Domain: {}", dataset.domain_code());
    println!("Variables: {}", dataset.len());
    println!("Observations: {}", dataset.nrows());
}
```

### Writing

```rust
use xportrs::{Column, ColumnData, Dataset, Xpt};

// Create columns with data
let usubjid = Column::new("USUBJID", ColumnData::String(vec![
    Some("ABC-001".into()),
    Some("ABC-002".into()),
]));

let age = Column::new("AGE", ColumnData::F64(vec![
    Some(45.0),
    Some(38.0),
]));

// Create dataset
let dataset = Dataset::with_label("DM", Some("Demographics"), vec![usubjid, age])?;

// Write to file
Xpt::writer(dataset)
    .finalize()?
    .write_path("dm.xpt")?;
```

### With Polars (optional feature)

```rust
use xportrs::Xpt;
use polars::prelude::*;

// Read to DataFrame
let datasets = Xpt::reader()
    .read_path("dm.xpt")?;
let df: DataFrame = datasets[0].clone().try_into()?;

// Write from DataFrame
let dataset: Dataset = df.try_into()?;
Xpt::writer(dataset)
    .finalize()?
    .write_path("output.xpt")?;
```

## Resources

- [xportrs on crates.io](https://crates.io/crates/xportrs)
- [xportrs documentation](https://docs.rs/xportrs)
- [GitHub repository](https://github.com/rubentalstra/xportrs)

## See Also

- [XPT Format](../../output-formats/xpt-format.md) - User documentation
- [tss-output](tss-output.md) - Export integration
