# CDISC Controlled Terminology

This folder contains CDISC Controlled Terminology (CT) CSV files embedded at compile time.

## Folder Structure

```
terminology/
├── README.md           # This file
├── 2024-03-29/         # CT version 2024-03-29 (default)
│   ├── SDTM_CT_2024-03-29.csv
│   ├── ADaM_CT_2024-03-29.csv
│   ├── SEND_CT_2024-03-29.csv
│   └── ...
├── 2025-03-28/         # CT version 2025-03-28
│   └── ...
└── 2025-09-26/         # CT version 2025-09-26 (latest)
    └── ...
```

## Adding a New CT Version

CDISC publishes CT updates quarterly. Follow these steps to add a new version:

### Step 1: Download CT Files

1. Go to the [CDISC Library Browser](https://library.cdisc.org/browser/#/)
2. Navigate to Controlled Terminology and select the version you need
3. Download the CT packages (SDTM, ADaM, SEND, etc.) as CSV files

### Step 2: Create Version Folder

Create a new folder named with the CT release date:

```bash
mkdir crates/tss-standards/data/terminology/YYYY-MM-DD
```

### Step 3: Update `embedded.rs`

Add `include_str!()` constants for each new file:

```rust
// =============================================================================
// Controlled Terminology - YYYY-MM-DD
// =============================================================================

/// SDTM CT YYYY-MM-DD
pub const CT_YYYY_MM_DD_SDTM: &str =
    include_str!("../data/terminology/YYYY-MM-DD/SDTM_CT_YYYY-MM-DD.csv");

// ... repeat for each CT file
```

### Step 4: Update `ct/loader.rs`

1. Add new variant to `CtVersion` enum:

```rust
pub enum CtVersion {
    V2024_03_29,
    V2025_03_28,
    V2025_09_26,
    V_YYYY_MM_DD,  // Add new variant
}
```

2. Update `dir_name()`:

```rust
pub const fn dir_name(&self) -> &'static str {
    match self {
        Self::V2024_03_29 => "2024-03-29",
        Self::V2025_03_28 => "2025-03-28",
        Self::V2025_09_26 => "2025-09-26",
        Self::V_YYYY_MM_DD => "YYYY-MM-DD",  // Add
    }
}
```

3. Update `all()` to include new variant

4. Update `latest()` if this is the newest version

### Step 5: Update `ct_files_for_version()` in `embedded.rs`

Add match arm for the new version:

```rust
CtVersion::V_YYYY_MM_DD => vec![
    ("SDTM_CT_YYYY-MM-DD.csv", CT_YYYY_MM_DD_SDTM),
    ("ADaM_CT_YYYY-MM-DD.csv", CT_YYYY_MM_DD_ADAM),
    // ... all files for this version
],
```

### Step 6: Update `sdtm_ct_for_version()` in `embedded.rs`

Add match arm for SDTM-only loading.

### Step 7: Verify

```bash
cargo test --package tss-standards ct
cargo clippy --package tss-standards
```

## CT File Format

CDISC CT CSV files have these columns:

| Column                         | Description                                    |
|--------------------------------|------------------------------------------------|
| `Code`                         | NCI concept code                               |
| `Codelist Code`                | Parent codelist code (blank for codelist rows) |
| `Codelist Extensible (Yes/No)` | Whether custom values are allowed              |
| `Codelist Name`                | Human-readable name                            |
| `CDISC Submission Value`       | The valid value for submissions                |
| `CDISC Synonym(s)`             | Alternative names (semicolon-separated)        |
| `CDISC Definition`             | Term definition                                |
| `NCI Preferred Term`           | NCI standard term                              |

## Publishing Sets

Each CT version contains multiple publishing sets:

| Set        | Description                                       |
|------------|---------------------------------------------------|
| SDTM       | Study Data Tabulation Model                       |
| ADaM       | Analysis Data Model                               |
| SEND       | Standard for Exchange of Nonclinical Data         |
| Define-XML | Define-XML metadata                               |
| Protocol   | Protocol terminology                              |
| CDASH      | Clinical Data Acquisition Standards Harmonization |
| DDF        | Digital Data Flow                                 |
| MRCT       | Multi-Regional Clinical Trials                    |
| Glossary   | CDISC Glossary                                    |

Not all sets are available in every CT release.

## Source

Official CT packages: <https://library.cdisc.org/browser/#/>
