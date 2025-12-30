# Plan: Consolidate and Improve CT Normalization

## Overview

Refactor CT (Controlled Terminology) normalization to:

1. **Centralize Normalization Logic**: Move all CT normalization logic from
   `sdtm-core` to `sdtm-transform` (alongside existing `datetime`
   normalization).
2. **Add Smart Fallback Logic**: Map invalid values to "OTHER" or "UNKNOWN" when
   appropriate (e.g., "White, Caucasian, or Arabic" -> "OTHER").
3. **Simplify Architecture**: Remove scattered helper modules in `sdtm-core` and
   rely on the dedicated `sdtm-transform` crate.
4. **Standardize API**: Create a consistent `NormalizationOptions` interface for
   all normalization types.

---

## Problem Analysis

### Current Architecture Issues

- **Scattered Logic**: CT normalization lives in `sdtm-core/src/ct_utils.rs`,
  while Date normalization lives in `sdtm-transform/src/datetime.rs`.
- **Inconsistent Behavior**: `RACE` example ("White, Caucasian, or Arabic")
  fails because strict/lenient matching doesn't handle semantic fallbacks.
- **Duplication**: `sdtm-core/src/transforms.rs` and
  `sdtm-core/src/domain_processors/operations.rs` both implement similar logic.

### The `RACE` Example

- **Input**: `"White, Caucasian, or Arabic"`
- **Current Result**: Invalid (no match).
- **Desired Result**: `"OTHER"` (because `RACE` codelist has "OTHER" and is
  non-extensible).
- **Rule**: If a value is invalid, but the codelist allows "OTHER", fallback to
  "OTHER". If "UNKNOWN" is allowed, fallback to "UNKNOWN".

---

## Solution Design

### 1. Move to `sdtm-transform`

We will move `sdtm-core/src/ct_utils.rs` to `sdtm-transform/src/ct.rs`. This
consolidates all "normalization" (dates, CT, etc.) into one crate.

### 2. New Normalization Strategy

```rust
pub struct NormalizationOptions {
    /// Matching strictness (Strict vs Lenient)
    pub matching_mode: CtMatchingMode,
    /// Whether to fallback to OTHER/UNKNOWN for invalid values
    pub use_fallback_term: bool,
}

pub fn normalize_ct_value(codelist: &Codelist, raw_value: &str, options: &NormalizationOptions) -> String {
    // 1. Try exact match
    // 2. Try synonym match
    // 3. Try lenient match (compact key)
    // 4. If no match AND use_fallback_term:
    //    a. If codelist has "OTHER" -> return "OTHER"
    //    b. If codelist has "UNKNOWN" -> return "UNKNOWN"
    // 5. Return original (for extensible codelists or if no fallback)
}
```

### 3. Codelist Helper

Add `fallback_term()` to `Codelist` in `sdtm-model`.

---

## Implementation Steps

### Step 1: Enhance `sdtm-model`

- **File**: `crates/sdtm-model/src/ct.rs`
- **Action**: Add `fallback_term(&self) -> Option<&str>` to `Codelist` struct.

### Step 2: Expand `sdtm-transform`

- **File**: `crates/sdtm-transform/src/ct.rs` (New File)
- **Action**:
  - Port logic from `sdtm-core/src/ct_utils.rs`.
  - Implement `NormalizationOptions`.
  - Implement the fallback logic.
- **File**: `crates/sdtm-transform/src/lib.rs`
  - **Action**: Export `ct` module.

### Step 3: Clean up `sdtm-core`

- **File**: `crates/sdtm-core/src/ct_utils.rs`
  - **Action**: Delete this file.
- **File**: `crates/sdtm-core/src/transforms.rs`
  - **Action**: Update imports to use `sdtm_transform::ct`.
- **File**: `crates/sdtm-core/src/domain_processors/operations.rs`
  - **Action**: Update imports and usage.
- **File**: `crates/sdtm-core/src/pipeline_context.rs`
  - **Action**: Add `use_fallback_terms` to `ProcessingOptions`.

---

## API Changes

### `sdtm-transform::ct`

```rust
pub struct NormalizationOptions {
    pub matching_mode: CtMatchingMode,
    pub use_fallback_term: bool,
}

impl Default for NormalizationOptions {
    fn default() -> Self {
        Self {
            matching_mode: CtMatchingMode::Lenient,
            use_fallback_term: true,
        }
    }
}

pub fn normalize_ct_value(ct: &Codelist, raw: &str, options: &NormalizationOptions) -> String;
```

---

## Verification

- **Test Case**: `RACE` = "White, Caucasian, or Arabic" -> "OTHER".
- **Test Case**: `SEX` = "M" -> "M" (Exact).
- **Test Case**: `SEX` = "Male" -> "M" (Synonym/Lenient).
- **Test Case**: Invalid value in extensible codelist -> Original value.
