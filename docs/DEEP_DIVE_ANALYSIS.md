# CDISC Transpiler - Deep Dive Codebase Analysis

**Generated:** 2025-12-30\
**Updated:** 2025-12-30 (Post sdtm-core removal)\
**Philosophy:** "Can we remove it? If not, why not? Is it really needed?"

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Crate Dependency Graph](#crate-dependency-graph)
3. [File-by-File Analysis](#file-by-file-analysis)
4. [Dead Code Inventory](#dead-code-inventory)
5. [Pending Renames](#pending-renames)
6. [Action Plan](#action-plan)

---

## Executive Summary

### Recent Changes

✅ **DELETED:** `sdtm-core` crate (~3,600 lines) - Pipeline infrastructure was
never used by GUI\
✅ **MOVED:** `transforms.rs` (456 lines) → `sdtm-transform/src/transforms.rs`

### Architecture Overview (Current State)

The codebase now has **9 crates** with clear layered architecture:

| Layer          | Crate            | Files | Lines  | Used By        | Status              |
| -------------- | ---------------- | ----- | ------ | -------------- | ------------------- |
| **Foundation** | `sdtm-model`     | 10    | 1,522  | All crates     | ⚠️ Has dead code    |
| **Data I/O**   | `sdtm-ingest`    | 5     | 1,063  | GUI, transform | ✅ Clean            |
| **Data I/O**   | `sdtm-xpt`       | 1     | 685    | Future: output | 🔮 Future feature   |
| **Standards**  | `sdtm-standards` | 5     | 540    | GUI            | ✅ Clean            |
| **Transform**  | `sdtm-transform` | 11    | 2,888  | GUI, output    | ⚠️ Has dead code    |
| **Mapping**    | `sdtm-map`       | 5     | 1,438  | GUI            | ✅ Clean            |
| **Validation** | `sdtm-validate`  | 1     | 968    | GUI            | ⚠️ Has dead code    |
| **Output**     | `sdtm-report`    | 6     | 1,411  | Future: GUI    | 🔮 Rename to output |
| **Frontend**   | `sdtm-gui`       | ~15   | ~6,000 | Entry point    | ✅ Clean            |

**Total:** ~16,500 lines of Rust code (down from ~20,000)

### Key Findings

#### 🗑️ Dead Code (Can Remove Now)

| Category    | Item                                      | Lines | Action              |
| ----------- | ----------------------------------------- | ----- | ------------------- |
| 🗑️ **FILE** | `sdtm-model/src/error.rs`                 | 22    | Delete - unused     |
| 🗑️ **FILE** | `sdtm-transform/normalization/numeric.rs` | 30    | Delete - duplicates |
| 🗑️ **FILE** | `sdtm-transform/suppqual.rs`              | 329   | Delete - unused     |
| 🗑️ **FILE** | `sdtm-transform/relationships.rs`         | 554   | Delete - unused     |
| 🗑️ **FILE** | `sdtm-transform/frame_builder.rs`         | 161   | Delete - unused     |
| 🗑️ **CODE** | `sdtm-validate` gating functions          | ~60   | Delete - unused     |

#### 🔮 Future Features (Keep But Not Wired Up Yet)

| Crate         | Lines | Purpose                              | Status                              |
| ------------- | ----- | ------------------------------------ | ----------------------------------- |
| `sdtm-report` | 1,411 | Export: XPT, Dataset-XML, Define-XML | 🏷️ Rename to `sdtm-output`          |
| `sdtm-xpt`    | 685   | XPT file generation for FDA          | ✅ Keep - needed for FDA submission |

#### ⚠️ Review Needed

| Item                           | Issue                                     |
| ------------------------------ | ----------------------------------------- |
| `sdtm-model/src/processing.rs` | Contains `OutputFormat` - move or keep?   |
| GUI dep on `sdtm-report`       | Listed but not imported - remove for now? |

### Summary Counts

| Metric          | Before Core Removal | Current | After Cleanup |
| --------------- | ------------------- | ------- | ------------- |
| Crates          | 10                  | 9       | **9**         |
| Source Lines    | ~20,000             | ~16,500 | **~15,350**   |
| Dead Code Lines | ~3,600              | ~1,156  | **0**         |

**Potential savings: ~1,156 lines of actual dead code (~7% of remaining code)**

---

## Crate Dependency Graph

### Current State

```
sdtm-model (foundation)          ⚠️ has error.rs dead code
    │
    ├── sdtm-ingest              ✅ clean
    │       │
    │       └── sdtm-transform   ⚠️ has dead: suppqual, relationships, frame_builder, numeric
    │               │
    │               └── sdtm-output (currently: sdtm-report)  🔮 FUTURE FEATURE
    │                       │
    │                       └── sdtm-xpt  🔮 FUTURE: FDA XPT generation
    │
    ├── sdtm-standards           ✅ clean
    │
    ├── sdtm-map                 ✅ clean
    │
    ├── sdtm-validate            ⚠️ has dead gating code
    │
    └── sdtm-gui                 ✅ (will use sdtm-output when ready)
```

### After Cleanup + Rename

```
sdtm-model (foundation)          ✅ 
    │
    ├── sdtm-ingest              ✅
    │       │
    │       └── sdtm-transform   ✅ (smaller, cleaner)
    │               │
    │               └── sdtm-output  🔮 (renamed from sdtm-report)
    │                       │
    │                       └── sdtm-xpt  🔮 (FDA XPT files)
    │
    ├── sdtm-standards           ✅
    │
    ├── sdtm-map                 ✅
    │
    ├── sdtm-validate            ✅ (smaller)
    │
    └── sdtm-gui                 ✅
```

---

## File-by-File Analysis

### Decision Key

| Symbol        | Meaning                          |
| ------------- | -------------------------------- |
| ✅ **KEEP**   | Used, no changes needed          |
| 🗑️ **DELETE** | Can be completely removed        |
| 🔮 **FUTURE** | Not wired up yet, keep for later |
| 🏷️ **RENAME** | Needs renaming for clarity       |

---

### sdtm-model (1,522 lines)

| File             | Lines | Decision  | Justification                               |
| ---------------- | ----- | --------- | ------------------------------------------- |
| `lib.rs`         | ~50   | ✅ KEEP   | Module exports                              |
| `domain.rs`      | ~270  | ✅ KEEP   | `Domain`, `Variable`, `DatasetClass`        |
| `ct.rs`          | ~285  | ✅ KEEP   | `Codelist`, `Term`, `TerminologyRegistry`   |
| `conformance.rs` | ~220  | ✅ KEEP   | `ValidationReport`, `ValidationIssue`       |
| `p21.rs`         | ~265  | ✅ KEEP   | `P21Rule`, `P21Category`                    |
| `options.rs`     | ~120  | ✅ KEEP   | `ProcessingOptions`, `NormalizationOptions` |
| `metadata.rs`    | ~100  | ✅ KEEP   | `SourceColumn`, `StudyMetadata`             |
| `mapping.rs`     | ~50   | ✅ KEEP   | `MappingSuggestion`, `MappingConfig`        |
| `lookup.rs`      | ~80   | ✅ KEEP   | `CaseInsensitiveSet`                        |
| `error.rs`       | ~22   | 🗑️ DELETE | `SdtmError` - **NEVER USED**, use anyhow    |
| `processing.rs`  | ~90   | ⚠️ REVIEW | `OutputFormat` used by validate tests       |

**Dead code in sdtm-model: ~22 lines** (error.rs only)

---

### sdtm-ingest (1,063 lines)

| File                | Lines | Decision | Justification                |
| ------------------- | ----- | -------- | ---------------------------- |
| `lib.rs`            | ~25   | ✅ KEEP  | Module exports               |
| `csv_table.rs`      | ~250  | ✅ KEEP  | CSV reading                  |
| `discovery.rs`      | ~150  | ✅ KEEP  | Domain file discovery        |
| `polars_utils.rs`   | ~120  | ✅ KEEP  | `any_to_string`, `parse_f64` |
| `study_metadata.rs` | ~400  | ✅ KEEP  | `AppliedStudyMetadata`       |

**Dead code in sdtm-ingest: 0 lines** ✅

---

### sdtm-transform (2,888 lines)

| File                        | Lines | Decision  | Justification                                |
| --------------------------- | ----- | --------- | -------------------------------------------- |
| `lib.rs`                    | ~25   | ✅ KEEP   | Module exports                               |
| `transforms.rs`             | ~456  | ✅ KEEP   | `build_preview_dataframe` - **USED BY GUI**  |
| `data_utils.rs`             | ~200  | ✅ KEEP   | String manipulation                          |
| `frame.rs`                  | ~105  | ✅ KEEP   | `DomainFrame` - **USED BY OUTPUT**           |
| `domain_sets.rs`            | ~100  | ✅ KEEP   | `domain_map_by_code` - **USED BY OUTPUT**    |
| `normalization/mod.rs`      | ~10   | ✅ KEEP   | Module exports                               |
| `normalization/ct.rs`       | ~300  | ✅ KEEP   | CT normalization                             |
| `normalization/datetime.rs` | ~650  | ✅ KEEP   | ISO 8601 parsing                             |
| `normalization/numeric.rs`  | ~30   | 🗑️ DELETE | **DUPLICATES** sdtm-ingest/polars_utils      |
| `suppqual.rs`               | ~329  | 🗑️ DELETE | **NEVER CALLED** from outside crate          |
| `relationships.rs`          | ~554  | 🗑️ DELETE | **NEVER CALLED** from outside crate          |
| `frame_builder.rs`          | ~161  | 🗑️ DELETE | **ONLY USED BY** dead suppqual/relationships |

**Dead code in sdtm-transform: ~1,074 lines**

---

### sdtm-standards (540 lines)

| File            | Lines | Decision | Justification       |
| --------------- | ----- | -------- | ------------------- |
| `lib.rs`        | ~35   | ✅ KEEP  | Module exports      |
| `csv_utils.rs`  | ~100  | ✅ KEEP  | CSV reading         |
| `loaders.rs`    | ~200  | ✅ KEEP  | Load SDTMIG domains |
| `ct_loader.rs`  | ~150  | ✅ KEEP  | Load CT codelists   |
| `p21_loader.rs` | ~55   | ✅ KEEP  | Load P21 rules      |

**Dead code in sdtm-standards: 0 lines** ✅

---

### sdtm-validate (968 lines)

| File     | Lines | Decision | Justification                             |
| -------- | ----- | -------- | ----------------------------------------- |
| `lib.rs` | 968   | ⚠️ MIXED | Only `validate_domain` is used externally |

**Functions in sdtm-validate:**

| Function                   | Lines | Used Outside? | Decision  |
| -------------------------- | ----- | ------------- | --------- |
| `validate_domain`          | ~800  | ✅ YES (GUI)  | ✅ KEEP   |
| `validate_domains`         | ~20   | ❌ NO         | 🗑️ DELETE |
| `GatingDecision` struct    | ~15   | ❌ NO         | 🗑️ DELETE |
| `strict_outputs_requested` | ~5    | ❌ NO         | 🗑️ DELETE |
| `gate_strict_outputs`      | ~20   | ❌ NO         | 🗑️ DELETE |

**Dead code in sdtm-validate: ~60 lines**

---

### sdtm-map (1,438 lines)

| File            | Lines | Decision | Justification      |
| --------------- | ----- | -------- | ------------------ |
| `lib.rs`        | ~50   | ✅ KEEP  | Module exports     |
| `engine.rs`     | ~800  | ✅ KEEP  | `MappingEngine`    |
| `patterns.rs`   | ~350  | ✅ KEEP  | Synonym tables     |
| `repository.rs` | ~150  | ✅ KEEP  | Save/load mappings |
| `utils.rs`      | ~88   | ✅ KEEP  | String utilities   |

**Dead code in sdtm-map: 0 lines** ✅

---

### sdtm-report → sdtm-output (1,411 lines) 🔮 FUTURE FEATURE

| File             | Lines | Decision  | Justification              |
| ---------------- | ----- | --------- | -------------------------- |
| `lib.rs`         | ~20   | 🔮 FUTURE | Module exports             |
| `common.rs`      | ~100  | 🔮 FUTURE | Shared utilities           |
| `xpt.rs`         | ~400  | 🔮 FUTURE | XPT output (uses sdtm-xpt) |
| `dataset_xml.rs` | ~450  | 🔮 FUTURE | Dataset-XML output         |
| `define_xml.rs`  | ~300  | 🔮 FUTURE | Define-XML 2.1 output      |
| `sas.rs`         | ~141  | 🔮 FUTURE | SAS program generation     |

**Status:** Not wired up to GUI yet. Will be used after
mapping/transform/validation flow is complete.

**Action:** Rename crate from `sdtm-report` → `sdtm-output` for clarity.

---

### sdtm-xpt (685 lines) 🔮 FUTURE FEATURE

| File     | Lines | Decision  | Justification                      |
| -------- | ----- | --------- | ---------------------------------- |
| `lib.rs` | 685   | 🔮 FUTURE | XPT file format for FDA submission |

**Status:** Used by sdtm-output (sdtm-report). Required for regulatory
submissions.

**Keep:** This is essential for FDA compliance - XPT is the required format.

---

### sdtm-gui (~6,000 lines)

| Area         | Status   | Issue                                      |
| ------------ | -------- | ------------------------------------------ |
| `Cargo.toml` | ⚠️ CHECK | Has `sdtm-report` dep - remove until ready |
| Source files | ✅ KEEP  | All actively used                          |

---

## Dead Code Inventory

### Summary by Category

| Category           | Lines  | Files/Items                                                           |
| ------------------ | ------ | --------------------------------------------------------------------- |
| **Entire Files**   | ~1,096 | numeric.rs, suppqual.rs, relationships.rs, frame_builder.rs, error.rs |
| **Dead Functions** | ~60    | validate gating code                                                  |
| **Total**          | ~1,156 | ~7% of current codebase                                               |

### NOT Dead Code (Clarification)

| Crate         | Lines | Why Keep                                       |
| ------------- | ----- | ---------------------------------------------- |
| `sdtm-report` | 1,411 | Future: export functionality not wired yet     |
| `sdtm-xpt`    | 685   | Future: FDA XPT format required for submission |

---

## Pending Renames

### Crate Rename: sdtm-report → sdtm-output

**Rationale:**

- "report" implies read-only summary/display
- "output" better describes export/generation functionality
- Clearer that this generates submission files

**Files to Update:**

1. `crates/sdtm-report/` → `crates/sdtm-output/`
2. `crates/sdtm-output/Cargo.toml` - update package name
3. `Cargo.toml` - update workspace members
4. `crates/sdtm-gui/Cargo.toml` - update dependency (when wired up)

---

## Action Plan

### Phase 1: Clean Dead Code (~1,156 lines)

```bash
# Delete dead files in sdtm-transform
rm crates/sdtm-transform/src/normalization/numeric.rs
rm crates/sdtm-transform/src/suppqual.rs
rm crates/sdtm-transform/src/relationships.rs
rm crates/sdtm-transform/src/frame_builder.rs

# Delete error.rs in sdtm-model
rm crates/sdtm-model/src/error.rs

# Update lib.rs files to remove exports
# Update normalization/mod.rs to remove numeric

cargo check --all
```

### Phase 2: Clean sdtm-validate (~60 lines)

```bash
# Remove dead functions from lib.rs:
# - GatingDecision struct
# - strict_outputs_requested
# - gate_strict_outputs  
# - validate_domains

cargo check --all
```

### Phase 3: Rename sdtm-report → sdtm-output

```bash
# Rename directory
mv crates/sdtm-report crates/sdtm-output

# Update Cargo.toml files
# - crates/sdtm-output/Cargo.toml: name = "sdtm-output"
# - Cargo.toml: update workspace members
# - crates/sdtm-gui/Cargo.toml: update or remove dep

cargo check --all
```

### Phase 4: Remove Unused GUI Dependency

```bash
# Edit crates/sdtm-gui/Cargo.toml
# Remove: sdtm-report = { path = "../sdtm-report" }
# (Will add back as sdtm-output when output is implemented)

cargo check --all
```

### Phase 5: Final Verification

```bash
cargo build --all
cargo test --all
cargo clippy --all -- -D warnings
```

---

## Final State After Cleanup

| Crate          | Lines   | Status               |
| -------------- | ------- | -------------------- |
| sdtm-model     | ~1,500  | ✅                   |
| sdtm-ingest    | ~1,063  | ✅                   |
| sdtm-transform | ~1,814  | ✅ (cleaned)         |
| sdtm-standards | ~540    | ✅                   |
| sdtm-validate  | ~908    | ✅ (cleaned)         |
| sdtm-map       | ~1,438  | ✅                   |
| sdtm-output    | ~1,411  | 🔮 (renamed, future) |
| sdtm-xpt       | ~685    | 🔮 (future)          |
| sdtm-gui       | ~6,000  | ✅                   |
| **Total**      | ~15,359 | ✅                   |

**Reduction: 16,500 → 15,359 lines (~7% smaller)**\
**Crates: 9 (with 1 rename)**

---

## Future Work (When Output is Ready)

When the mapping/transform/validation flow is complete:

1. Wire `sdtm-output` into GUI export functionality
2. Add export buttons/dialogs to GUI
3. Generate XPT files for FDA submission
4. Generate Dataset-XML and Define-XML
5. Optional: SAS program generation
