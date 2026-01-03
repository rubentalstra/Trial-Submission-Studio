# tss-xpt

XPT (SAS Transport) file I/O crate.

## Overview

`tss-xpt` provides reading and writing of XPT V5 and V8 format files.

## Responsibilities

- Parse XPT file headers
- Read XPT data records
- Write XPT files
- Handle numeric conversions
- Manage missing value codes

## Dependencies

```toml
[dependencies]
byteorder = "1.5"
encoding_rs = "0.8"
tss-model = { path = "../tss-model" }
```

## Architecture

### Module Structure

```
tss-xpt/
├── src/
│   ├── lib.rs
│   ├── reader.rs      # XPT file reading
│   ├── writer.rs      # XPT file writing
│   ├── header.rs      # Header parsing
│   ├── namestr.rs     # Variable definitions
│   ├── numeric.rs     # IEEE/SAS conversion
│   └── missing.rs     # Missing value handling
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
use tss_xpt::XptReader;

let reader = XptReader::open("dm.xpt") ?;
let metadata = reader.metadata();
let records = reader.read_all() ?;
```

### Writing

```rust
use tss_xpt::XptWriter;

let writer = XptWriter::new("dm.xpt", XptVersion::V5) ?;
writer.write_metadata( & metadata) ?;
writer.write_records( & records) ?;
writer.finish() ?;
```

## Testing

```bash
cargo test --package tss-xpt
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
