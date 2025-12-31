# Plan: Rewrite `sdtm-map` Crate from Scratch

## Summary

Rewrite `sdtm-map` as a minimal, clean crate for **manual column-to-SDTM-variable mapping**.

**Key decisions:**
- Full fuzzy matching (Jaro-Winkler)
- Centralize scoring in sdtm-map (GUI calls it, not reimplements)
- No persistence (session-only)
- Delete and rewrite from scratch

## Current Problems

| Issue | Impact |
|-------|--------|
| ~40% dead code (repository, merge utils) | Bloat |
| GUI reimplements scoring in `calculate_name_similarity()` | Duplication |
| 40+ hardcoded synonyms in `patterns.rs` | Not configurable |
| Opaque scoring (scores can exceed 1.0) | Confusing |
| No explainability | Can't debug scores |
| Broken timestamp generation | Bug |

## New Architecture

```
crates/sdtm-map/src/
├── lib.rs        # Public API (~20 lines)
├── error.rs      # MappingError enum (~25 lines)
├── score.rs      # ScoringEngine with Jaro-Winkler (~200 lines)
└── state.rs      # MappingState session-only (~150 lines)
```

**Deleted files:**
- `repository.rs` (232 lines) - No persistence needed
- `patterns.rs` (186 lines) - No synonyms needed
- `utils.rs` (65 lines) - Inline normalize()
- `types.rs` (47 lines) - Merge into other files

**Result: ~1100 lines → ~350 lines (68% reduction)**

## Public API

### Types

```rust
// Scoring
pub struct ColumnHint { is_numeric, unique_ratio, null_ratio, label }
pub struct ColumnScore { score: f32, explanation: Vec<ScoreComponent> }
pub struct ScoreComponent { name, value, description }
pub struct Suggestion { source_column, target_variable, score }

// State
pub struct MappingState { /* internal */ }
pub struct MappingSummary { total_variables, mapped, suggested, required_total, required_mapped }
pub enum VariableStatus { Accepted, Suggested, Unmapped }

// Export
pub struct MappingConfig { domain_code, study_id, mappings }
pub struct Mapping { source_column, target_variable, confidence }

// Errors
pub enum MappingError { VariableNotFound, ColumnNotFound, ColumnAlreadyUsed }
```

### ScoringEngine (centralized scoring)

```rust
impl ScoringEngine {
    pub fn new(domain: Domain, hints: BTreeMap<String, ColumnHint>) -> Self;

    /// Score a single column against a variable (for dropdown sorting)
    pub fn score(&self, column: &str, variable_name: &str) -> Option<ColumnScore>;

    /// Score all columns for a variable (sorted by score)
    pub fn score_all_for_variable(&self, variable_name: &str, columns: &[String]) -> Vec<(String, ColumnScore)>;

    /// Auto-suggest one-to-one mappings
    pub fn suggest_all(&self, columns: &[String], min_confidence: f32) -> Vec<Suggestion>;
}
```

### MappingState (session-only)

```rust
impl MappingState {
    pub fn new(domain, study_id, columns, hints, min_confidence) -> Self;

    // Accessors
    pub fn domain(&self) -> &Domain;
    pub fn scorer(&self) -> &ScoringEngine;  // For dropdown scoring
    pub fn status(&self, variable_name: &str) -> VariableStatus;
    pub fn suggestion(&self, variable_name: &str) -> Option<(&str, f32)>;
    pub fn accepted(&self, variable_name: &str) -> Option<(&str, f32)>;
    pub fn current_mapping(&self, variable_name: &str) -> Option<(&str, f32)>;
    pub fn available_columns(&self) -> Vec<&str>;
    pub fn summary(&self) -> MappingSummary;

    // Actions
    pub fn accept_suggestion(&mut self, variable_name: &str) -> Result<(), MappingError>;
    pub fn accept_manual(&mut self, variable_name: &str, column: &str) -> Result<(), MappingError>;
    pub fn clear(&mut self, variable_name: &str) -> bool;

    // Export
    pub fn to_config(&self) -> MappingConfig;
}
```

## Scoring Algorithm

**Pure Jaro-Winkler + basic adjustments (no synonyms):**

1. **Base: Jaro-Winkler similarity** (0.0-1.0)
   - Compare normalized column name vs variable name
   - Normalization: lowercase, replace `_-. ` with space, trim

2. **Label similarity boost** (+10%)
   - If column hint has label AND variable has label
   - Boost if label similarity >85%

3. **Suffix matching adjustments**
   - SEQ suffix: boost if both have it, penalty if mismatch
   - CD suffix: similar logic for code variables

4. **Type mismatch penalty** (-15%)
   - Numeric column vs character variable (or vice versa)
   - Based on ColumnHint.is_numeric vs variable name ending in 'N'

**No synonyms** - keep it simple and predictable.

**Explainability example:**
```
"Name similarity: 85% ('SUBJID' vs 'USUBJID'); Label match: +10%"
```

## GUI Integration Changes

### Remove (in `mapping.rs`):
```rust
// DELETE this function - use centralized scoring
fn calculate_name_similarity(source: &str, target: &str) -> f32 { ... }
```

### Replace dropdown scoring:
```rust
// Before:
let similarity = calculate_name_similarity(col, &var_name);

// After:
let score = ms.scorer().score(col, &var_name)
    .map(|s| s.score)
    .unwrap_or(0.0);
```

## Implementation Phases

### Phase 1: Delete old code
- [ ] Delete `repository.rs`
- [ ] Delete `patterns.rs`
- [ ] Delete `utils.rs`
- [ ] Clear `lib.rs`

### Phase 2: Create new implementation
- [ ] Create `error.rs` with MappingError enum
- [ ] Create `score.rs` with ScoringEngine
- [ ] Create `state.rs` with MappingState
- [ ] Update `lib.rs` with exports

### Phase 3: Update GUI
- [ ] Update `services/mapping.rs` to use new API
- [ ] Update `views/domain_editor/mapping.rs`:
  - Remove `calculate_name_similarity()`
  - Use `scorer().score()` for dropdown sorting

### Phase 4: Update other consumers
- [ ] Verify `sdtm-output` works with new `MappingConfig`
- [ ] Update `sdtm-ingest` if needed (ColumnHint export)

### Phase 5: Testing
- [ ] Unit tests for ScoringEngine
- [ ] Integration tests for MappingState
- [ ] Verify GUI still works

## Critical Files

**To delete:**
- `crates/sdtm-map/src/repository.rs`
- `crates/sdtm-map/src/patterns.rs`
- `crates/sdtm-map/src/utils.rs`

**To rewrite:**
- `crates/sdtm-map/src/lib.rs`
- `crates/sdtm-map/src/state.rs`
- `crates/sdtm-map/src/engine.rs` → `score.rs`
- `crates/sdtm-map/src/types.rs` → merge into other files

**To update:**
- `crates/sdtm-gui/src/services/mapping.rs`
- `crates/sdtm-gui/src/views/domain_editor/mapping.rs`

## Breaking Changes

| Old | New |
|-----|-----|
| `MappingSuggestion` | `Suggestion` (simpler) |
| `VariableMappingStatus` | `VariableStatus` |
| `MappingRepository` | Deleted |
| `MappingEngine` | `ScoringEngine` |
| `from_domain()` | `new()` |

## Dependencies

```toml
[dependencies]
sdtm-model = { path = "../sdtm-model" }
rapidfuzz = "0.6"  # For Jaro-Winkler
serde = { version = "1", features = ["derive"] }
```
