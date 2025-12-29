---
name: domain-processor
description: Create or modify SDTM domain-specific processors. Use when implementing new domain transformations, adding CT normalization, or fixing domain-specific business logic.
---

# Domain Processor Development Skill

## Purpose

This skill guides development of domain-specific processors that apply SDTM transformations and business rules.

## When to Use

- Creating a new domain processor
- Adding controlled terminology normalization
- Implementing USUBJID prefixing logic
- Adding --SEQ column generation
- Fixing domain-specific validation issues

## Domain Processor Architecture

### Location
```
crates/sdtm-core/src/domain_processors/
├── mod.rs              # Registry and trait definition
├── dm.rs               # Demographics processor
├── ex.rs               # Exposure processor
├── vs.rs               # Vital Signs processor
└── [domain].rs         # Other domain processors
```

### Core Pattern

Every domain processor:
1. Receives a `DataFrame` and `PipelineContext`
2. Applies domain-specific transformations
3. Returns transformed `DataFrame`

### DomainProcessor Trait

```rust
pub trait DomainProcessor: Send + Sync {
    fn process(
        &self,
        df: DataFrame,
        context: &PipelineContext,
    ) -> Result<DataFrame, Box<dyn Error>>;
}
```

## Creating a New Domain Processor

### Step 1: Create the file

```bash
# Create new processor file
touch crates/sdtm-core/src/domain_processors/ae.rs
```

### Step 2: Implement the trait

```rust
use polars::prelude::*;
use crate::domain_processors::DomainProcessor;
use crate::pipeline::PipelineContext;
use std::error::Error;

pub struct AeProcessor;

impl DomainProcessor for AeProcessor {
    fn process(
        &self,
        mut df: DataFrame,
        context: &PipelineContext,
    ) -> Result<DataFrame, Box<dyn Error>> {
        // 1. USUBJID prefixing (if needed)
        df = prefix_usubjid(df, &context.study_id)?;

        // 2. Add --SEQ column
        df = add_sequence_column(df, "AESEQ")?;

        // 3. Normalize controlled terminology
        df = normalize_ct_columns(df)?;

        // 4. Domain-specific transformations
        df = apply_domain_rules(df)?;

        Ok(df)
    }
}
```

### Step 3: Register in mod.rs

```rust
// In crates/sdtm-core/src/domain_processors/mod.rs
mod ae;
pub use ae::AeProcessor;

// In registry function
pub fn get_processor(domain: &str) -> Option<Box<dyn DomainProcessor>> {
    match domain.to_uppercase().as_str() {
        "DM" => Some(Box::new(DmProcessor)),
        "EX" => Some(Box::new(ExProcessor)),
        "AE" => Some(Box::new(AeProcessor)), // Add here
        _ => None,
    }
}
```

## Common Transformations

### 1. USUBJID Prefixing

```rust
use polars::prelude::*;

fn prefix_usubjid(df: DataFrame, study_id: &str) -> Result<DataFrame, PolarsError> {
    df.lazy()
        .with_column(
            concat_str([
                lit(study_id),
                lit("-"),
                col("USUBJID"),
            ], "", false)
            .alias("USUBJID")
        )
        .collect()
}
```

### 2. Sequence Column Generation

```rust
fn add_sequence_column(df: DataFrame, seq_col: &str) -> Result<DataFrame, PolarsError> {
    let row_count = df.height();
    let sequence = (1..=row_count as i32).collect::<Vec<_>>();

    df.with_column(
        Series::new(seq_col, sequence)
    )
}
```

### 3. CT Normalization

```rust
fn normalize_sex_column(df: DataFrame) -> Result<DataFrame, PolarsError> {
    df.lazy()
        .with_column(
            when(col("SEX").eq(lit("Male")))
                .then(lit("M"))
                .when(col("SEX").eq(lit("Female")))
                .then(lit("F"))
                .otherwise(col("SEX"))
                .alias("SEX")
        )
        .collect()
}
```

### 4. Case-Insensitive Column Check

```rust
use crate::utils::CaseInsensitiveSet;

fn check_required_columns(df: &DataFrame) -> Result<(), Box<dyn Error>> {
    let columns = CaseInsensitiveSet::from_iter(
        df.get_column_names().iter().map(|s| s.to_string())
    );

    if !columns.contains("USUBJID") {
        return Err("Missing required column: USUBJID".into());
    }

    Ok(())
}
```

## Development Workflow

### 1. Research Requirements
- Read SDTMIG chapter in `standards/sdtmig/v3_4/chapters/`
- Check Variables.csv for required/expected variables
- Identify CT codelists in CDISC_CT.csv

### 2. Implement Processor
- Create processor file in `domain_processors/`
- Implement required transformations
- Add CT normalization for controlled variables
- Register in mod.rs

### 3. Test
```bash
# Run domain-specific tests
cargo test --package sdtm-core [domain]_processor

# Run end-to-end test
cargo run -- -s test_data/study -o output/
```

### 4. Validate
- Check validation output for errors
- Verify XPT generation succeeds
- Review generated Define-XML

## Common Patterns

### DM (Demographics) Special Cases
- ARM/ARMCD normalization
- RFSTDTC/RFENDTC date handling
- Country code standardization

### EX (Exposure) Special Cases
- EXDOSE numeric conversion
- EXDOSU units standardization
- Exposure start/end date derivation

### VS (Vital Signs) Special Cases
- VSTESTCD standardization
- VSORRESU → VSSTRESU unit conversion
- VSORRES → VSSTRESN numeric conversion

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usubjid_prefix() {
        let df = create_test_dataframe();
        let context = PipelineContext::new("STUDY001".to_string());

        let processor = AeProcessor;
        let result = processor.process(df, &context).unwrap();

        // Assert USUBJID has prefix
        assert!(result.column("USUBJID")
            .unwrap()
            .utf8()
            .unwrap()
            .get(0)
            .unwrap()
            .starts_with("STUDY001-"));
    }
}
```

## Key Principles

- **Read before implement**: Check SDTMIG chapter documentation first
- **Immutable context**: Never modify `PipelineContext`
- **Case-insensitive matching**: Use `CaseInsensitiveSet` for column names
- **Fail fast**: Return errors early, don't silently skip issues
- **Domain-specific only**: Keep generic logic in `sdtm-core/src/lib.rs`

## Related Documentation

- See `crates/sdtm-core/src/domain_processors/mod.rs` for trait definition
- See `crates/sdtm-core/src/domain_processors/dm.rs` for reference implementation
- See `standards/sdtmig/v3_4/chapters/` for SDTM requirements
- See `docs/NAMING_CONVENTIONS.md` for terminology
