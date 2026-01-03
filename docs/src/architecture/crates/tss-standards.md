# tss-standards

CDISC standards data loader crate.

## Overview

`tss-standards` loads and provides access to embedded CDISC standard definitions.

## Responsibilities

- Load SDTM-IG definitions
- Load controlled terminology
- Provide domain/variable metadata
- Version management

## Dependencies

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
include_dir = "0.7"
tss-model = { path = "../tss-model" }
```

## Architecture

### Module Structure

```
tss-standards/
├── src/
│   ├── lib.rs
│   ├── loader.rs         # Data loading
│   ├── sdtm.rs           # SDTM definitions
│   ├── terminology.rs    # Controlled terminology
│   └── cache.rs          # In-memory caching
```

### Embedded Data

Standards are embedded at compile time:

```rust
use include_dir::{include_dir, Dir};

static STANDARDS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../standards");
```

## Data Structures

### SDTM Definitions

```rust
pub struct SdtmIg {
    pub version: String,
    pub domains: Vec<DomainDefinition>,
}

pub struct DomainDefinition {
    pub code: String,           // e.g., "DM"
    pub name: String,           // e.g., "Demographics"
    pub class: DomainClass,
    pub structure: String,
    pub variables: Vec<VariableDefinition>,
}

pub struct VariableDefinition {
    pub name: String,
    pub label: String,
    pub data_type: DataType,
    pub core: Core,             // Required/Expected/Permissible
    pub codelist: Option<String>,
    pub description: String,
}
```

### Controlled Terminology

```rust
pub struct ControlledTerminology {
    pub version: String,
    pub codelists: Vec<Codelist>,
}

pub struct Codelist {
    pub code: String,           // e.g., "C66731"
    pub name: String,           // e.g., "Sex"
    pub extensible: bool,
    pub terms: Vec<Term>,
}

pub struct Term {
    pub code: String,
    pub value: String,
    pub synonyms: Vec<String>,
}
```

## API

### Loading Standards

```rust
use tss_standards::Standards;

// Load with specific versions
let standards = Standards::load(
SdtmVersion::V3_4,
CtVersion::V2024_12_20,
) ?;

// Get domain definition
let dm = standards.get_domain("DM") ?;

// Get codelist
let sex = standards.get_codelist("SEX") ?;
```

### Querying

```rust
// Get required variables for domain
let required = standards.required_variables("DM");

// Check if value is in codelist
let valid = standards.is_valid_term("SEX", "M");

// Get variable definition
let var = standards.get_variable("DM", "USUBJID") ?;
```

## Embedded Data Format

### SDTM JSON

```json
{
  "version": "3.4",
  "domains": [
    {
      "code": "DM",
      "name": "Demographics",
      "class": "SPECIAL_PURPOSE",
      "structure": "One record per subject",
      "variables": [
        {
          "name": "STUDYID",
          "label": "Study Identifier",
          "dataType": "Char",
          "core": "Required"
        }
      ]
    }
  ]
}
```

### CT JSON

```json
{
  "version": "2024-12-20",
  "codelists": [
    {
      "code": "C66731",
      "name": "Sex",
      "extensible": false,
      "terms": [
        {
          "code": "C16576",
          "value": "F"
        },
        {
          "code": "C20197",
          "value": "M"
        }
      ]
    }
  ]
}
```

## Caching

Standards are cached in memory after first load:

```rust
lazy_static! {
    static ref STANDARDS_CACHE: RwLock<Option<Standards>> = RwLock::new(None);
}
```

## Testing

```bash
cargo test --package tss-standards
```

### Test Categories

- JSON parsing
- Version loading
- Query accuracy
- Missing data handling

## See Also

- [CDISC Standards](../../cdisc-standards/overview.md) - Standards overview
- [Controlled Terminology](../../cdisc-standards/controlled-terminology.md) - CT details
- [tss-validate](tss-validate.md) - Uses standards for validation
