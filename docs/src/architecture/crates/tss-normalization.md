# tss-normalization

Data normalization crate for CDISC conversions.

## Overview

`tss-normalization` applies normalizations to convert source data to SDTM-compliant format.

## Responsibilities

- Apply column mappings
- Normalize data values to SDTM standards
- Derive computed variables
- Handle date conversions to ISO 8601
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
tss-normalization/
├── src/
│   ├── lib.rs
│   ├── executor.rs       # Normalization execution
│   ├── inference.rs      # Type inference from domain metadata
│   ├── preview.rs        # Preview DataFrame builder
│   ├── types.rs          # Core types (NormalizationType, NormalizationRule, etc.)
│   ├── error.rs          # NormalizationError
│   └── normalization/
│       ├── mod.rs
│       ├── ct.rs         # Controlled terminology normalization
│       ├── datetime.rs   # ISO 8601 datetime formatting
│       ├── duration.rs   # ISO 8601 duration formatting
│       ├── numeric.rs    # Numeric conversions
│       └── studyday.rs   # Study day calculations
```

## Normalization Types

### NormalizationType Enum

```rust
pub enum NormalizationType {
    /// Copy value directly without modification
    CopyDirect,
    /// Auto-generate constant (STUDYID, DOMAIN)
    Constant,
    /// Derive USUBJID as STUDYID-SUBJID
    UsubjidPrefix,
    /// Generate sequence number per USUBJID
    SequenceNumber,
    /// Format as ISO 8601 datetime
    Iso8601DateTime,
    /// Format as ISO 8601 date
    Iso8601Date,
    /// Format as ISO 8601 duration
    Iso8601Duration,
    /// Calculate study day relative to RFSTDTC
    StudyDay { reference_dtc: String },
    /// Normalize using controlled terminology codelist
    CtNormalization { codelist_code: String },
    /// Convert to numeric (Float64)
    NumericConversion,
}
```

## API

### Building a Pipeline

```rust
use tss_normalization::{infer_normalization_rules, execute_normalization, NormalizationContext};

// Infer rules from domain metadata
let pipeline = infer_normalization_rules(&domain);

// Create execution context
let context = NormalizationContext::new(study_id, &domain.name)
    .with_ct_registry(ct_registry)
    .with_mappings(mappings);

// Execute normalization
let result = execute_normalization(&source_df, &pipeline, &context)?;
```

### Preview Functions

```rust
use tss_normalization::build_preview_dataframe_with_dm_and_omitted;

let result = build_preview_dataframe_with_dm_and_omitted(
    &source_df,
    &mappings,
    &omitted,
    &domain,
    &study_id,
    dm_df.as_ref(),
    ct_registry.as_ref(),
)?;
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
cargo test --package tss-normalization
```

### Test Strategy

- Unit tests for each normalization type
- Integration tests with sample data
- Snapshot tests for output consistency

## See Also

- [Column Mapping](../../user-guide/column-mapping.md) - Mapping workflow
- [tss-map](tss-map.md) - Mapping engine
- [tss-validate](tss-validate.md) - Validation after normalization
