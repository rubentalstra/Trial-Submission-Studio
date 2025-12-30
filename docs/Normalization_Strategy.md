# SDTM Normalization Strategy

## 1. Executive Summary

This document outlines the strategy for normalizing clinical data to SDTM
standards. The goal is to ensure data conformance (ISO 8601 dates, Controlled
Terminology) while preserving data integrity and avoiding unsafe "guessing".

The strategy introduces a centralized **Normalization Engine** within
`sdtm-transform` that handles:

1. **Controlled Terminology (CT)**: Exact matching, synonym resolution, and safe
   fallbacks.
2. **Date/Time**: ISO 8601 conversion and partial date handling.
3. **Numeric**: Standardization of numeric results and units.

## 2. The `RACE` Case Study

### Problem

Input value: `"White, Caucasian, or Arabic"` Target Variable: `DM.RACE`
Codelist: `C74457` (Race), Non-extensible.

### Analysis

- **Strict Matching**: Fails. Value is not in the allowed list.
- **Synonym Matching**: Fails. "Caucasian" might map to "WHITE", but "Arabic" is
  distinct (though FDA defines it under White).
- **SDTMIG Guidance (Chapter 5)**:
  > "If race was collected via an 'Other, Specify' field and the sponsor chooses
  > not to map the value... then the value of RACE should be 'OTHER'."
- **Codelist Definition**: Contains `OTHER` ("Different than the one(s)
  previously specified...").

### Solution

For `RACE`, if the value cannot be mapped to a specific category (like "WHITE")
via a synonym or manual map, it **must** be mapped to `"OTHER"` to be compliant,
as the codelist is non-extensible. Leaving it as-is would be a conformance
error.

## 3. Normalization Logic

The normalization process will follow a strict hierarchy of operations:

### 3.1. Controlled Terminology (CT)

For a given value `v` and codelist `C`:

1. **Manual Mapping (Highest Priority)**
   - Check if the user has provided a specific map for this value (e.g., in a
     `mappings.csv`).
   - _Example_: `"White, Caucasian, or Arabic" -> "WHITE"` (if user decides).

2. **Exact Match**
   - Check if `v` exists in `C` (case-insensitive).
   - _Example_: `"Asian" -> "ASIAN"`.

3. **Synonym Match**
   - Check if `v` is a known synonym in `C`.
   - _Example_: `"Caucasian" -> "WHITE"` (if "Caucasian" is a synonym in CT).

4. **Safe Fallback (Configurable)**
   - If `C` is **Non-Extensible**:
     - If `C` contains `"OTHER"` and `v` is clearly an "other" value (or user
       opts-in to fallback): Map to `"OTHER"`.
     - If `C` contains `"UNKNOWN"` and `v` indicates unknown (e.g., "?"): Map to
       `"UNKNOWN"`.
   - _Note_: This step must be used cautiously. For `RACE`, it is compliant. For
     other domains, it might obscure data.

5. **Failure Handling**
   - **Extensible Codelist**: Keep original value `v` (Warning).
   - **Non-Extensible Codelist**:
     - If fallback failed: Report **Error** (Invalid Term).

### 3.2. Date/Time (ISO 8601)

- **Input**: Raw date strings (e.g., "10/02/2023", "2023-Feb-10").
- **Process**:
  1. Attempt to parse using standard formats.
  2. Convert to ISO 8601 (`YYYY-MM-DD`).
  3. Handle partial dates (e.g., "2023-02") if allowed.
- **Failure**: Keep original, flag as invalid format.

## 4. Architecture

We will consolidate all normalization logic into
`crates/sdtm-transform/src/normalization/`.

```
crates/sdtm-transform/
├── src/
│   ├── lib.rs
│   ├── normalization/
│   │   ├── mod.rs          # Entry point
│   │   ├── ct.rs           # CT logic (Codelist, Synonyms, Fallbacks)
│   │   ├── datetime.rs     # ISO 8601 logic
│   │   ├── numeric.rs      # Unit/Value standardization
│   │   └── options.rs      # Configuration structs
```

### `NormalizationOptions`

```rust
pub struct NormalizationOptions {
    /// Allow mapping invalid non-extensible values to "OTHER" if available.
    /// Default: false (Safety first).
    pub enable_other_fallback: bool,
    
    /// Allow mapping unknown-like values to "UNKNOWN" if available.
    /// Default: true.
    pub enable_unknown_fallback: bool,
    
    /// Custom value mappings (Raw -> Submission Value).
    pub custom_maps: HashMap<String, String>,
}
```

## 5. Implementation Plan

1. **Refactor `sdtm-transform`**: Create the `normalization` module structure.
2. **Migrate CT Logic**: Move `ct_utils.rs` logic to `normalization/ct.rs`.
3. **Implement Fallback Logic**: Add the "Safe Fallback" logic described above,
   specifically handling the `RACE` case.
4. **Integrate Custom Maps**: Add support for user-defined mappings in the
   options.
5. **Update `sdtm-core`**: Update domain processors to use the new engine.
6. **Verify**: Test with the `RACE` example and other edge cases.

## 6. Conclusion

This strategy avoids "guessing" by prioritizing explicit mappings and synonyms.
The "OTHER" fallback is treated as a safety net for non-extensible codelists
(like `RACE`) where the alternative is a conformance error, but it is gated
behind configuration to ensure user intent.
