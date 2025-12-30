# SDTM Transformation System - Implementation Plan

## Overview

This plan implements a **data-driven, variable-level transformation system** for
SDTM domains. Instead of hardcoding rules per domain, we **derive transformation
types from variable metadata** already in `Variables.csv` and our `Variable`
struct.

### Key Principle: No Hardcoded Domain Rules

Transformation rules are **inferred** from existing variable metadata:

| Metadata Field           | Transformation Inference                                                        |
| ------------------------ | ------------------------------------------------------------------------------- |
| Variable name pattern    | `STUDYID` → Constant, `--SEQ` → Sequence, `--DTC` → DateTime, `--DY` → StudyDay |
| `codelist_code`          | Present → CT Normalization with that codelist                                   |
| `described_value_domain` | "ISO 8601 datetime" → DateTime, "ISO 8601 duration" → Duration                  |
| `role`                   | "Identifier" + pattern → special handling                                       |
| `data_type`              | `Num` + result variable → NumericConversion                                     |

---

## Phase 1: Enhance Variable Metadata

### 1.1 Add `described_value_domain` to Variable struct

Currently missing from our `Variable` struct. Need to parse from CSV:

- `"ISO 8601 datetime or interval"` →
  `Some(DescribedValueDomain::Iso8601DateTime)`
- `"ISO 8601 duration"` → `Some(DescribedValueDomain::Iso8601Duration)`
- Empty → `None`

```rust
// In sdtm-model/src/domain.rs

/// Described value domain for variables (from SDTMIG Variables.csv)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DescribedValueDomain {
    Iso8601DateTime,    // "ISO 8601 datetime or interval"
    Iso8601Date,        // "ISO 8601 date"
    Iso8601Duration,    // "ISO 8601 duration"
    MedDRA,             // External dictionary
    WHODrug,            // External dictionary
    Other(String),      // Catch-all
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub label: Option<String>,
    pub data_type: VariableType,
    pub length: Option<u32>,
    pub role: Option<String>,
    pub core: Option<String>,
    pub codelist_code: Option<String>,
    pub order: Option<u32>,
    // NEW: Add this field
    pub described_value_domain: Option<String>,
}
```

### 1.2 Update Standards Loader

In `sdtm-standards/src/loaders.rs`, parse the `Described Value Domain(s)`
column:

```rust
let variable = Variable {
    // ... existing fields ...
    described_value_domain: row
        .get("Described Value Domain(s)")
        .filter(|v| !v.is_empty())
        .cloned(),
};
```

---

## Phase 2: Data-Driven Transform Derivation

### 2.1 TransformType Enum (Simplified)

```rust
/// Transformation type - derived from variable metadata, NOT hardcoded per domain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformType {
    // Direct value operations
    CopyDirect,
    Constant { value: String },
    
    // Identifier patterns (inferred from name)
    UsubjidPrefix,      // USUBJID: prepend STUDYID
    SequenceNumber,     // --SEQ: auto-generate
    
    // Controlled terminology (inferred from codelist_code)
    CtNormalization { codelist_code: String },
    
    // Date/time (inferred from described_value_domain)
    Iso8601DateTime,
    Iso8601Date,
    Iso8601Duration,
    StudyDay { reference: String },  // --DY: calculate from --DTC
    
    // Numeric (inferred from data_type + role)
    NumericConversion,
}
```

### 2.2 Core Derivation Logic

**The magic function** - derives transform type from variable metadata:

```rust
impl Variable {
    /// Derive the transformation type from variable metadata.
    /// This is the core logic - NO hardcoded domain rules needed.
    /// 
    /// Derivation is based ONLY on:
    /// - Variable name patterns (STUDYID, DOMAIN, *SEQ, *DY, *DTC, *DUR)
    /// - `described_value_domain` (ISO 8601 formats)
    /// - `codelist_code` (CT normalization)
    /// - `data_type` + `role` combination
    pub fn infer_transform_type(&self) -> TransformType {
        let name_upper = self.name.to_uppercase();
        
        // 1. Check for universal identifier patterns (same across ALL domains)
        match name_upper.as_str() {
            "STUDYID" => return TransformType::Constant { value: String::new() },
            "DOMAIN" => return TransformType::Constant { value: String::new() },  // Set at runtime
            "USUBJID" => return TransformType::UsubjidPrefix,
            _ => {}
        }
        
        // 2. Check variable name suffix patterns (work for ANY domain)
        if name_upper.ends_with("SEQ") && self.role_is("Identifier") {
            return TransformType::SequenceNumber;
        }
        if name_upper.ends_with("DY") && self.role_is("Timing") {
            // --DY variables: calculate from corresponding --DTC (e.g., AEDY → AEDTC)
            let dtc_var = name_upper.replace("DY", "DTC");
            return TransformType::StudyDay { reference: dtc_var };
        }
        
        // 3. Check described_value_domain for ISO 8601 formats
        if let Some(ref domain) = self.described_value_domain {
            let domain_lower = domain.to_lowercase();
            if domain_lower.contains("iso 8601") {
                if domain_lower.contains("duration") {
                    return TransformType::Iso8601Duration;
                } else {
                    return TransformType::Iso8601DateTime;
                }
            }
        }
        
        // 4. Check for codelist - CT normalization
        if let Some(ref code) = self.codelist_code {
            return TransformType::CtNormalization { 
                codelist_code: code.clone() 
            };
        }
        
        // 5. Check for numeric result variables
        if self.data_type == VariableType::Num && self.is_result_qualifier() {
            return TransformType::NumericConversion;
        }
        
        // 6. Default: direct copy
        TransformType::CopyDirect
    }
    
    fn role_is(&self, expected: &str) -> bool {
        self.role.as_ref().map(|r| r.eq_ignore_ascii_case(expected)).unwrap_or(false)
    }
    
    fn is_result_qualifier(&self) -> bool {
        self.role.as_ref()
            .map(|r| r.to_lowercase().contains("result"))
            .unwrap_or(false)
    }
}
```

### 2.3 Variable Name Pattern Rules

These patterns work across ALL domains automatically:

| Pattern   | Condition                                     | Transform                |
| --------- | --------------------------------------------- | ------------------------ |
| `STUDYID` | Exact match                                   | `Constant`               |
| `DOMAIN`  | Exact match                                   | `Constant` (domain code) |
| `USUBJID` | Exact match                                   | `UsubjidPrefix`          |
| `*SEQ`    | Ends with SEQ + Identifier role               | `SequenceNumber`         |
| `*DY`     | Ends with DY + Timing role                    | `StudyDay`               |
| `*DTC`    | Ends with DTC + Timing role + ISO 8601 domain | `Iso8601DateTime`        |
| `*DUR`    | Ends with DUR + ISO 8601 duration domain      | `Iso8601Duration`        |
| Any       | Has `codelist_code`                           | `CtNormalization`        |
| Any       | Num type + Result role                        | `NumericConversion`      |

---

## Phase 3: Domain Pipeline (Data-Driven)

### 3.1 TransformRule Struct

```rust
/// A single transformation rule for a variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRule {
    /// Target SDTM variable name
    pub target_variable: String,
    /// Source column from mapping (if mapped)
    pub source_column: Option<String>,
    /// Transformation type (derived from variable metadata)
    pub transform_type: TransformType,
    /// Whether auto-derived or user-customized
    pub origin: TransformOrigin,
    /// Execution order (based on variable order)
    pub order: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformOrigin {
    /// Auto-derived from variable metadata
    Derived,
    /// User has customized this rule
    UserDefined,
}
```

### 3.2 DomainPipeline - Auto-Generated from Domain

```rust
/// Transformation pipeline for a domain - auto-generated from Variable metadata
pub struct DomainPipeline {
    pub domain_code: String,
    pub rules: Vec<TransformRule>,
}

impl DomainPipeline {
    /// Build pipeline automatically from domain metadata.
    /// NO hardcoded rules - everything derived from Variable struct.
    pub fn from_domain(domain: &Domain) -> Self {
        let rules: Vec<TransformRule> = domain.variables
            .iter()
            .enumerate()
            .map(|(idx, var)| TransformRule {
                target_variable: var.name.clone(),
                source_column: None, // Set from mapping later
                transform_type: var.infer_transform_type(),  // No domain_code needed!
                origin: TransformOrigin::Derived,
                order: var.order.unwrap_or(idx as u32),
            })
            .collect();
        
        Self {
            domain_code: domain.code.clone(),
            rules,
        }
    }
    
    /// Apply mapping info to set source columns
    pub fn with_mapping(mut self, mapping: &MappingConfig) -> Self {
        for rule in &mut self.rules {
            if let Some(source) = mapping.get_source_for(&rule.target_variable) {
                rule.source_column = Some(source.to_string());
            }
        }
        self
    }
}
```

---

## Phase 4: Removed - No Hardcoded Domain Rules

~~Previously this section contained hardcoded rules for DM, AE, CM, LB, VS~~

**DELETED** - All rules are now derived from variable metadata automatically.

---

## Phase 5: GUI Integration

### 5.1 Transform Tab - Shows Derived Rules

The Transform tab will display the **auto-derived rules** from variable
metadata:

```rust
// In sdtm-gui/src/views/domain_editor/transform.rs

fn show_transform_panel(ui: &mut Ui, domain: &Domain, pipeline: &DomainPipeline, theme: &Theme) {
    for rule in &pipeline.rules {
        ui.horizontal(|ui| {
            // Status icon using egui_phosphor
            let icon = match &rule.transform_type {
                TransformType::CopyDirect => egui_phosphor::regular::COPY,
                TransformType::CtNormalization { .. } => egui_phosphor::regular::BOOK_OPEN,
                TransformType::Iso8601DateTime | TransformType::Iso8601Date => egui_phosphor::regular::CALENDAR,
                TransformType::Iso8601Duration => egui_phosphor::regular::TIMER,
                TransformType::SequenceNumber => egui_phosphor::regular::HASH,
                TransformType::StudyDay { .. } => egui_phosphor::regular::CALENDAR_CHECK,
                TransformType::Constant { .. } => egui_phosphor::regular::LOCK,
                TransformType::UsubjidPrefix => egui_phosphor::regular::IDENTIFICATION_CARD,
                TransformType::NumericConversion => egui_phosphor::regular::FUNCTION,
            };
            
            ui.label(RichText::new(icon).color(theme.accent));
            ui.label(&rule.target_variable);
            ui.label(egui_phosphor::regular::ARROW_LEFT);
            
            if let Some(ref source) = rule.source_column {
                ui.label(source);
            } else {
                ui.label(RichText::new(egui_phosphor::regular::LINK_BREAK).color(theme.text_muted));
                ui.colored_label(theme.text_muted, "(not mapped)");
            }
            
            // Show transform type on the right
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(rule.transform_type.display_name()).small());
            });
        });
    }
}
```

### 5.2 Transform Preview

Show before/after values for the selected variable:

```rust
pub struct TransformPreview {
    pub variable_name: String,
    pub source_values: Vec<String>,
    pub transformed_values: Vec<String>,
    pub warnings: Vec<TransformWarning>,
}

fn show_preview(ui: &mut Ui, preview: &TransformPreview) {
    ui.heading(&preview.variable_name);
    
    egui::Grid::new("preview_grid").show(ui, |ui| {
        ui.label("Source");
        ui.label(egui_phosphor::regular::ARROW_RIGHT);
        ui.label("Transformed");
        ui.end_row();
        
        for (src, dst) in preview.source_values.iter().zip(&preview.transformed_values) {
            ui.label(src);
            ui.label(egui_phosphor::regular::ARROW_RIGHT);
            if src != dst {
                ui.colored_label(theme.success, dst);
            } else {
                ui.label(dst);
            }
            ui.end_row();
        }
    });
    
    // Show warnings (CT mismatches, parse failures, etc.)
    for warning in &preview.warnings {
        ui.horizontal(|ui| {
            ui.label(RichText::new(egui_phosphor::regular::WARNING).color(theme.warning));
            ui.label(&warning.message);
        });
    }
}
```

### 5.3 CT Mapping Dialog (For Mismatches)

When CT normalization fails to match:

```rust
fn show_ct_mapping_dialog(
    ui: &mut Ui,
    unmatched_value: &str,
    codelist: &Codelist,
    custom_maps: &mut HashMap<String, String>,
) {
    ui.heading(format!(
        "{} Map \"{}\" to CT term",
        egui_phosphor::regular::LINK,
        unmatched_value
    ));
    
    // Search box with icon
    ui.horizontal(|ui| {
        ui.label(egui_phosphor::regular::MAGNIFYING_GLASS);
        ui.text_edit_singleline(&mut search);
    });
    
    // Show matching CT terms
    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        for term in codelist.terms.iter().filter(|t| t.matches_search(&search)) {
            if ui.button(format!(
                "{} {}",
                egui_phosphor::regular::CHECK,
                &term.submission_value
            )).clicked() {
                custom_maps.insert(unmatched_value.to_string(), term.submission_value.clone());
            }
        }
    });
}
```

---

## Phase 6: Execution Engine

### 6.1 Transform Context

```rust
pub struct TransformContext<'a> {
    pub study_id: &'a str,
    pub ct_registry: &'a TerminologyRegistry,
    pub reference_start_dates: Option<&'a DataFrame>,  // RFSTDTC for --DY calculation
    pub custom_ct_maps: &'a HashMap<String, HashMap<String, String>>,
}
```

### 6.2 Execute Pipeline

```rust
impl DomainPipeline {
    pub fn execute(&self, df: &mut DataFrame, ctx: &TransformContext) -> TransformResult {
        let mut result = TransformResult::default();
        
        for rule in &self.rules {
            match self.execute_rule(df, rule, ctx) {
                Ok(stats) => result.merge(stats),
                Err(e) => result.errors.push(e),
            }
        }
        
        result
    }
    
    fn execute_rule(
        &self,
        df: &mut DataFrame,
        rule: &TransformRule,
        ctx: &TransformContext,
    ) -> Result<RuleStats> {
        match &rule.transform_type {
            TransformType::Constant { .. } => {
                // Set constant - value determined at runtime from pipeline context
                // STUDYID → ctx.study_id, DOMAIN → self.domain_code
                let value = match rule.target_variable.as_str() {
                    "STUDYID" => ctx.study_id.to_string(),
                    "DOMAIN" => self.domain_code.clone(),
                    _ => String::new(),
                };
                set_constant_column(df, &rule.target_variable, &value)
            }
            TransformType::UsubjidPrefix => {
                // Prepend STUDYID to SUBJID values
                apply_usubjid_prefix(df, ctx.study_id)
            }
            TransformType::SequenceNumber => {
                // Generate sequential numbers per subject
                assign_sequence_numbers(df, &rule.target_variable)
            }
            TransformType::CtNormalization { codelist_code } => {
                // Normalize to CT terms
                let Some(source) = &rule.source_column else {
                    return Ok(RuleStats::skipped("no source column"));
                };
                normalize_ct_column(df, source, &rule.target_variable, codelist_code, ctx)
            }
            TransformType::Iso8601DateTime => {
                // Parse and format as ISO 8601
                let Some(source) = &rule.source_column else {
                    return Ok(RuleStats::skipped("no source column"));
                };
                parse_datetime_column(df, source, &rule.target_variable)
            }
            TransformType::StudyDay { reference } => {
                // Calculate study day from DTC and RFSTDTC
                calculate_study_day(df, reference, &rule.target_variable, ctx)
            }
            TransformType::CopyDirect => {
                // Direct copy from source
                let Some(source) = &rule.source_column else {
                    return Ok(RuleStats::skipped("no source column"));
                };
                copy_column(df, source, &rule.target_variable)
            }
            _ => Ok(RuleStats::skipped("not implemented"))
        }
    }
}
```

---

## Implementation Order (Revised)

### Step 1: Enhance Variable Struct (1 day)

1. Add `described_value_domain: Option<String>` to `Variable` in sdtm-model
2. Update sdtm-standards loader to parse `Described Value Domain(s)` column
3. Run tests

### Step 2: Add Transform Derivation (1-2 days)

1. Create `sdtm-transform/src/rules/mod.rs` with `TransformType` enum
2. Add `Variable::infer_transform_type()` method to sdtm-model (no domain
   parameter!)
3. Add `DomainPipeline::from_domain()` function
4. Add tests verifying correct derivation from metadata alone

### Step 3: Implement Transform Executors (2-3 days)

1. Implement `Constant` executor
2. Implement `UsubjidPrefix` executor
3. Implement `SequenceNumber` executor
4. Implement `CtNormalization` executor (leveraging existing ct.rs)
5. Implement `Iso8601DateTime` executor (leveraging existing datetime.rs)
6. Implement `StudyDay` executor
7. Implement `CopyDirect` executor

### Step 4: GUI Integration (2 days)

1. Update transform.rs to show derived rules
2. Add transform preview component
3. Add CT mapping dialog
4. Add "Apply Transforms" functionality

### Step 5: Testing & Polish (1 day)

1. Integration tests with mock data
2. Verify all domains derive correct transforms
3. Performance testing

---

## File Changes Summary (Revised)

### Modified Files

```
crates/sdtm-model/src/domain.rs
  - Add `described_value_domain: Option<String>` to Variable
  - Add `Variable::infer_transform_type()` method

crates/sdtm-standards/src/loaders.rs
  - Parse `Described Value Domain(s)` column

crates/sdtm-transform/src/lib.rs
  - Export new rules module

crates/sdtm-gui/src/views/domain_editor/transform.rs
  - Show derived rules with icons
  - Add preview panel
```

### New Files

```
crates/sdtm-transform/src/rules/
  mod.rs           # TransformType, TransformRule, DomainPipeline
  
crates/sdtm-gui/src/views/domain_editor/
  ct_mapping_dialog.rs  # CT term selection dialog
```

### Deleted

```
(none - we removed the hardcoded domain rules from the plan)
```

---

## Benefits of Data-Driven Approach

1. **Zero maintenance for new domains** - Any domain in Variables.csv
   automatically gets correct transforms
2. **Single source of truth** - Variable metadata in CSV drives everything
3. **Testable** - Can unit test `infer_transform_type()` with synthetic
   variables
4. **Extensible** - Add new TransformTypes as needed, derivation logic handles
   them
5. **Auditable** - Users can see exactly why each transform was derived

---

## Success Criteria

1. ✅ All domains auto-derive correct transformation rules from metadata
2. ✅ No hardcoded domain-specific rules anywhere
3. ✅ CT normalization uses `codelist_code` from Variable
4. ✅ DateTime transforms use `described_value_domain` to detect ISO 8601
5. ✅ GUI shows all derived rules with visual indicators
6. ✅ Users can preview transforms before applying
7. ✅ CT mismatches can be resolved via mapping dialog
