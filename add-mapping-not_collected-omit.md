# Plan: Add "Not Collected" and "Omit" Options to Mapping GUI

## Summary

Add functionality to mark unmapped variables as "Not Collected" (creates null column with Define-XML comment) or "Omit" (exclude from output) based on SDTM Core designation rules.

**Key SDTM Rules:**
- **Required**: Cannot be null - must map a source column (show error)
- **Expected**: Can be null with documentation - "Not Collected" option
- **Permissible**: Can be null or omitted entirely - "Not Collected" OR "Omit" options

## Data Model Changes

### 1. New Enum: `VariableAssignment` (`crates/sdtm-map/src/state.rs`)

```rust
/// How an unmapped variable should be handled
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableAssignment {
    /// Variable is mapped to a source column
    Mapped { column: String, confidence: f32 },
    /// Data was not collected - creates null column with Define-XML comment
    NotCollected { reason: String },
    /// Variable should be omitted from output (Permissible only)
    Omit,
}
```

### 2. Update `VariableStatus` Enum

```rust
pub enum VariableStatus {
    Accepted,       // Mapped to a column
    Suggested,      // Has a suggestion
    NotCollected,   // Explicitly marked as not collected
    Omitted,        // Marked to omit (Permissible only)
    Unmapped,       // No mapping, no assignment
}
```

### 3. Update `MappingState` (`crates/sdtm-map/src/state.rs`)

```rust
pub struct MappingState {
    domain: Domain,
    study_id: String,
    scorer: ScoringEngine,
    suggestions: BTreeMap<String, (String, f32)>,
    accepted: BTreeMap<String, (String, f32)>,        // Column mappings
    not_collected: BTreeMap<String, String>,          // NEW: var -> reason
    omitted: BTreeSet<String>,                        // NEW: vars to omit
    source_columns: Vec<String>,
    column_hints: BTreeMap<String, ColumnHint>,
}
```

### 4. New Methods on `MappingState`

```rust
impl MappingState {
    /// Mark variable as "not collected" with Define-XML reason
    /// Only allowed for Expected and Permissible variables
    pub fn mark_not_collected(&mut self, var: &str, reason: &str) -> Result<(), MappingError>;

    /// Mark variable to be omitted from output
    /// Only allowed for Permissible variables
    pub fn mark_omit(&mut self, var: &str) -> Result<(), MappingError>;

    /// Clear any assignment (mapping, not_collected, or omit)
    pub fn clear_assignment(&mut self, var: &str);

    /// Get the assignment for a variable
    pub fn get_assignment(&self, var: &str) -> Option<VariableAssignment>;

    /// Get all not_collected entries (for Define-XML export)
    pub fn all_not_collected(&self) -> &BTreeMap<String, String>;

    /// Get all omitted variables
    pub fn all_omitted(&self) -> &BTreeSet<String>;
}
```

### 5. New Error Variants (`crates/sdtm-map/src/error.rs`)

```rust
pub enum MappingError {
    // ... existing ...
    CannotSetNullOnRequired(String),      // Variable name
    CannotOmitNonPermissible(String),     // Variable name
}
```

## GUI Changes

### 1. Update Mapping Tab Detail Panel (`crates/sdtm-gui/src/views/domain_editor/mapping.rs`)

For unmapped variables, show options based on Core designation:

```
┌─────────────────────────────────────────────────────┐
│ RFSTDTC (Expected)                                  │
│ Reference Start Date/Time                           │
├─────────────────────────────────────────────────────┤
│ [Dropdown: Select source column...]                 │
│                                                     │
│ ─── OR ───                                          │
│                                                     │
│ [Mark as Not Collected]                             │
│                                                     │
│ Reason for Define-XML:                              │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Data not collected in this study                │ │
│ └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

For Permissible variables, also show:
```
│ [Mark as Not Collected]  [Omit from Output]         │
```

For Required variables, show error:
```
│ ⚠ Required variable - must map a source column     │
│ [Dropdown: Select source column...]                 │
```

### 2. Update Variable List Icons

```rust
fn get_status_icon(status: VariableStatus) -> (&'static str, Color32) {
    match status {
        VariableStatus::Accepted => (CHECK, green),
        VariableStatus::Suggested => (CIRCLE_DASHED, yellow),
        VariableStatus::NotCollected => (PROHIBIT, muted),      // NEW
        VariableStatus::Omitted => (MINUS_CIRCLE, muted),       // NEW
        VariableStatus::Unmapped => (MINUS, muted),
    }
}
```

### 3. Summary Bar Update

Show counts for each status:
- "5 Mapped" (green)
- "3 Suggested" (yellow)
- "2 Not Collected" (gray)
- "1 Omitted" (gray)
- "2 Unmapped" (red if Required, yellow otherwise)

## Transform Pipeline Changes

### 1. Update `build_preview_dataframe` (`crates/sdtm-transform/src/preview.rs`)

Handle three cases:
1. **Mapped**: Use source column (existing behavior)
2. **NotCollected**: Create null column
3. **Omitted**: Skip variable entirely

```rust
pub fn build_preview_dataframe(
    source_df: &DataFrame,
    mappings: &BTreeMap<String, String>,      // var -> column
    not_collected: &BTreeSet<String>,          // vars with null
    omitted: &BTreeSet<String>,                // vars to skip
    domain: &Domain,
    study_id: &str,
    ct_registry: Option<&TerminologyRegistry>,
) -> Result<DataFrame, TransformError>
```

### 2. Update `execute_pipeline` (`crates/sdtm-transform/src/executor.rs`)

Skip rules for omitted variables:
```rust
for rule in pipeline.rules_ordered() {
    if omitted.contains(&rule.target_variable) {
        continue;  // Skip omitted variables
    }
    // ... existing logic
}
```

## Validation Changes

### 1. Update `checks/required.rs`

Don't report RequiredMissing if there's an assignment (but there shouldn't be for Required):
```rust
// Required variables must be mapped - NotCollected/Omit not allowed
if variable.core == Some(CoreDesignation::Required) {
    if !has_mapping(var) {
        issues.push(Issue::RequiredMissing { ... });
    }
}
```

### 2. Update `checks/expected.rs`

Don't report ExpectedMissing if marked as NotCollected:
```rust
// Expected variables: OK if mapped OR marked as NotCollected
if variable.core == Some(CoreDesignation::Expected) {
    if !has_mapping(var) && !is_not_collected(var) {
        issues.push(Issue::ExpectedMissing { ... });
    }
}
```

## Implementation Phases

### Phase 1: Data Model (sdtm-map)
- [ ] Add `not_collected: BTreeMap<String, String>` to MappingState
- [ ] Add `omitted: BTreeSet<String>` to MappingState
- [ ] Add `mark_not_collected()` method with Core validation
- [ ] Add `mark_omit()` method with Permissible validation
- [ ] Add error variants to MappingError
- [ ] Update `variable_status()` to return new states
- [ ] Update serialization for MappingConfig export

### Phase 2: GUI (sdtm-gui)
- [ ] Update GUI MappingState wrapper to expose new methods
- [ ] Add "Mark as Not Collected" button in detail panel
- [ ] Add "Omit from Output" button for Permissible variables
- [ ] Add text input for Define-XML reason
- [ ] Update status icons for new states
- [ ] Update summary bar counts
- [ ] Show error message for Required unmapped variables

### Phase 3: Transform Pipeline (sdtm-transform)
- [ ] Update `build_preview_dataframe` signature
- [ ] Handle not_collected vars (create null column)
- [ ] Handle omitted vars (skip in output)
- [ ] Update validation tab to pass new parameters

### Phase 4: Validation (sdtm-validate)
- [ ] Update expected.rs to check not_collected state
- [ ] Ensure Required check still enforces mapping requirement

## Files to Modify

```
crates/sdtm-map/src/state.rs          # Add not_collected, omitted fields + methods
crates/sdtm-map/src/error.rs          # Add new error variants
crates/sdtm-map/src/lib.rs            # Re-export new types

crates/sdtm-gui/src/services/mapping.rs    # Expose new methods
crates/sdtm-gui/src/views/domain_editor/mapping.rs  # UI changes

crates/sdtm-transform/src/preview.rs       # Handle new states
crates/sdtm-transform/src/executor.rs      # Skip omitted vars

crates/sdtm-validate/src/checks/expected.rs  # Check not_collected
```

## Define-XML Integration (Future)

The `not_collected` map stores variable → reason pairs that will be used when generating Define-XML:

```xml
<ItemDef OID="IT.DM.RFSTDTC" Name="RFSTDTC" ...>
  <Description>
    <TranslatedText xml:lang="en">Reference Start Date/Time</TranslatedText>
  </Description>
  <!-- Comment from not_collected reason -->
  <def:CommentOID OID="COM.DM.RFSTDTC"/>
</ItemDef>

<def:CommentDef OID="COM.DM.RFSTDTC">
  <Description>
    <TranslatedText xml:lang="en">Data not collected in this study</TranslatedText>
  </Description>
</def:CommentDef>
```
