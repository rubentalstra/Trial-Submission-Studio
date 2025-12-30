# CDISC Transpiler - Deep Dive Codebase Analysis

**Generated:** 2025-12-30\
**Updated:** 2025-12-30\
**Philosophy:** "Can we remove it? If not, why not? Is it really needed?"

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Crate Dependency Graph](#crate-dependency-graph)
3. [File-by-File Analysis](#file-by-file-analysis)
4. [Type Inventory](#type-inventory)
5. [Redundancies & Removal Candidates](#redundancies--removal-candidates)
6. [Action Plan](#action-plan)
7. [Naming Convention Alignment](#naming-convention-alignment)

---

## Executive Summary

### Architecture Overview

The codebase has **10 crates** with clear layered architecture:

| Layer             | Crate            | Files | LoC (est) | Can Remove Crate? | Why Needed?           |
| ----------------- | ---------------- | ----- | --------- | ----------------- | --------------------- |
| **Foundation**    | `sdtm-model`     | 10    | ~1500     | ❌ No             | Core types, zero deps |
| **Data I/O**      | `sdtm-ingest`    | 5     | ~800      | ❌ No             | CSV loading           |
| **Data I/O**      | `sdtm-xpt`       | 1     | ~700      | ❌ No             | XPT format support    |
| **Standards**     | `sdtm-standards` | 5     | ~600      | ❌ No             | Load SDTMIG/CT/P21    |
| **Transform**     | `sdtm-transform` | 10    | ~1200     | ❌ No             | Pure transformations  |
| **Orchestration** | `sdtm-core`      | 25    | ~2500     | ❌ No             | Pipeline + processors |
| **Mapping**       | `sdtm-map`       | 5     | ~700      | ❌ No             | Column mapping        |
| **Validation**    | `sdtm-validate`  | 1     | ~970      | ❌ No             | Conformance checking  |
| **Output**        | `sdtm-report`    | 5     | ~1000     | ❌ No             | Output generation     |
| **Frontend**      | `sdtm-gui`       | ~15   | ~2000     | ❌ No             | Desktop app           |

**Total:** ~67 source files, ~12,000 lines of code

### Key Findings: What CAN Be Removed/Simplified

| Category      | Item                                          | Action                  | Impact                 |
| ------------- | --------------------------------------------- | ----------------------- | ---------------------- |
| 🗑️ **DELETE** | `sdtm-transform/src/normalization/numeric.rs` | Remove file             | Duplicates sdtm-ingest |
| 🗑️ **DELETE** | `sdtm-model/src/error.rs`                     | Remove file             | Use anyhow instead     |
| 📦 **MERGE**  | `sdtm-core/src/transforms.rs`                 | Merge into processor.rs | Reduce duplication     |
| 📐 **SPLIT**  | `sdtm-validate/src/lib.rs` (969 lines)        | Split into 4 modules    | Maintainability        |
| 🏷️ **RENAME** | `StudyMetadata` → `SourceMetadata`            | Clarify purpose         | Avoid SDTMIG conflict  |
| 🏷️ **RENAME** | `StudyCodelist` → `SourceCodelist`            | Clarify purpose         | Avoid SDTMIG conflict  |
| ⚠️ **REVIEW** | `ProcessorRegistry`                           | Consider simplification | 17 fixed processors    |

### Summary Counts

- **Files Safe to Remove:** 2
- **Files to Split/Reorganize:** 2
- **Type Renames Needed:** 2

---

## Crate Dependency Graph

```
sdtm-model (foundation - no internal deps)
    │
    ├── sdtm-ingest (data loading)
    │       │
    │       └── sdtm-transform (transformation logic)
    │               │
    │               ├── sdtm-core (orchestration)
    │               │       │
    │               │       └── sdtm-validate
    │               │
    │               └── sdtm-map (column mapping)
    │
    ├── sdtm-standards (standards loaders)
    │
    ├── sdtm-report (output generation)
    │       └── sdtm-xpt
    │
    └── sdtm-gui (desktop app)
            └── (all above)
```

---

## File-by-File Analysis

### Decision Key

| Symbol          | Meaning                                     |
| --------------- | ------------------------------------------- |
| ✅ **KEEP**     | Essential, well-designed, no changes needed |
| 🗑️ **DELETE**   | Can be completely removed                   |
| 📦 **MERGE**    | Combine with another file                   |
| 📐 **SPLIT**    | Break into smaller modules                  |
| 🔄 **REFACTOR** | Keep but needs changes                      |
| ⚠️ **REVIEW**   | Needs closer examination                    |

---

### sdtm-model (10 files)

| File             | Lines | Decision    | Justification                               |
| ---------------- | ----- | ----------- | ------------------------------------------- |
| `lib.rs`         | ~50   | ✅ KEEP     | Module exports - required                   |
| `domain.rs`      | ~270  | ✅ KEEP     | Core `Domain`, `Variable`, `DatasetClass`   |
| `ct.rs`          | ~285  | ✅ KEEP     | `Codelist`, `Term`, `TerminologyRegistry`   |
| `conformance.rs` | ~220  | ✅ KEEP     | `ValidationReport`, `ValidationIssue`       |
| `p21.rs`         | ~265  | ✅ KEEP     | `P21Rule`, `P21Category`                    |
| `options.rs`     | ~120  | ✅ KEEP     | `ProcessingOptions`, `NormalizationOptions` |
| `metadata.rs`    | ~100  | 🔄 REFACTOR | Rename types (see naming section)           |
| `mapping.rs`     | ~50   | ✅ KEEP     | `MappingSuggestion`, `MappingConfig`        |
| `processing.rs`  | ~90   | ⚠️ REVIEW   | GUI-specific types, could move to sdtm-gui  |
| `error.rs`       | ~40   | 🗑️ DELETE   | `SdtmError` unused - use `anyhow` instead   |
| `lookup.rs`      | ~80   | ✅ KEEP     | `CaseInsensitiveSet` - used everywhere      |

**Summary:** 8 KEEP, 1 DELETE, 1 REFACTOR, 1 REVIEW

---

### sdtm-ingest (5 files)

| File                | Lines | Decision | Justification                                     |
| ------------------- | ----- | -------- | ------------------------------------------------- |
| `lib.rs`            | ~25   | ✅ KEEP  | Module exports                                    |
| `csv_table.rs`      | ~250  | ✅ KEEP  | CSV reading with double-header detection          |
| `discovery.rs`      | ~150  | ✅ KEEP  | Domain file discovery                             |
| `polars_utils.rs`   | ~120  | ✅ KEEP  | `any_to_string`, `parse_f64` - canonical location |
| `study_metadata.rs` | ~400  | ✅ KEEP  | `AppliedStudyMetadata` + re-exports               |

**Summary:** 5 KEEP - **Clean crate**

---

### sdtm-transform (10 files)

| File                        | Lines | Decision    | Justification                              |
| --------------------------- | ----- | ----------- | ------------------------------------------ |
| `lib.rs`                    | ~25   | ✅ KEEP     | Module exports                             |
| `data_utils.rs`             | ~200  | ✅ KEEP     | String manipulation, QNAM sanitization     |
| `frame.rs`                  | ~105  | ✅ KEEP     | `DomainFrame` - core wrapper type          |
| `frame_builder.rs`          | ~150  | ✅ KEEP     | DataFrame construction                     |
| `domain_sets.rs`            | ~100  | ✅ KEEP     | Domain collection utilities                |
| `suppqual.rs`               | ~330  | ✅ KEEP     | SUPPQUAL generation per SDTMIG 8.4         |
| `relationships.rs`          | ~200  | ✅ KEEP     | RELREC/RELSPEC per SDTMIG 8.5              |
| `normalization/mod.rs`      | ~10   | 🔄 REFACTOR | Remove numeric re-export                   |
| `normalization/ct.rs`       | ~300  | ✅ KEEP     | CT value normalization                     |
| `normalization/datetime.rs` | ~250  | ✅ KEEP     | ISO 8601 parsing                           |
| `normalization/numeric.rs`  | ~80   | 🗑️ DELETE   | **Duplicates sdtm-ingest/polars_utils.rs** |

**Summary:** 8 KEEP, 1 DELETE, 1 REFACTOR

---

### sdtm-core (25 files)

| File                   | Lines | Decision  | Justification                             |
| ---------------------- | ----- | --------- | ----------------------------------------- |
| `lib.rs`               | ~50   | ✅ KEEP   | Module exports                            |
| `pipeline_context.rs`  | ~120  | ✅ KEEP   | `PipelineContext` - central orchestration |
| `processor.rs`         | ~400  | ✅ KEEP   | Main `process_domain()` function          |
| `transforms.rs`        | ~200  | 📦 MERGE  | Move into processor.rs or make internal   |
| **domain_processors/** |       |           |                                           |
| `processor_trait.rs`   | ~520  | ⚠️ REVIEW | Keep trait, consider simplifying registry |
| `common.rs`            | ~350  | ✅ KEEP   | Shared helper functions                   |
| `operations.rs`        | ~400  | ✅ KEEP   | Reusable column operations                |
| `default.rs`           | ~20   | ✅ KEEP   | Fallback processor                        |
| `ae.rs`                | ~40   | ✅ KEEP   | Adverse Events                            |
| `cm.rs`                | ~50   | ✅ KEEP   | Concomitant Meds                          |
| `da.rs`                | ~30   | ✅ KEEP   | Drug Accountability                       |
| `dm.rs`                | ~40   | ✅ KEEP   | Demographics                              |
| `ds.rs`                | ~60   | ✅ KEEP   | Disposition                               |
| `ex.rs`                | ~50   | ✅ KEEP   | Exposure                                  |
| `ie.rs`                | ~60   | ✅ KEEP   | Inclusion/Exclusion                       |
| `lb.rs`                | ~130  | ✅ KEEP   | Laboratory (most complex)                 |
| `mh.rs`                | ~40   | ✅ KEEP   | Medical History                           |
| `pe.rs`                | ~40   | ✅ KEEP   | Physical Exam                             |
| `pr.rs`                | ~50   | ✅ KEEP   | Procedures                                |
| `qs.rs`                | ~80   | ✅ KEEP   | Questionnaires                            |
| `se.rs`                | ~30   | ✅ KEEP   | Subject Elements                          |
| `ta.rs`                | ~20   | ✅ KEEP   | Trial Arms                                |
| `te.rs`                | ~20   | ✅ KEEP   | Trial Elements                            |
| `ts.rs`                | ~80   | ✅ KEEP   | Trial Summary                             |
| `vs.rs`                | ~80   | ✅ KEEP   | Vital Signs                               |

**Summary:** 21 KEEP, 1 MERGE, 1 REVIEW

---

### sdtm-standards (5 files)

| File            | Lines | Decision | Justification         |
| --------------- | ----- | -------- | --------------------- |
| `lib.rs`        | ~35   | ✅ KEEP  | Module exports        |
| `csv_utils.rs`  | ~100  | ✅ KEEP  | CSV reading utilities |
| `loaders.rs`    | ~200  | ✅ KEEP  | Load SDTMIG domains   |
| `ct_loader.rs`  | ~250  | ✅ KEEP  | Load CT codelists     |
| `p21_loader.rs` | ~150  | ✅ KEEP  | Load P21 rules        |

**Summary:** 5 KEEP - **Clean crate, no changes needed**

---

### sdtm-validate (1 file)

| File     | Lines | Decision | Justification                  |
| -------- | ----- | -------- | ------------------------------ |
| `lib.rs` | 969   | 📐 SPLIT | Too large - split into modules |

**Proposed Split:**

```
sdtm-validate/src/
├── lib.rs              (~100 lines) - Exports + validate_domain()
├── ct.rs               (~200 lines) - CT validation checks
├── presence.rs         (~200 lines) - Required/Expected variable checks
├── format.rs           (~150 lines) - Date format, text length
├── consistency.rs      (~150 lines) - Sequence uniqueness
└── gating.rs           (~100 lines) - GatingDecision, gate_strict_outputs()
```

**Summary:** 1 SPLIT

---

### sdtm-map (5 files)

| File            | Lines | Decision | Justification                |
| --------------- | ----- | -------- | ---------------------------- |
| `lib.rs`        | ~50   | ✅ KEEP  | Module exports               |
| `engine.rs`     | ~300  | ✅ KEEP  | `MappingEngine` - core logic |
| `patterns.rs`   | ~150  | ✅ KEEP  | Synonym tables               |
| `repository.rs` | ~150  | ✅ KEEP  | Save/load mappings           |
| `utils.rs`      | ~50   | ✅ KEEP  | String utilities             |

**Summary:** 5 KEEP - **Clean crate, no changes needed**

---

### sdtm-report (6 files)

| File             | Lines | Decision | Justification                      |
| ---------------- | ----- | -------- | ---------------------------------- |
| `lib.rs`         | ~20   | ✅ KEEP  | Module exports                     |
| `common.rs`      | ~100  | ✅ KEEP  | Shared utilities                   |
| `xpt.rs`         | ~150  | ✅ KEEP  | XPT output (delegates to sdtm-xpt) |
| `dataset_xml.rs` | ~400  | ✅ KEEP  | Dataset-XML output                 |
| `define_xml.rs`  | ~250  | ✅ KEEP  | Define-XML output                  |
| `sas.rs`         | ~150  | ✅ KEEP  | SAS program output                 |

**Summary:** 6 KEEP - **Clean crate, no changes needed**

---

### sdtm-xpt (1 file)

| File     | Lines | Decision | Justification                             |
| -------- | ----- | -------- | ----------------------------------------- |
| `lib.rs` | 686   | ✅ KEEP  | XPT format reader/writer - self-contained |

**Summary:** 1 KEEP - **Clean crate, no changes needed**

---

### sdtm-gui (~15 files)

| Directory     | Files | Decision | Justification       |
| ------------- | ----- | -------- | ------------------- |
| `main.rs`     | 1     | ✅ KEEP  | Entry point         |
| `app.rs`      | 1     | ✅ KEEP  | Main app struct     |
| `theme.rs`    | 1     | ✅ KEEP  | UI theming          |
| `components/` | ~4    | ✅ KEEP  | Reusable widgets    |
| `views/`      | ~4    | ✅ KEEP  | Page views          |
| `state/`      | ~2    | ✅ KEEP  | App state           |
| `services/`   | ~2    | ✅ KEEP  | Background services |
| `dialogs/`    | ~1    | ✅ KEEP  | Modal dialogs       |

**Summary:** All KEEP - GUI-specific, no changes planned

---

## Type Inventory

### Types to Delete (2)

| Type        | Location                                | Why Delete                     |
| ----------- | --------------------------------------- | ------------------------------ |
| `SdtmError` | sdtm-model/error.rs                     | Unused, anyhow used everywhere |
| (functions) | sdtm-transform/normalization/numeric.rs | Duplicates sdtm-ingest         |

### Types to Rename (2)

| Current Name    | New Name         | Location               | Why Rename            |
| --------------- | ---------------- | ---------------------- | --------------------- |
| `StudyMetadata` | `SourceMetadata` | sdtm-model/metadata.rs | Avoid SDTMIG conflict |
| `StudyCodelist` | `SourceCodelist` | sdtm-model/metadata.rs | Avoid SDTMIG conflict |

### Types to Review (5)

| Type                   | Location                     | Issue               |
| ---------------------- | ---------------------------- | ------------------- |
| `ProcessStudyRequest`  | sdtm-model/processing.rs     | GUI-specific, move? |
| `ProcessStudyResponse` | sdtm-model/processing.rs     | GUI-specific, move? |
| `DomainResult`         | sdtm-model/processing.rs     | GUI-specific, move? |
| `OutputPaths`          | sdtm-model/processing.rs     | GUI-specific, move? |
| `ProcessorRegistry`    | sdtm-core/processor_trait.rs | Over-engineered?    |

### All Other Types: KEEP

All remaining ~55 types are essential and correctly placed.

---

## Redundancies & Removal Candidates

### 1. Duplicated Numeric Utilities 🗑️

**Files:**

- `sdtm-ingest/src/polars_utils.rs` ✅ (KEEP - canonical location)
- `sdtm-transform/src/normalization/numeric.rs` 🗑️ (DELETE)

**Functions duplicated:**

```rust
pub fn parse_f64(s: &str) -> Option<f64>
pub fn parse_i64(s: &str) -> Option<i64>
pub fn format_numeric(value: f64) -> String
```

**Action:** Delete `normalization/numeric.rs`, update imports to use
sdtm-ingest.

---

### 2. Unused Error Type 🗑️

**File:** `sdtm-model/src/error.rs`

**Content:**

```rust
pub enum SdtmError {
    Io(String),
    Message(String),
}
```

**Usage search result:** Zero callers. All error handling uses `anyhow::Result`.

**Action:** Delete file, remove from `lib.rs` exports.

---

### 3. Transforms Duplication 📦

**Files:**

- `sdtm-core/src/processor.rs` - Full pipeline processing
- `sdtm-core/src/transforms.rs` - Standalone functions

**Overlapping logic:**

- USUBJID prefixing
- Sequence assignment
- CT normalization

**Why both exist:** `transforms.rs` was added for GUI use without full pipeline
context.

**Recommendation:** Keep both but refactor processor.rs to call transforms.rs
internally.

---

### 4. ProcessorRegistry Complexity ⚠️

**Current design:**

```rust
pub struct ProcessorRegistry {
    processors: HashMap<&'static str, Box<dyn DomainProcessor>>,
    default_processor: Box<dyn DomainProcessor>,
}
```

**Reality:**

- 17 processors registered at compile time
- No runtime registration used
- No plugin system

**Could simplify to:**

```rust
fn process_domain(domain_code: &str, ...) -> Result<()> {
    match domain_code.to_uppercase().as_str() {
        "AE" => process_ae(...),
        "CM" => process_cm(...),
        // ...17 cases...
        _ => process_default(...),
    }
}
```

**Recommendation:** Keep for now - not broken. Simplify only if performance
matters.

---

## Action Plan

### Phase 1: Safe Deletions (Low Risk) ✅

| Step | File                                          | Action                    | Commands                                                |
| ---- | --------------------------------------------- | ------------------------- | ------------------------------------------------------- |
| 1.1  | `sdtm-model/src/error.rs`                     | Delete file               | `rm crates/sdtm-model/src/error.rs`                     |
| 1.2  | `sdtm-model/src/lib.rs`                       | Remove `pub mod error;`   | Edit file                                               |
| 1.3  | `sdtm-transform/src/normalization/numeric.rs` | Delete file               | `rm crates/sdtm-transform/src/normalization/numeric.rs` |
| 1.4  | `sdtm-transform/src/normalization/mod.rs`     | Remove `pub mod numeric;` | Edit file                                               |

### Phase 2: Type Renames (Medium Risk) 🔄

| Step | File                         | Old Name        | New Name                      |
| ---- | ---------------------------- | --------------- | ----------------------------- |
| 2.1  | `sdtm-model/src/metadata.rs` | `StudyMetadata` | `SourceMetadata`              |
| 2.2  | `sdtm-model/src/metadata.rs` | `StudyCodelist` | `SourceCodelist`              |
| 2.3  | All importers                | Update imports  | Find/replace across workspace |

### Phase 3: File Splitting (Medium Risk) 📐

| Step | File                       | Action               |
| ---- | -------------------------- | -------------------- |
| 3.1  | `sdtm-validate/src/lib.rs` | Split into 5 modules |

### Phase 4: Merging (Optional) 📦

| Step | File                          | Action                  |
| ---- | ----------------------------- | ----------------------- |
| 4.1  | `sdtm-core/src/transforms.rs` | Merge into processor.rs |

---

## Naming Convention Alignment

### SDTMIG v3.4 Terminology Check

| Codebase Term     | SDTMIG Term                     | Status     | Action               |
| ----------------- | ------------------------------- | ---------- | -------------------- |
| `Domain`          | Domain                          | ✅ Correct | None                 |
| `Variable`        | Variable                        | ✅ Correct | None                 |
| `DatasetClass`    | General Observation Class       | ✅ Correct | None                 |
| `Codelist`        | Controlled Terminology Codelist | ✅ Correct | None                 |
| `Term`            | Codelist Term                   | ✅ Correct | None                 |
| `SUPPQUAL`        | Supplemental Qualifier          | ✅ Correct | None                 |
| `RELREC`          | Related Records                 | ✅ Correct | None                 |
| `DomainFrame`     | N/A (internal)                  | ⚠️ OK      | Document as internal |
| `PipelineContext` | N/A (internal)                  | ⚠️ OK      | Document as internal |
| `StudyMetadata`   | ❌ CONFLICT                     | 🔄 Rename  | → `SourceMetadata`   |
| `StudyCodelist`   | ❌ CONFLICT                     | 🔄 Rename  | → `SourceCodelist`   |

### Why Rename Study* → Source*?

In SDTMIG, "Study" refers to the clinical trial itself:

- `STUDYID` - Study identifier variable
- Study metadata - Trial design, protocol info

Our `StudyMetadata` type represents **source data** from EDC systems:

- Items.csv - Source column definitions
- CodeLists.csv - Source value decoding

Renaming to `SourceMetadata` clarifies this is **input** data, not SDTM study
metadata.

---

## Summary Statistics

| Metric                | Before | After    |
| --------------------- | ------ | -------- |
| Source Files          | ~67    | ~65 (-2) |
| Public Types          | ~60    | ~58 (-2) |
| Duplicated Functions  | 3      | 0        |
| Lines in largest file | 969    | ~200     |
| Type naming conflicts | 2      | 0        |

---

## Approval Checklist

Before executing any changes:

- [ ] Review Phase 1 deletions (error.rs, numeric.rs)
- [ ] Review Phase 2 renames (StudyMetadata → SourceMetadata)
- [ ] Review Phase 3 splits (sdtm-validate)
- [ ] Approve execution order
- [ ] Verify test coverage before/after
- [ ] Run `cargo build` and `cargo test` between phases
