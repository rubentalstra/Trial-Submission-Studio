# Implementation Plan: Clean `sdtm-standards` Crate

## Goal

Rewrite `sdtm-standards` from scratch with:
- **SDTM-IG v3.4 only** (remove v2.0 support)
- **Both CT versions** (2024-03-29 and 2025-09-26) with clean API
- **Move P21 loading to sdtm-validate**
- **No caching** (let callers manage)

---

## New Module Structure

```
crates/sdtm-standards/src/
├── lib.rs        # Public API and re-exports
├── error.rs      # StandardsError enum with thiserror
├── paths.rs      # Standards directory path resolution
├── sdtm_ig.rs    # SDTM-IG domain/variable loading
└── ct.rs         # Controlled Terminology loading with CtVersion enum
```

**4 files** - clean and focused.

---

## Public API

### CT Version Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CtVersion {
    #[default]
    V2024_03_29,  // Current production default
    V2025_09_26,  // Latest
}

impl CtVersion {
    pub fn dir_name(&self) -> &'static str;
    pub fn all() -> &'static [CtVersion];
    pub fn latest() -> Self;
}
```

### Main Functions

```rust
// SDTM-IG Loading
pub mod sdtm_ig {
    pub fn load() -> Result<Vec<Domain>>;                    // From default path
    pub fn load_from(path: &Path) -> Result<Vec<Domain>>;    // From custom path
}

// CT Loading
pub mod ct {
    pub fn load(version: CtVersion) -> Result<TerminologyRegistry>;
    pub fn load_from(path: &Path) -> Result<TerminologyRegistry>;
    pub fn load_catalog(path: &Path) -> Result<TerminologyCatalog>;
    pub fn load_sdtm_only(version: CtVersion) -> Result<TerminologyCatalog>;
}

// Path utilities
pub fn standards_root() -> PathBuf;
pub const STANDARDS_ENV_VAR: &str = "CDISC_STANDARDS_DIR";
```

---

## Error Types

```rust
#[derive(Debug, Error)]
pub enum StandardsError {
    #[error("Standards directory not found: {path}")]
    DirectoryNotFound { path: PathBuf },

    #[error("CSV file not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Failed to read CSV {path}: {source}")]
    CsvRead { path: PathBuf, #[source] source: csv::Error },

    #[error("Invalid {field} value '{value}' in {file}")]
    InvalidValue { field: &'static str, value: String, file: PathBuf },
}
```

---

## Implementation Approach

### Type-Safe CSV Parsing with Serde

```rust
// sdtm_ig.rs - Datasets.csv row
#[derive(Debug, Deserialize)]
struct DatasetCsvRow {
    #[serde(rename = "Class")]
    class: String,
    #[serde(rename = "Dataset Name")]
    dataset_name: String,
    #[serde(rename = "Dataset Label")]
    dataset_label: String,
    #[serde(rename = "Structure")]
    structure: String,
}

// sdtm_ig.rs - Variables.csv row
#[derive(Debug, Deserialize)]
struct VariableCsvRow {
    #[serde(rename = "Variable Order")]
    variable_order: String,
    #[serde(rename = "Dataset Name")]
    dataset_name: String,
    #[serde(rename = "Variable Name")]
    variable_name: String,
    #[serde(rename = "Variable Label")]
    variable_label: String,
    #[serde(rename = "Type")]
    variable_type: String,
    #[serde(rename = "CDISC CT Codelist Code(s)")]
    codelist_code: String,
    #[serde(rename = "Described Value Domain(s)")]
    described_value_domain: String,
    #[serde(rename = "Role")]
    role: String,
    #[serde(rename = "Core")]
    core: String,
}

// ct.rs - CT CSV row
#[derive(Debug, Deserialize)]
struct CtCsvRow {
    #[serde(rename = "Code")]
    code: String,
    #[serde(rename = "Codelist Code")]
    codelist_code: String,
    #[serde(rename = "Codelist Extensible (Yes/No)")]
    extensible: String,
    #[serde(rename = "Codelist Name")]
    codelist_name: String,
    #[serde(rename = "CDISC Submission Value")]
    submission_value: String,
    #[serde(rename = "CDISC Synonym(s)")]
    synonyms: String,
    #[serde(rename = "CDISC Definition")]
    definition: String,
    #[serde(rename = "NCI Preferred Term")]
    preferred_term: String,
}
```

---

## P21 Migration to sdtm-validate

**Move:** `sdtm-standards/src/p21_loader.rs` → `sdtm-validate/src/p21_loader.rs`

**Update sdtm-validate/lib.rs:**
```rust
pub mod p21_loader;
pub use p21_loader::{load_p21_rules, load_default_p21_rules};
```

---

## Dependencies

### sdtm-standards/Cargo.toml
```toml
[dependencies]
csv.workspace = true
serde = { workspace = true, features = ["derive"] }
sdtm-model = { path = "../sdtm-model" }
thiserror = "2.0"
```

### sdtm-validate/Cargo.toml (additions)
```toml
[dependencies]
csv.workspace = true  # For P21 loader
```

---

## Implementation Phases

### Phase 1: Setup and P21 Migration
1. Move `p21_loader.rs` to `sdtm-validate`
2. Update `sdtm-validate/lib.rs` with P21 loader exports
3. Update `sdtm-standards/Cargo.toml` (remove sdtm-validate dep)
4. Delete old P21 code from sdtm-standards

### Phase 2: Error Types and Paths
1. Create `error.rs` with `StandardsError`
2. Create `paths.rs` with path resolution
3. Create minimal `lib.rs` with module structure

### Phase 3: SDTM-IG Loader
1. Create `sdtm_ig.rs` with serde-based CSV parsing
2. Implement `load()` and `load_from()`
3. Parse into `Domain`, `Variable` with proper enums

### Phase 4: CT Loader
1. Create `ct.rs` with `CtVersion` enum
2. Implement `load()`, `load_from()`, `load_catalog()`, `load_sdtm_only()`
3. Support both CT versions cleanly

### Phase 5: Cleanup and Testing
1. Delete old files: `csv_utils.rs`, old `loaders.rs`, old `ct_loader.rs`
2. Update tests
3. Verify all tests pass

---

## Critical Files

### DELETE from sdtm-standards:
- `crates/sdtm-standards/src/csv_utils.rs`
- `crates/sdtm-standards/src/p21_loader.rs` (after migration)

### CREATE/REWRITE in sdtm-standards:
- `crates/sdtm-standards/src/lib.rs`
- `crates/sdtm-standards/src/error.rs` (NEW)
- `crates/sdtm-standards/src/paths.rs` (NEW)
- `crates/sdtm-standards/src/sdtm_ig.rs` (replaces loaders.rs)
- `crates/sdtm-standards/src/ct.rs` (replaces ct_loader.rs)

### CREATE in sdtm-validate:
- `crates/sdtm-validate/src/p21_loader.rs`

### UPDATE:
- `crates/sdtm-standards/Cargo.toml`
- `crates/sdtm-validate/Cargo.toml`
- `crates/sdtm-validate/src/lib.rs`
- Any crates importing P21 from sdtm-standards

---

## Usage Examples

```rust
// Load SDTM-IG
let domains = sdtm_standards::sdtm_ig::load()?;
let ae = domains.iter().find(|d| d.code == "AE").unwrap();

// Load CT with version selection
use sdtm_standards::ct::{self, CtVersion};
let registry = ct::load(CtVersion::latest())?;       // 2025-09-26
let registry = ct::load(CtVersion::default())?;      // 2024-03-29

// Load just SDTM CT
let sdtm_ct = ct::load_sdtm_only(CtVersion::V2024_03_29)?;

// Load P21 rules (now from sdtm-validate)
let rules = sdtm_validate::load_default_p21_rules()?;
```

---

## Final Public API (8 exports)

```rust
// Types
pub use ct::CtVersion;
pub use error::StandardsError;

// Functions
pub use ct::{load as load_ct, load_catalog, load_sdtm_only};
pub use paths::{standards_root, STANDARDS_ENV_VAR};
pub use sdtm_ig::load as load_sdtm_ig;

// Modules (for namespaced access)
pub mod ct;
pub mod sdtm_ig;
```
