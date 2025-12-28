# Naming Conventions

This document establishes naming conventions for the CDISC Transpiler codebase,
aligning **SDTMIG terminology** with **Rust style guidelines**.

## Goals

1. **Consistency**: Same concept → same name everywhere
2. **Clarity**: Names should be self-documenting
3. **SDTM Alignment**: Use official CDISC terminology where applicable
4. **Rust Compliance**: Follow Rust API Guidelines (RFC 430)

---

## SDTM Terminology Reference

Per SDTMIG v3.4 Chapter 2 (Fundamentals of the SDTM):

### Core Concepts

| SDTM Term           | Definition                                                       | Rust Type Name       |
| ------------------- | ---------------------------------------------------------------- | -------------------- |
| **Domain**          | Collection of logically related observations with a common topic | `Domain`             |
| **Dataset**         | Physical representation of a domain (the .xpt file)              | `Dataset`            |
| **Variable**        | A column in a dataset                                            | `Variable`           |
| **Observation**     | A row in a dataset                                               | (row in `DataFrame`) |
| **Dataset Class**   | Category of domain (Interventions, Events, Findings, etc.)       | `DatasetClass`       |
| **Controlled Term** | A valid value from CDISC Controlled Terminology                  | `Term`               |
| **Codelist**        | A set of controlled terms for a variable                         | `Codelist`           |

### Variable Roles (SDTMIG v3.4 Section 2.1)

| Role           | Purpose                                                             | Example             |
| -------------- | ------------------------------------------------------------------- | ------------------- |
| **Identifier** | Identifies study, subject, domain, sequence                         | STUDYID, USUBJID    |
| **Topic**      | Focus of the observation                                            | --TESTCD, --TRT     |
| **Qualifier**  | Additional attributes (Result, Record, Grouping, Synonym, Variable) | --ORRES, --CAT      |
| **Timing**     | When the observation occurred                                       | --DTC, --STDTC      |
| **Rule**       | Trial Design conditions (start, end, branch, loop)                  | RULE (in TE domain) |

### Core Designations (SDTMIG v3.4 Section 4.1.5)

| Designation | Meaning                                    | Validation Severity |
| ----------- | ------------------------------------------ | ------------------- |
| **Req**     | Required: must exist, cannot be null       | Error               |
| **Exp**     | Expected: should exist when applicable     | Warning             |
| **Perm**    | Permissible: optional, no issue if missing | Info/None           |

### Dataset Classes (SDTMIG v3.4 Section 2.3)

| Class               | Domains                                          | Description                           |
| ------------------- | ------------------------------------------------ | ------------------------------------- |
| **Interventions**   | AG, CM, EC, EX, ML, PR, SU                       | Treatments administered to subject    |
| **Events**          | AE, BE, CE, DS, DV, HO, MH                       | Occurrences independent of evaluation |
| **Findings**        | BS, CV, DA, EG, IE, LB, MB, MI, MK, MS, PC, etc. | Results of planned evaluations        |
| **Findings About**  | FA, SR                                           | Findings about Events/Interventions   |
| **Special-Purpose** | CO, DM, SE, SM, SV                               | Subject-level, non-GO class           |
| **Trial Design**    | TA, TD, TE, TI, TM, TS, TV                       | Study design (no subject data)        |
| **Relationship**    | RELREC, RELSPEC, RELSUB, SUPPQUAL                | Links between records/datasets        |
| **Study Reference** | DI, OI                                           | Study-specific terminology            |

---

## Rust Naming Guidelines

Per Rust API Guidelines (RFC 430):

| Item                | Convention        | Example                       |
| ------------------- | ----------------- | ----------------------------- |
| Types (struct/enum) | `UpperCamelCase`  | `DatasetClass`, `Codelist`    |
| Functions/methods   | `snake_case`      | `validate_domain`, `is_valid` |
| Constants           | `SCREAMING_SNAKE` | `MAX_VARIABLE_LENGTH`         |
| Modules             | `snake_case`      | `controlled_terminology`      |
| Type parameters     | `UpperCamelCase`  | `T`, `Error`                  |

---

## Type Renames (Completed ✅)

### sdtm-model

| Old Name                | New Name              | Rationale                                    | Status |
| ----------------------- | --------------------- | -------------------------------------------- | ------ |
| `CtTerm`                | `Term`                | Simpler; CT context is implied by module     | ✅     |
| `CtCatalog`             | `TerminologyCatalog`  | More descriptive; matches CDISC "CT Package" | ✅     |
| `CtRegistry`            | `TerminologyRegistry` | More descriptive                             | ✅     |
| `ResolvedCodelist`      | `ResolvedCodelist`    | Keep (descriptive)                           | ✅     |
| `IssueSeverity`         | `Severity`            | Shorter; context is clear from usage         | ✅     |
| `ConformanceIssue`      | `ValidationIssue`     | More accurate; "validation" is the activity  | ✅     |
| `ConformanceReport`     | `ValidationReport`    | Consistent with `ValidationIssue`            | ✅     |
| `DatasetClass`          | `DatasetClass`        | Keep (matches SDTM terminology)              | ✅     |
| `CaseInsensitiveLookup` | `CaseInsensitiveSet`  | More accurate (it's a set, not a lookup)     | ✅     |

### sdtm-validate

| Current Name                  | New Name            | Rationale                        |
| ----------------------------- | ------------------- | -------------------------------- |
| `ValidationContext`           | `ValidationContext` | ✅ Keep                          |
| `validate_domain`             | `validate_domain`   | ✅ Keep                          |
| `validate_domains`            | `validate_domains`  | ✅ Keep                          |
| `Validator`                   | `DomainValidator`   | More specific; validates domains |
| `CrossDomainValidationInput`  | `CrossDomainInput`  | Shorter                          |
| `CrossDomainValidationResult` | `CrossDomainResult` | Shorter                          |

### sdtm-core

| Current Name           | New Name                 | Rationale               |
| ---------------------- | ------------------------ | ----------------------- |
| `ProcessingContext`    | `ProcessingContext`      | ✅ Keep                 |
| `ProcessingOptions`    | `ProcessingOptions`      | ✅ Keep                 |
| `DomainPipeline`       | `TransformationPipeline` | Clearer purpose         |
| `ProcessingStep`       | `TransformationStep`     | Matches pipeline rename |
| `normalize_ct_columns` | `normalize_terminology`  | More descriptive        |
| `SuppqualResult`       | `SuppqualResult`         | ✅ Keep                 |

### sdtm-standards

| Current Name             | New Name             | Rationale                    |
| ------------------------ | -------------------- | ---------------------------- |
| `ct_loader`              | `terminology_loader` | More descriptive module name |
| `load_ct_from_directory` | `load_terminology`   | Shorter, clear               |

---

## Module Organization

### Recommended Module Names

| Purpose                        | Module Name   | File                         |
| ------------------------------ | ------------- | ---------------------------- |
| Controlled Terminology types   | `terminology` | `terminology.rs`             |
| Validation types and functions | `validation`  | `validation.rs`              |
| Domain metadata types          | `domain`      | `domain.rs`                  |
| Variable metadata types        | `variable`    | `variable.rs` (or in domain) |
| XPT file handling              | `xpt`         | `xpt.rs`                     |
| Dataset-XML handling           | `dataset_xml` | `dataset_xml.rs`             |
| Define-XML handling            | `define_xml`  | `define_xml.rs`              |

### Crate Naming

| Current Crate    | Status  | Notes                          |
| ---------------- | ------- | ------------------------------ |
| `sdtm-model`     | ✅ Keep | Core data types                |
| `sdtm-standards` | ✅ Keep | Standards loading (SDTMIG, CT) |
| `sdtm-validate`  | ✅ Keep | Validation logic               |
| `sdtm-core`      | ✅ Keep | Processing/transformation      |
| `sdtm-ingest`    | ✅ Keep | Data ingestion                 |
| `sdtm-map`       | ✅ Keep | Column mapping                 |
| `sdtm-report`    | ✅ Keep | Output generation              |
| `sdtm-xpt`       | ✅ Keep | XPT format handling            |
| `sdtm-cli`       | ✅ Keep | Command-line interface         |

---

## Specific Naming Rules

### 1. Controlled Terminology

Use "terminology" in public APIs, "CT" only in internal/short contexts:

```rust
// ✅ Good - public API
pub fn load_terminology(path: &Path) -> TerminologyRegistry
pub fn resolve_term(registry: &TerminologyRegistry, code: &str) -> Option<&Term>

// ✅ Acceptable - internal/short variable names
let ct_registry = load_terminology(&ct_path)?;
let ct = registry.resolve("C66731")?;
```

### 2. Validation

Use consistent terminology for validation concepts:

```rust
// ✅ Good
pub enum Severity { Error, Warning, Info }

pub struct ValidationIssue {
    pub severity: Severity,
    pub code: String,          // Issue code (e.g., "SD0001")
    pub variable: Option<String>,
    pub message: String,
    pub row_count: Option<u64>,
    pub codelist_code: Option<String>,
}

pub struct ValidationReport {
    pub domain_code: String,
    pub issues: Vec<ValidationIssue>,
}

// ✅ Good - function names
pub fn validate_domain(domain: &Domain, df: &DataFrame) -> ValidationReport
pub fn has_errors(report: &ValidationReport) -> bool
```

### 3. Domain and Dataset

Distinguish between metadata (Domain) and data (Dataset/DataFrame):

```rust
// Domain = metadata definition from SDTMIG
pub struct Domain {
    pub code: String,           // "DM", "AE", "LB"
    pub name: String,           // "Demographics", "Adverse Events"
    pub class: DatasetClass,    // Interventions, Events, Findings
    pub variables: Vec<Variable>,
}

// Dataset = actual data being processed (use Polars DataFrame)
type Dataset = polars::prelude::DataFrame;
```

### 4. Variable Naming Fragments

Per SDTMIG Appendix D, use consistent prefixes:

```rust
// When referring to SDTM variable patterns in code:
const TOPIC_SUFFIX: &str = "TESTCD";     // --TESTCD
const DATETIME_SUFFIX: &str = "DTC";      // --DTC  
const START_DATETIME_SUFFIX: &str = "STDTC"; // --STDTC
const END_DATETIME_SUFFIX: &str = "ENDTC";   // --ENDTC
const SEQUENCE_SUFFIX: &str = "SEQ";      // --SEQ

// Function to check if variable matches pattern
pub fn is_datetime_variable(name: &str) -> bool {
    name.ends_with("DTC") || name.ends_with("STDTC") || name.ends_with("ENDTC")
}
```

### 5. Core Designation

Use enum, not strings:

```rust
/// Variable core designation per SDTMIG v3.4 Section 4.1.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreDesignation {
    /// Required: must exist, cannot be null
    Required,
    /// Expected: should exist when applicable  
    Expected,
    /// Permissible: optional
    Permissible,
}

impl CoreDesignation {
    /// Parse from SDTMIG CSV value ("Req", "Exp", "Perm")
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "REQ" | "REQUIRED" => Some(Self::Required),
            "EXP" | "EXPECTED" => Some(Self::Expected),
            "PERM" | "PERMISSIBLE" => Some(Self::Permissible),
            _ => None,
        }
    }
    
    /// Severity when variable is missing
    pub fn missing_severity(&self) -> Severity {
        match self {
            Self::Required => Severity::Error,
            Self::Expected => Severity::Warning,
            Self::Permissible => Severity::Info, // Or don't report
        }
    }
}
```

### 6. Error Handling

Use descriptive error types:

```rust
// ✅ Good - specific error enum
#[derive(Debug, thiserror::Error)]
pub enum TranspilerError {
    #[error("failed to load standards: {0}")]
    StandardsLoad(#[from] std::io::Error),
    
    #[error("invalid domain code: {code}")]
    InvalidDomainCode { code: String },
    
    #[error("required variable missing: {variable} in {domain}")]
    RequiredVariableMissing { domain: String, variable: String },
    
    #[error("controlled terminology violation: {value} not in {codelist}")]
    TerminologyViolation { value: String, codelist: String },
}

// ✅ Good - type alias for Result
pub type Result<T> = std::result::Result<T, TranspilerError>;
```

---

## Abbreviations

### Allowed Abbreviations

These abbreviations are standard in CDISC and may be used:

| Abbreviation | Meaning                    | Usage Context           |
| ------------ | -------------------------- | ----------------------- |
| `CT`         | Controlled Terminology     | Internal variables only |
| `DTC`        | Date/Time Character        | SDTM variable suffix    |
| `SEQ`        | Sequence                   | SDTM variable suffix    |
| `XPT`        | SAS Transport              | File format             |
| `XML`        | Extensible Markup Language | File format             |

### Forbidden Abbreviations

Avoid these unclear abbreviations:

| ❌ Avoid | ✅ Use Instead         |
| -------- | ---------------------- |
| `ctx`    | `context`              |
| `df`     | `data`, `frame`        |
| `cfg`    | `config`               |
| `val`    | `value`, `validate`    |
| `proc`   | `process`, `processor` |
| `impl`   | (reserved keyword)     |

---

## Function Naming Patterns

### Validation Functions

```rust
// Pattern: validate_<thing>
pub fn validate_domain(...) -> ValidationReport
pub fn validate_variable(...) -> Vec<ValidationIssue>
pub fn validate_terminology(...) -> Vec<ValidationIssue>

// Pattern: check_<thing> for internal boolean checks
fn check_required_variables(...) -> Vec<ValidationIssue>
fn check_terminology_values(...) -> Vec<ValidationIssue>
fn check_datetime_format(...) -> Vec<ValidationIssue>

// Pattern: has_<thing> for boolean queries
pub fn has_errors(report: &ValidationReport) -> bool
pub fn has_warnings(report: &ValidationReport) -> bool
```

### Loading/Parsing Functions

```rust
// Pattern: load_<thing>
pub fn load_standards(path: &Path) -> Result<Vec<Domain>>
pub fn load_terminology(path: &Path) -> Result<TerminologyRegistry>

// Pattern: parse_<thing> for string parsing
pub fn parse_datetime(value: &str) -> Option<NaiveDateTime>
pub fn parse_codelist_code(value: &str) -> Vec<String>
```

### Transformation Functions

```rust
// Pattern: normalize_<thing>
pub fn normalize_terminology(df: &mut DataFrame, codelist: &Codelist)
pub fn normalize_case(value: &str) -> String

// Pattern: transform_<thing>
pub fn transform_domain(df: &mut DataFrame, domain: &Domain)

// Pattern: process_<thing> for multi-step operations
pub fn process_domain(df: &mut DataFrame, context: &ProcessingContext)
pub fn process_study(input: ProcessStudyRequest) -> ProcessStudyResponse
```

---

## Type Organization

### Preferred Struct Field Order

1. Identifier fields (code, name, id)
2. Classification fields (class, type, role)
3. Data fields (value, content)
4. Metadata fields (description, notes)
5. Flags/options (extensible, required)

```rust
pub struct Variable {
    // 1. Identifier
    pub name: String,
    
    // 2. Classification  
    pub role: VariableRole,
    pub core: CoreDesignation,
    
    // 3. Data
    pub data_type: DataType,
    pub codelist_code: Option<String>,
    
    // 4. Metadata
    pub label: String,
    pub description: Option<String>,
    
    // 5. Flags
    pub is_identifier: bool,
}
```

---

## Migration Checklist

When renaming types/functions:

- [ ] Update type definition
- [ ] Update all usages (grep search)
- [ ] Update re-exports in `lib.rs`
- [ ] Update tests
- [ ] Update documentation/comments
- [ ] Run `cargo fmt` and `cargo clippy`
- [ ] Run `cargo test`

---

## References

- SDTMIG v3.4: `/standards/sdtmig/v3_4/chapters/`
- SDTM_CT_relationships.md: `/SDTM_CT_relationships.md`
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- RFC 430 (Naming):
  https://rust-lang.github.io/rfcs/0430-finalizing-naming-conventions.html
