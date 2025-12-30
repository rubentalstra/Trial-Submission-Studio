# Plan: Consolidate and Improve CT Normalization

## Overview

Refactor CT (Controlled Terminology) normalization to:
1. **Add smart fallback logic**: Map invalid values to "OTHER" when appropriate
2. **Consolidate scattered code**: Move all normalization to one location
3. **Simplify for manual mapping workflow**: Remove unnecessary complexity

---

## Problem Analysis

### Current RACE Example
- Input: `"White, Caucasian, or Arabic"`
- Current behavior: Returns unchanged (no match found)
- Expected behavior: Should normalize to `"OTHER"`

### SDTMIG v3.4 Guidance (Chapter 5)
> "If a subject...selected 'Other'. RACE was populated with 'OTHER'"
> "For subjects who refuse to provide or do not know their race information, the value of RACE should be set to 'UNKNOWN' or 'NOT REPORTED'"

### RACE Codelist (C74457)
- **Non-extensible** (invalid values = errors)
- Valid values: AMERICAN INDIAN OR ALASKA NATIVE, ASIAN, BLACK OR AFRICAN AMERICAN, NATIVE HAWAIIAN OR OTHER PACIFIC ISLANDER, NOT REPORTED, OTHER, UNKNOWN, WHITE

### Current Code Locations (Scattered)
1. `sdtm-core/src/ct_utils.rs` - Core normalization
2. `sdtm-core/src/transforms.rs` - GUI transforms (duplicated logic)
3. `sdtm-core/src/domain_processors/operations.rs` - Pipeline batch ops
4. `sdtm-core/src/processor.rs` - Main pipeline normalization

---

## Solution Design

### New Normalization Strategy

```
normalize_ct_value(codelist, raw_value, options) → String
  1. Try exact match (submission value)
  2. Try synonym match
  3. Try lenient match (compact key)
  4. If no match AND codelist has "OTHER" term → return "OTHER"
  5. If no match AND codelist has "UNKNOWN" term → return "UNKNOWN"
  6. Else return original (for extensible codelists)
```

**Key insight**: For non-extensible codelists with "OTHER"/"UNKNOWN" terms, unmatched values should fallback to these rather than staying invalid.

### New Options Struct

```rust
pub struct NormalizationOptions {
    /// Matching strictness
    pub matching_mode: CtMatchingMode,
    /// Whether to fallback to OTHER/UNKNOWN for invalid values
    pub use_fallback_term: bool,
}
```

---

## Files to Modify

### 1. `sdtm-model/src/ct.rs` (Add Helper Methods)

**Add to `Codelist`:**
```rust
/// Get fallback term for invalid values (OTHER or UNKNOWN)
pub fn fallback_term(&self) -> Option<&str> {
    // Prefer OTHER, then UNKNOWN
    if self.terms.contains_key("OTHER") {
        Some("OTHER")
    } else if self.terms.contains_key("UNKNOWN") {
        Some("UNKNOWN")
    } else {
        None
    }
}
```

### 2. `sdtm-core/src/ct_utils.rs` (Primary - Consolidate Here)

**Current functions:**
- `normalize_ct_value()` - Core logic
- `resolve_ct_value()` - Matching dispatch
- `resolve_ct_value_strict()` - Exact/synonym only
- `resolve_ct_value_lenient()` - With compact key

**Changes:**
- Add `NormalizationOptions` struct with `Default` impl
- Update `normalize_ct_value()` to use fallback when no match
- Make functions `pub` (currently `pub(crate)`)

### 3. `sdtm-core/src/transforms.rs` (Remove Duplication)

**Current:** Has its own `normalize_ct_column()` that duplicates logic
**Change:** Call `ct_utils::normalize_ct_value()` instead

### 4. `sdtm-core/src/domain_processors/operations.rs` (Simplify)

**Current:** Complex `normalize_ct_columns()` function
**Change:** Delegate to centralized `ct_utils` functions

### 5. `sdtm-core/src/pipeline_context.rs` (Add Option)

**Add to `ProcessingOptions`:**
```rust
pub use_fallback_terms: bool,  // Default: true
```

---

## Implementation Steps

### Step 1: Add Codelist Helper (sdtm-model)
- Add `Codelist::fallback_term()` method
- Returns "OTHER" if present, else "UNKNOWN" if present, else None

### Step 2: Enhance ct_utils.rs
- Add `NormalizationOptions` struct with Default impl
- Update `normalize_ct_value()` to accept options
- Add fallback logic: if no match and `use_fallback_term`, return `codelist.fallback_term()`
- Make functions public

### Step 3: Update transforms.rs
- Remove duplicated normalization logic
- Call centralized `ct_utils::normalize_ct_value()`
- Keep `normalize_ct_column()` as a thin wrapper for DataFrames

### Step 4: Update domain_processors/operations.rs
- Simplify `normalize_ct_columns()` to use centralized logic

### Step 5: Update pipeline_context.rs
- Add `use_fallback_terms` option (default: true)

### Step 6: Tests
- Add test for RACE "White, Caucasian, or Arabic" → "OTHER"
- Add test for codelist without OTHER/UNKNOWN (returns original)
- Update existing tests for new signature

---

## API Changes

### Before
```rust
normalize_ct_value(ct: &Codelist, raw: &str, mode: CtMatchingMode) -> String
```

### After
```rust
normalize_ct_value(ct: &Codelist, raw: &str, options: &NormalizationOptions) -> String

pub struct NormalizationOptions {
    pub matching_mode: CtMatchingMode,
    pub use_fallback_term: bool,  // Default: true
}

impl Default for NormalizationOptions {
    fn default() -> Self {
        Self {
            matching_mode: CtMatchingMode::Lenient,
            use_fallback_term: true,
        }
    }
}
```

---

## Files Summary

| File | Action | Purpose |
|------|--------|---------|
| `sdtm-model/src/ct.rs` | Modify | Add `fallback_term()` method |
| `sdtm-core/src/ct_utils.rs` | Modify | Add NormalizationOptions, fallback logic, make public |
| `sdtm-core/src/transforms.rs` | Modify | Remove duplication, use ct_utils |
| `sdtm-core/src/domain_processors/operations.rs` | Modify | Simplify to use centralized logic |
| `sdtm-core/src/pipeline_context.rs` | Modify | Add use_fallback_terms option |

---

## Expected Behavior After Implementation

```
Input: "White, Caucasian, or Arabic"
Codelist: RACE (C74457, non-extensible)
Steps:
  1. Exact match? No
  2. Synonym match? No
  3. Lenient match? No
  4. use_fallback_term? Yes
  5. codelist.fallback_term()? "OTHER"
  6. Return "OTHER"
Result: "OTHER"
```

---

## Backward Compatibility

- Existing callers of `normalize_ct_value()` will need to pass options
- Default options maintain current lenient behavior + add fallback
- Can disable fallback with `use_fallback_term: false` if needed
