# CDISC Transpiler — GUI Architecture

## Executive Summary

The CDISC Transpiler GUI transforms clinical trial source data into
SDTM-compliant formats. This document defines the user experience, information
architecture, technical implementation, and necessary architectural refactoring
for a desktop application built with egui + eframe.

**Target Users**: Clinical data programmers, biostatisticians, and data managers
who understand SDTM but need an intuitive tool for data transformation.

**Core Task**: Map source CSV columns to SDTM variables, validate against
Controlled Terminology, and export submission-ready files.

**Critical Architectural Shift**: This GUI requires moving from a linear
pipeline architecture to a modular, state-driven architecture that supports
non-linear, interactive workflows.

---

## Table of Contents

1. [Architecture Transformation Overview](#architecture-transformation-overview)
2. [Detailed Refactoring Plan](#detailed-refactoring-plan)
3. [Understanding the Domain](#understanding-the-domain)
4. [User Goals & Workflow](#user-goals--workflow)
5. [Information Architecture](#information-architecture)
6. [Detailed Screen Specifications](#detailed-screen-specifications)
7. [Technical Implementation](#technical-implementation)
8. [Migration Strategy](#migration-strategy)

---

## Architecture Transformation Overview

### Current State: Linear Pipeline Architecture

The current codebase is designed as a **linear, one-shot pipeline**:

```
Ingest → Map → Preprocess → Domain Rules → Validate → Output
```

**Problems for GUI:**

1. **Tight coupling**: Each stage expects previous stages to be complete
2. **All-or-nothing**: You can't map one domain without processing everything
3. **No intermediate state**: Pipeline runs to completion or fails
4. **Hard to inspect**: Can't pause and examine results mid-process
5. **Difficult rollback**: Can't undo individual mappings without restarting

### Target State: Modular, State-Driven Architecture

The GUI requires **independent, reusable components** with **persistent state**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Study Session State                           │
│  ┌───────────┐  ┌──────────┐  ┌────────────┐  ┌──────────┐            │
│  │  Domains  │  │ Mappings │  │ Validation │  │  Output  │            │
│  │   State   │  │  State   │  │   State    │  │  State   │            │
│  └───────────┘  └──────────┘  └────────────┘  └──────────┘            │
└─────────────────────────────────────────────────────────────────────────┘
         ↕              ↕              ↕              ↕
┌─────────────────────────────────────────────────────────────────────────┐
│                         Independent Services                             │
│  ┌───────────┐  ┌──────────┐  ┌────────────┐  ┌──────────┐            │
│  │  Domain   │  │ Mapping  │  │ Validation │  │  Export  │            │
│  │ Discovery │  │  Engine  │  │  Service   │  │ Service  │            │
│  └───────────┘  └──────────┘  └────────────┘  └──────────┘            │
└─────────────────────────────────────────────────────────────────────────┘
```

**Benefits:**

- Map domains in any order
- Save and resume work
- Validate individual domains on demand
- Preview transformations before applying
- Undo/redo individual changes
- Export subsets of domains

---

## Detailed Refactoring Plan

### 1. Create New Crate: `sdtm-session`

**Purpose**: Manage GUI session state and persistence.

**Responsibilities:**

- Study session creation and loading
- Domain-level state management
- Mapping configuration storage
- Undo/redo stack
- Session serialization/deserialization

**Key Types:**

```rust
pub struct StudySession {
    pub study_id: String,
    pub study_folder: PathBuf,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub domains: HashMap<String, DomainState>,
    pub global_settings: GlobalSettings,
}

pub struct DomainState {
    pub code: String,
    pub source_file: PathBuf,
    pub status: DomainStatus,
    pub mapping: Option<MappingConfig>,
    pub validation: Option<ValidationReport>,
    pub preview_data: Option<DataFrame>,
    pub suppqual_mappings: Vec<SuppqualMapping>,
}

pub enum DomainStatus {
    NotStarted,
    MappingInProgress,
    MappingComplete,
    ValidationFailed,
    ReadyForExport,
}
```

**Dependencies:** `sdtm-model` only (minimal coupling)

---

### 2. Refactor `sdtm-core` → Extract Pipeline Logic

**Current Problem:** `sdtm-core` contains `PipelineContext` and `process_domain`
which assume a linear pipeline flow.

**Solution:** Split into two layers:

#### Layer 1: Pure Domain Processing (keep in `sdtm-core`)

```rust
// Stateless, reusable functions
pub fn apply_usubjid_prefix(
    df: &mut DataFrame, 
    study_id: &str, 
    mode: UsubjidPrefixMode
) -> Result<()>;

pub fn assign_sequence_numbers(
    df: &mut DataFrame, 
    domain_code: &str
) -> Result<()>;

pub fn normalize_ct_column(
    df: &mut DataFrame, 
    variable: &Variable, 
    codelist: &Codelist
) -> Result<()>;

pub fn apply_domain_rules(
    df: &mut DataFrame, 
    domain: &Domain, 
    rules: &DomainRules
) -> Result<()>;
```

#### Layer 2: Pipeline Orchestration (new `sdtm-pipeline` crate or keep in CLI)

```rust
// Orchestration logic for batch processing
pub struct PipelineRunner { /* ... current implementation ... */ }
```

**Benefits:**

- GUI can call individual processing functions on-demand
- No forced ordering between operations
- Each function is testable in isolation

---

### 3. Decouple `sdtm-map` from Automatic Execution

**Current Problem:** Mapping engine automatically applies mappings during
`build_mapped_domain_frame`.

**Solution:** Separate suggestion from application:

```rust
// Phase 1: Generate suggestions (no side effects)
pub fn suggest_mappings(
    source_schema: &CsvSchema,
    domain: &Domain,
    hints: &[ColumnHint],
) -> Vec<MappingSuggestion> { /* ... */ }

// Phase 2: Apply selected mappings (returns new DataFrame)
pub fn apply_mapping(
    source_df: &DataFrame,
    mapping: &MappingConfig,
    domain: &Domain,
) -> Result<DataFrame> { /* ... */ }

// Phase 3: Preview mapping (shows sample rows without full processing)
pub fn preview_mapping(
    source_df: &DataFrame,
    variable: &Variable,
    source_column: &str,
    row_limit: usize,
) -> Result<Vec<(String, String)>> { /* ... */ }
```

**Benefits:**

- User can review suggestions before accepting
- Easy to show "before/after" previews
- Can modify suggestions interactively

---

### 4. Make Validation Independent

**Current Problem:** Validation is tightly coupled to full pipeline execution.

**Solution:** Expose incremental validation:

```rust
// Validate a single domain independently
pub fn validate_domain(
    df: &DataFrame,
    domain: &Domain,
    ct_registry: &TerminologyRegistry,
) -> ValidationReport { /* ... */ }

// Validate a specific variable's values
pub fn validate_variable(
    values: &[String],
    variable: &Variable,
    codelist: Option<&Codelist>,
) -> Vec<ValidationIssue> { /* ... */ }

// Preview CT mapping for user confirmation
pub fn preview_ct_normalization(
    source_values: &[String],
    codelist: &Codelist,
    matching_mode: CtMatchingMode,
) -> Vec<(String, Option<String>, f64)> { // (source, mapped, confidence)
    /* ... */
}
```

**Benefits:**

- Validate as the user works, not just at the end
- Show real-time feedback on CT compliance
- Allow user to resolve issues incrementally

---

### 5. Refactor SUPPQUAL Generation

**Current Problem:** SUPPQUAL is built automatically during pipeline execution,
makes assumptions about what should be excluded.

**Solution:** Make it user-controllable:

```rust
// Identify candidates for SUPPQUAL
pub fn identify_suppqual_candidates(
    source_schema: &CsvSchema,
    mapped_columns: &[String],
    standard_variables: &HashSet<String>,
) -> Vec<SuppqualCandidate> { /* ... */ }

pub struct SuppqualCandidate {
    pub source_column: String,
    pub suggested_qnam: String,
    pub suggested_qlabel: String,
    pub sample_values: Vec<String>,
    pub recommendation: SuppqualRecommendation,
}

pub enum SuppqualRecommendation {
    Include,     // Non-standard, should be in SUPP
    Exclude,     // Common across files, probably study-level
    UserDecide,  // Ambiguous, let user choose
}

// Generate SUPPQUAL only for user-approved columns
pub fn build_suppqual_from_selections(
    source_df: &DataFrame,
    parent_domain: &Domain,
    selections: &[SuppqualSelection],
) -> Result<DataFrame> { /* ... */ }
```

**Benefits:**

- User controls what goes into SUPPQUAL
- Clear visibility into excluded columns
- Can adjust decisions without re-running everything

---

### 6. Simplify Transform Operations

**Current Problem:** Date transforms, CT normalization, etc. are scattered
across the pipeline.

**Solution:** Centralize as configurable transforms:

```rust
pub enum TransformRule {
    DateFormat { from_pattern: String, to_pattern: String },
    CtNormalization { variable: String, matching_mode: CtMatchingMode },
    Uppercase { variable: String },
    Concatenate { target: String, sources: Vec<String>, separator: String },
    Constant { target: String, value: String },
}

pub struct DomainTransforms {
    pub domain_code: String,
    pub rules: Vec<TransformRule>,
}

// Apply transforms to a DataFrame
pub fn apply_transforms(
    df: &mut DataFrame,
    transforms: &DomainTransforms,
) -> Result<TransformReport> { /* ... */ }

// Preview a single transform
pub fn preview_transform(
    df: &DataFrame,
    rule: &TransformRule,
    sample_size: usize,
) -> Vec<(String, String)> { /* ... */ }
```

**Benefits:**

- User can see and modify all transforms
- Easy to add/remove/reorder transforms
- Preview before applying

---

### 7. Decouple Output Generation

**Current Problem:** Output writing is all-or-nothing, tied to pipeline
completion.

**Solution:** Allow selective export:

```rust
// Export individual domains
pub fn export_domain(
    df: &DataFrame,
    domain: &Domain,
    output_dir: &Path,
    formats: &[OutputFormat],
) -> Result<OutputPaths> { /* ... */ }

// Export a subset of domains
pub fn export_domains_selective(
    frames: &[(String, DataFrame, Domain)],
    output_dir: &Path,
    formats: &[OutputFormat],
    define_xml_options: Option<DefineXmlOptions>,
) -> Result<ExportReport> { /* ... */ }

// Preview export (dry run)
pub fn preview_export(
    df: &DataFrame,
    domain: &Domain,
    format: OutputFormat,
) -> Result<ExportPreview> { /* ... */ }
```

**Benefits:**

- Export individual domains as they're completed
- Re-export after changes without full rebuild
- Show file sizes and record counts before writing

---

## Proposed Crate Structure for GUI-Only

**CRITICAL DECISION: Removing CLI Entirely**

Since we're building GUI-only, we can eliminate:

- ❌ `sdtm-cli` crate (delete entirely)
- ❌ `PipelineRunner` orchestration (CLI-specific)
- ❌ Linear pipeline constraints
- ❌ CLI argument parsing
- ❌ Batch processing logic

**Simplified Crate Structure:**

```
sdtm-transpiler/
├── crates/
│   ├── sdtm-model/          # ✓ Core types - NO changes
│   ├── sdtm-standards/      # ✓ Standards loading - NO changes
│   ├── sdtm-xpt/            # ✓ XPT format - NO changes
│   │
│   ├── sdtm-ingest/         # ⚠️ MINOR: Add preview methods
│   ├── sdtm-validate/       # ⚠️ MINOR: Already supports incremental
│   ├── sdtm-report/         # ⚠️ MINOR: Add selective export
│   │
│   ├── sdtm-map/            # ⚠️ REFACTOR: Split suggest/apply
│   ├── sdtm-transform/      # ⚠️ REFACTOR: Make operations standalone
│   │
│   ├── sdtm-core/           # ⚠️ MAJOR: Extract all business logic
│   │                        #          Remove PipelineContext/PipelineRunner
│   │
│   ├── sdtm-session/        # 🆕 NEW: Session state + services
│   └── sdtm-gui/            # 🆕 NEW: egui application
```

**What This Simplifies:**

| Component         | CLI Approach              | GUI-Only Approach             |
| ----------------- | ------------------------- | ----------------------------- |
| **Orchestration** | `PipelineRunner` + stages | Direct service calls from GUI |
| **State**         | Transient, CLI args       | Persistent `StudySession`     |
| **Processing**    | All-or-nothing batch      | Per-domain, on-demand         |
| **Validation**    | End-of-pipeline           | Real-time, incremental        |
| **Export**        | All domains together      | Selective, per-domain         |

---

## Refactored Architecture (GUI-Only)

### Core Philosophy

**Before (CLI Pipeline):**

```
User → CLI Args → PipelineRunner → Stage 1 → Stage 2 → ... → Output
                       ↓
              Tightly coupled stages
```

**After (GUI Services):**

```
User → GUI → Services → Independent Operations
        ↓         ↓            ↓
      State  No coupling   Composable
```

---

### Layer 1: Core Business Logic (No Orchestration)

**Location:** Refactored from `sdtm-core`, `sdtm-transform`, `sdtm-map`

These become **pure, stateless functions** with NO dependencies on pipeline
context:

```rust
// ===== sdtm-map/src/lib.rs =====

pub fn suggest_column_mappings(
    source_schema: &CsvSchema,
    domain: &Domain,
    hints: &[ColumnHint],
) -> Vec<MappingSuggestion> {
    // Pure function - no side effects
}

pub fn apply_column_mapping(
    source_df: &DataFrame,
    variable: &Variable,
    source_column: &str,
) -> Result<Series> {
    // Returns transformed series, doesn't modify DataFrame
}

// ===== sdtm-transform/src/processing.rs =====

pub fn apply_usubjid_prefix(
    usubjid_values: &[String],
    study_id: &str,
) -> Vec<String> {
    // Simple transformation, no context needed
}

pub fn assign_sequence_numbers(
    df: &DataFrame,
    subject_column: &str,
) -> Vec<i64> {
    // Returns sequence numbers, doesn't modify DataFrame
}

pub fn normalize_to_ct(
    values: &[String],
    codelist: &Codelist,
    mode: CtMatchingMode,
) -> Vec<String> {
    // Pure CT normalization
}

// ===== sdtm-transform/src/suppqual.rs =====

pub fn identify_unmapped_columns(
    source_schema: &CsvSchema,
    mapped_variables: &[String],
) -> Vec<String> {
    // Returns list of unmapped columns
}

pub fn build_suppqual_dataframe(
    source_df: &DataFrame,
    selections: &[SuppqualMapping],
    parent_domain: &str,
) -> Result<DataFrame> {
    // Creates SUPP DataFrame from selections
}
```

**Key Insight:** These functions don't know about "pipelines" - they're just
domain operations.

---

### Layer 2: Service Layer (Stateful Coordination)

**Location:** New `sdtm-session` crate

Services manage state and coordinate business logic:

```rust
// ===== sdtm-session/src/services/mapping.rs =====

pub struct MappingService {
    standards: Vec<Domain>,
    repositories: MappingRepository,
}

impl MappingService {
    pub fn suggest_for_domain(
        &self,
        domain_code: &str,
        source_schema: &CsvSchema,
    ) -> Result<Vec<MappingSuggestion>> {
        let domain = self.get_domain(domain_code)?;
        let hints = build_column_hints(source_schema);
        Ok(suggest_column_mappings(source_schema, domain, &hints))
    }
    
    pub fn apply_mapping(
        &self,
        source_df: &DataFrame,
        config: &MappingConfig,
        domain_code: &str,
    ) -> Result<DataFrame> {
        let domain = self.get_domain(domain_code)?;
        let mut result = source_df.clone();
        
        for (var_name, mapping) in &config.mappings {
            let variable = domain.get_variable(var_name)?;
            let series = apply_column_mapping(source_df, variable, &mapping.source)?;
            result.with_column(series)?;
        }
        
        Ok(result)
    }
}

// ===== sdtm-session/src/services/processing.rs =====

pub struct ProcessingService {
    ct_registry: TerminologyRegistry,
}

impl ProcessingService {
    pub fn process_domain(
        &self,
        df: &DataFrame,
        domain_code: &str,
        study_id: &str,
        options: &ProcessingOptions,
    ) -> Result<DataFrame> {
        let mut result = df.clone();
        
        // Apply transforms independently
        if options.apply_usubjid_prefix {
            result = self.apply_usubjid_transform(&result, study_id)?;
        }
        
        if options.apply_sequence_numbers {
            result = self.apply_sequence_transform(&result, "USUBJID")?;
        }
        
        if options.normalize_ct {
            result = self.apply_ct_normalization(&result, domain_code)?;
        }
        
        Ok(result)
    }
    
    fn apply_usubjid_transform(&self, df: &DataFrame, study_id: &str) -> Result<DataFrame> {
        let usubjid_col = df.column("USUBJID")?;
        let values: Vec<String> = /* extract values */;
        let prefixed = apply_usubjid_prefix(&values, study_id);
        df.with_column(Series::new("USUBJID", prefixed))
    }
}

// ===== sdtm-session/src/services/validation.rs =====

pub struct ValidationService {
    ct_registry: TerminologyRegistry,
}

impl ValidationService {
    pub fn validate_domain(
        &self,
        df: &DataFrame,
        domain_code: &str,
    ) -> ValidationReport {
        // Already exists in sdtm-validate - just wrap it
        validate_domain(domain, df, Some(&self.ct_registry))
    }
    
    pub fn validate_variable_values(
        &self,
        values: &[String],
        variable: &Variable,
    ) -> Vec<ValidationIssue> {
        // New incremental validation
    }
}

// ===== sdtm-session/src/session.rs =====

pub struct StudySession {
    pub study_id: String,
    pub study_folder: PathBuf,
    pub domains: HashMap<String, DomainState>,
    
    // Services
    pub mapping_service: MappingService,
    pub processing_service: ProcessingService,
    pub validation_service: ValidationService,
    pub export_service: ExportService,
}

pub struct DomainState {
    pub source_file: PathBuf,
    pub source_data: DataFrame,      // Original CSV data
    pub mapped_data: Option<DataFrame>,  // After mapping
    pub processed_data: Option<DataFrame>, // After processing
    pub mapping_config: Option<MappingConfig>,
    pub validation_report: Option<ValidationReport>,
    pub suppqual_selections: Vec<SuppqualMapping>,
    pub status: DomainStatus,
}

impl StudySession {
    pub fn load_domain(&mut self, domain_code: &str, file: PathBuf) -> Result<()> {
        let df = read_csv_table(&file)?;
        self.domains.insert(domain_code.to_string(), DomainState {
            source_file: file,
            source_data: df,
            status: DomainStatus::Loaded,
            ..Default::default()
        });
        Ok(())
    }
    
    pub fn map_domain(&mut self, domain_code: &str, config: MappingConfig) -> Result<()> {
        let state = self.domains.get_mut(domain_code)?;
        let mapped = self.mapping_service.apply_mapping(
            &state.source_data,
            &config,
            domain_code,
        )?;
        
        state.mapped_data = Some(mapped);
        state.mapping_config = Some(config);
        state.status = DomainStatus::Mapped;
        Ok(())
    }
    
    pub fn process_domain(&mut self, domain_code: &str) -> Result<()> {
        let state = self.domains.get_mut(domain_code)?;
        let mapped = state.mapped_data.as_ref().ok_or(/* error */)?;
        
        let processed = self.processing_service.process_domain(
            mapped,
            domain_code,
            &self.study_id,
            &ProcessingOptions::default(),
        )?;
        
        state.processed_data = Some(processed);
        state.status = DomainStatus::Processed;
        Ok(())
    }
    
    pub fn validate_domain(&mut self, domain_code: &str) -> Result<()> {
        let state = self.domains.get_mut(domain_code)?;
        let processed = state.processed_data.as_ref().ok_or(/* error */)?;
        
        let report = self.validation_service.validate_domain(processed, domain_code);
        state.validation_report = Some(report);
        Ok(())
    }
}
```

**Key Benefits:**

- ✅ No pipeline orchestration
- ✅ Each domain is independent
- ✅ Operations can be called in any order
- ✅ State is persistent and inspectable
- ✅ Easy to undo/redo
- ✅ GUI can call services directly

---

## What Can Be DELETED

### 1. Delete Entirely: `sdtm-cli` Crate

```bash
rm -rf crates/sdtm-cli/
```

**What's removed:**

- `pipeline.rs` - 1137 lines of orchestration
- `cli.rs` - CLI argument parsing
- `commands.rs` - CLI entry points
- `logging.rs` - CLI-specific logging
- `summary.rs` - CLI output formatting
- `types.rs` - CLI-specific result types

**Impact:** None for GUI

---

### 2. Extract from `sdtm-core`: Remove Pipeline Orchestration

**Delete:**

- `pipeline_context.rs` - Replace with simpler context
- `frame_builder.rs` → Move `build_mapped_domain_frame` to `sdtm-session`

**Keep & Simplify:**

- `processor.rs` → Extract individual functions, remove `process_domain` wrapper
- `domain_processors/` → Keep as-is (pure business logic)
- `ct_utils.rs` → Keep as-is

**Before (1 big function):**

```rust
pub fn process_domain(input: DomainProcessInput<'_>) -> Result<()> {
    // 200+ lines of orchestration
}
```

**After (many small functions):**

```rust
pub fn apply_usubjid_prefix(df: &mut DataFrame, study_id: &str) -> Result<()>
pub fn assign_sequences(df: &mut DataFrame, subject_col: &str) -> Result<()>
pub fn normalize_ct_values(df: &mut DataFrame, var: &Variable, ct: &Codelist) -> Result<()>
pub fn apply_domain_processor(df: &mut DataFrame, domain_code: &str) -> Result<()>
```

---

## Simplified Proposed Crate Structure

```
sdtm-transpiler/
├── crates/
│   ├── sdtm-model/          # Shared types (NO changes needed)
│   ├── sdtm-standards/      # Standards loading (NO changes needed)
│   ├── sdtm-ingest/         # CSV reading (minor refactor for preview)
│   ├── sdtm-map/            # Mapping engine (split suggest/apply) ⚠️
│   ├── sdtm-core/           # Domain processing (extract pure functions) ⚠️
│   ├── sdtm-transform/      # Transforms (make configurable) ⚠️
│   ├── sdtm-validate/       # Validation (support incremental) ⚠️
│   ├── sdtm-xpt/            # XPT format (NO changes needed)
│   ├── sdtm-report/         # Output generation (support selective export) ⚠️
│   ├── sdtm-session/        # NEW: Session state management
│   ├── sdtm-gui/            # NEW: egui + eframe GUI application
│   ├── sdtm-cli/            # CLI (refactor to use services) ⚠️
│   └── sdtm-pipeline/       # OPTIONAL: Extract linear pipeline for CLI
```

**Codebase Size Reduction:**

| Component                    | Before (Lines) | After (Lines) | Reduction |
| ---------------------------- | -------------- | ------------- | --------- |
| sdtm-cli (DELETED)           | ~2500          | 0             | -100%     |
| sdtm-core (refactored)       | ~2000          | ~800          | -60%      |
| Total orchestration overhead | ~3000          | 0             | -100%     |
| **New GUI code**             | 0              | ~2000         | +2000     |
| **Net change**               | ~7500          | ~2800         | **-63%**  |

---

## Updated Migration Strategy (GUI-Only)

### Phase 1: Foundation & Cleanup (Week 1)

**Goal:** Remove CLI, create session infrastructure

1. **Delete `sdtm-cli` crate**
   ```bash
   rm -rf crates/sdtm-cli/
   # Update Cargo.toml to remove sdtm-cli from workspace
   ```

2. **Create `sdtm-session` crate**
   ```bash
   cargo new --lib crates/sdtm-session
   ```
   - Define `StudySession`, `DomainState`
   - Create service interfaces (empty implementations)
   - Add session serialization

3. **Create `sdtm-gui` crate skeleton**
   ```bash
   cargo new crates/sdtm-gui
   ```
   - Add egui + eframe dependencies
   - Create basic app structure
   - Implement Home screen (loads session)

**Deliverable:** Compiles with CLI removed, GUI shell loads

---

### Phase 2: Extract Core Functions (Week 2)

**Goal:** Refactor `sdtm-core` into standalone functions

1. **Refactor `sdtm-core/src/processor.rs`**
   - Extract `apply_usubjid_prefix` from `process_domain`
   - Extract `assign_sequences` from `process_domain`
   - Extract `normalize_ct_column` from existing logic
   - Keep `domain_processors/` as-is

2. **Remove `PipelineContext`**
   - Delete `pipeline_context.rs`
   - Functions take explicit parameters instead

3. **Update `sdtm-session`**
   - Implement `ProcessingService` using new functions
   - Add tests

**Deliverable:** Core logic is modular and testable

---

### Phase 3: Mapping Service (Week 3)

**Goal:** Make mapping work independently

1. **Refactor `sdtm-map`**
   - `suggest_column_mappings()` - already mostly there
   - Add `apply_single_mapping()` helper
   - Add `preview_mapping()` for sample rows

2. **Implement `MappingService` in `sdtm-session`**
   - Wraps `sdtm-map` functions
   - Manages mapping state
   - Saves/loads mapping configs

3. **Build Mapping Tab in GUI**
   - Variable list (left panel)
   - Suggestion details (right panel)
   - Accept/reject buttons
   - Preview samples

**Deliverable:** Can map domains interactively

---

### Phase 4: Validation & Transforms (Week 4)

**Goal:** Real-time validation and configurable transforms

1. **Enhance `sdtm-validate`**
   - `validate_variable_values()` for incremental checks
   - `preview_ct_mapping()` for user confirmation
   - Already has `validate_domain()`

2. **Implement `ValidationService`**
   - Real-time CT checking
   - Issue tracking per domain

3. **Build Validation & Transform Tabs**
   - Show CT mismatches
   - Let user map values
   - Configure transforms

**Deliverable:** Interactive validation with fix suggestions

---

### Phase 5: Processing & Preview (Week 5)

**Goal:** Show final output with all transforms applied

1. **Complete `ProcessingService`**
   - Chain all transforms
   - USUBJID prefix
   - Sequence assignment
   - CT normalization

2. **Handle SUPPQUAL**
   - `identify_unmapped_columns()`
   - User selects what goes to SUPP
   - `build_suppqual_dataframe()`

3. **Implement Preview & SUPP Tabs**
   - Data table with pagination
   - Before/after comparison
   - SUPP configuration UI

**Deliverable:** Full domain processing visible

---

### Phase 6: Export & Polish (Weeks 6-7)

**Goal:** Export functionality and UX refinement

1. **Enhance `sdtm-report`**
   - `export_single_domain()`
   - `export_selected_domains()`
   - Progress callbacks for GUI

2. **Implement Export Screen**
   - Domain summary table
   - Format selection
   - Export button with progress

3. **Add Session Persistence**
   - Auto-save on changes
   - Recent studies list
   - Load/save dialogs

4. **Polish UI**
   - Keyboard shortcuts (Ctrl+S, etc.)
   - Error toasts
   - Help tooltips
   - Status indicators

**Deliverable:** Production-ready GUI

---

### Phase 7: Testing & Optimization (Week 8)

**Goal:** Ensure quality and performance

1. **Integration Tests**
   - Load real study data
   - Map → Process → Validate → Export
   - Compare output with expected

2. **Performance**
   - Profile large datasets (10,000+ rows)
   - Optimize table rendering
   - Add lazy loading if needed

3. **Documentation**
   - User guide with screenshots
   - Developer docs for services
   - Architecture diagrams

**Deliverable:** Tested, documented, ready to ship

---

## Updated Success Criteria

### Functional Requirements

✅ User can:

- [ ] Load a study folder and see discovered domains
- [ ] Map variables with AI-assisted suggestions
- [ ] Configure transforms and see previews
- [ ] Validate against CT with fix suggestions
- [ ] Preview final output before export
- [ ] Control SUPPQUAL generation
- [ ] Export individual or all domains
- [ ] Save and resume work
- [ ] Undo mapping changes

### Technical Requirements (Simplified)

✅ Architecture:

- [ ] `sdtm-cli` completely removed
- [ ] Services are stateless and reusable
- [ ] Each domain processes independently
- [ ] Operations are composable (no pipeline)
- [ ] Session state persists across restarts
- [ ] No linear execution constraints

### Performance Requirements

✅ Performance:

- [ ] Load 100+ domains in < 10 seconds
- [ ] Mapping suggestions appear instantly (< 500ms)
- [ ] Validation updates in real-time (< 1s)
- [ ] Preview renders 1000 rows smoothly (60 FPS)
- [ ] Export completes with progress indicator

### Code Quality Requirements

✅ Codebase:

- [ ] 60%+ reduction in orchestration code
- [ ] All business logic is pure functions
- [ ] Services have <5 dependencies
- [ ] 80%+ test coverage on services
- [ ] Zero circular dependencies

---

## Updated Risk Assessment

| Risk                            | Impact | Probability | Mitigation                                     |
| ------------------------------- | ------ | ----------- | ---------------------------------------------- |
| **Breaking existing workflows** | Low    | None        | No CLI to break!                               |
| **Performance with large data** | Medium | Low         | Pagination, lazy loading, streaming            |
| **egui learning curve**         | Low    | Medium      | Simple layouts first, iterate                  |
| **State management bugs**       | Medium | Low         | Comprehensive tests, use serde for persistence |
| **Session corruption**          | Low    | Low         | Versioned format, migration path               |

---

## Conclusion (Updated)

This simplified, GUI-only architecture eliminates **ALL** pipeline orchestration
overhead:

**What We're Removing:**

- ✂️ Entire `sdtm-cli` crate (~2500 lines)
- ✂️ `PipelineRunner` and stage orchestration (~1500 lines)
- ✂️ `PipelineContext` complexity (~200 lines)
- ✂️ Linear execution constraints
- ✂️ CLI argument parsing and validation

**What We're Gaining:**

- ✨ Modular, reusable services
- ✨ Interactive, non-linear workflows
- ✨ Per-domain processing
- ✨ Real-time validation and preview
- ✨ User control over every decision
- ✨ Persistent session state
- ✨ Simpler, more maintainable codebase

**Net Result:** **-63% code**, +100% flexibility

**Timeline:** 8 weeks to production-ready GUI

---

## Service Layer Architecture

Create a service layer that both GUI and CLI can use:

```rust
// In sdtm-session or new sdtm-services crate

pub struct MappingService {
    engine: MappingEngine,
    standards: Vec<Domain>,
}

impl MappingService {
    pub fn suggest_for_domain(&self, schema: &CsvSchema, domain: &Domain) 
        -> Vec<MappingSuggestion> { /* ... */ }
    
    pub fn apply_mapping(&self, df: &DataFrame, mapping: &MappingConfig, domain: &Domain) 
        -> Result<DataFrame> { /* ... */ }
    
    pub fn preview_mapping(&self, df: &DataFrame, variable: &Variable, column: &str) 
        -> Vec<(String, String)> { /* ... */ }
}

pub struct ValidationService {
    ct_registry: TerminologyRegistry,
}

impl ValidationService {
    pub fn validate_domain(&self, df: &DataFrame, domain: &Domain) 
        -> ValidationReport { /* ... */ }
    
    pub fn validate_variable(&self, values: &[String], variable: &Variable) 
        -> Vec<ValidationIssue> { /* ... */ }
    
    pub fn preview_ct_normalization(&self, values: &[String], codelist: &Codelist) 
        -> Vec<(String, Option<String>)> { /* ... */ }
}

pub struct ExportService {
    // ...
}

pub struct StudySessionService {
    pub mapping: MappingService,
    pub validation: ValidationService,
    pub export: ExportService,
    pub session: StudySession,
}
```

---

## Part 1: Understanding the Domain

### What is SDTM?

SDTM (Study Data Tabulation Model) is an FDA-required standard for organizing
clinical trial data. Key concepts:

| Concept                         | Description                          | Example                                  |
| ------------------------------- | ------------------------------------ | ---------------------------------------- |
| **Domain**                      | A dataset category                   | AE (Adverse Events), DM (Demographics)   |
| **Variable**                    | A column in a domain                 | USUBJID, AETERM, AESTDTC                 |
| **Core**                        | Required/Expected/Permissible        | USUBJID is Required in all domains       |
| **Controlled Terminology (CT)** | Allowed values for certain variables | SEX must be M, F, U, or UNDIFFERENTIATED |

### The Mapping Problem

Source data rarely matches SDTM structure exactly:

```
SOURCE DATA (ae.csv)              SDTM TARGET (AE domain)
──────────────────────            ─────────────────────────
SUBJECT_ID         ──────────→    USUBJID
ADVERSE_EVENT      ──────────→    AETERM
SEVERITY           ──────────→    AESEV (needs CT validation)
START_DATE         ──────────→    AESTDTC (needs date format)
EXTRA_NOTES        ──────────→    ??? (unmapped → SUPP)
???                              AEDECOD (no source)
```

### Key Challenges

1. **Ambiguous mappings**: "SEVERITY" could map to AESEV, AETOXGR, or AESEVCD
2. **CT mismatches**: Source value "Mild" must become "MILD" per CT
3. **Missing required variables**: USUBJID is required but may have a different
   name
4. **Unmapped columns**: Source columns with no SDTM equivalent go to SUPP
   domain
5. **Auto-generated fields**: STUDYID, DOMAIN, --SEQ are computed, not mapped

---

## Part 2: User Goals & Workflow

### Primary User Goal

> "I have source CSV files. I need to create SDTM-compliant XPT files for FDA
> submission."

### User Journey

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           USER JOURNEY                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   1. SELECT STUDY                                                            │
│   ───────────────                                                            │
│   User opens a folder containing source CSV files.                           │
│   System discovers files and detects domain types.                           │
│                                                                              │
│                              ↓                                               │
│                                                                              │
│   2. REVIEW DOMAINS                                                          │
│   ─────────────────                                                          │
│   User sees all discovered domains with status overview.                     │
│   User picks a domain to configure.                                          │
│                                                                              │
│                              ↓                                               │
│                                                                              │
│   3. CONFIGURE MAPPINGS (main work)                                          │
│   ─────────────────────────────────                                          │
│   For each SDTM variable, user either:                                       │
│     • Accepts a high-confidence suggestion                                   │
│     • Reviews and confirms a medium-confidence match                         │
│     • Manually selects from available source columns                         │
│     • Skips the variable (if Permissible)                                    │
│                                                                              │
│   For unmapped source columns, user either:                                  │
│     • Assigns to SUPP domain with QNAM/QLABEL                                │
│     • Skips (data will not be exported)                                      │
│                                                                              │
│                              ↓                                               │
│                                                                              │
│   4. RESOLVE CT ISSUES                                                       │
│   ────────────────────                                                       │
│   System validates mapped values against Controlled Terminology.             │
│   User maps invalid source values to valid CT terms.                         │
│                                                                              │
│                              ↓                                               │
│                                                                              │
│   5. EXPORT                                                                  │
│   ────────                                                                   │
│   User reviews summary across all domains.                                   │
│   User generates XPT, Define-XML, and/or Dataset-XML files.                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Time Spent Per Screen

Based on typical usage patterns:

| Screen        | Time | Reason                 |
| ------------- | ---- | ---------------------- |
| Home          | 5%   | Quick selection        |
| Domain Editor | 85%  | Main work happens here |
| Export        | 10%  | Review and generate    |

**Implication**: The Domain Editor must be exceptionally well-designed.

---

## Part 3: Information Architecture

### Screen Map

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SCREEN MAP                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                              HOME                                            │
│                                │                                             │
│                                │ (select domain)                             │
│                                ↓                                             │
│   ┌────────────────────────────────────────────────────────┐                │
│   │                     DOMAIN EDITOR                       │                │
│   │                                                         │                │
│   │   ┌─────────┐ ┌───────────┐ ┌────────────┐ ┌─────────┐ ┌──────┐     │                │
│   │   │ Mapping │ │ Transform │ │ Validation │ │ Preview │ │ SUPP │     │                │
│   │   └─────────┘ └───────────┘ └────────────┘ └─────────┘ └──────┘     │                │
│   │                                                         │                │
│   └────────────────────────────────────────────────────────┘                │
│                                │                                             │
│                                │ (done with all domains)                     │
│                                ↓                                             │
│                             EXPORT                                           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Information Hierarchy

**What's most important at each level?**

1. **Home Screen**
   - Which domains exist?
   - What's the status of each?
   - Where do I need to focus?
   - **How confident is the system in domain detection?**

2. **Domain Editor - Mapping Tab**
   - Which SDTM variables need attention?
   - What's the suggested mapping for each?
   - How confident is the system?

3. **Domain Editor - Transform Tab (NEW)**
   - How should values be transformed? (e.g., Date formats, CT normalization)
   - Are there bulk patterns to apply?

4. **Domain Editor - Validation Tab**
   - Which values fail CT validation?
   - What are the valid alternatives?
   - How many occurrences are affected?

5. **Domain Editor - Preview Tab**
   - What will the output look like?
   - Are transformations applied correctly?

6. **Domain Editor - SUPP Tab**
   - Which source columns are unmapped?
   - Should they be included in SUPPQUAL?
   - What are the QNAM/QLABEL values?

7. **Export Screen**
   - Are all domains ready?
   - What output formats do I want?
   - Where should files be saved?

---

## Part 4: Detailed Screen Specifications

### Screen 1: Home

**Purpose**: Study selection and domain overview.

**Layout**: Two sections stacked vertically.

#### Section A: Study Selection (shown when no study loaded)

```
╭──────────────────────────────────────────────────────────────────────────────╮
│                                                                ◐    ⚙       │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                                                                              │
│                                                                              │
│                          CDISC Transpiler                                    │
│                              v0.1.0                                          │
│                                                                              │
│                                                                              │
│                    ╭┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮                   │
│                    ┊                                      ┊                   │
│                    ┊              📁                      ┊                   │
│                    ┊                                      ┊                   │
│                    ┊     Drop study folder here          ┊                   │
│                    ┊        or click to browse           ┊                   │
│                    ┊                                      ┊                   │
│                    ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯                   │
│                                                                              │
│                                                                              │
│                    Recent                                                    │
│                                                                              │
│                    DEMO_STUDY_001                     2 days ago        →    │
│                    PHASE3_TRIAL_XYZ                  1 week ago        →    │
│                                                                              │
│                                                                              │
│                                                                              │
╰──────────────────────────────────────────────────────────────────────────────╯
```

**Interactions**:

- Drop zone: Drag folder or click to open native picker
- Recent items: Click to load directly
- Settings gear: Opens preferences

#### Section B: Domain Overview (shown when study loaded)

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←                                                              ◐    ⚙      │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  DEMO_STUDY_001                                                              │
│  ~/studies/demo_study_001                                    32 domains      │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │ Search domains...                                                    🔍 │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                                                                         │ │
│  │  Domain   Label                Class          Rows    Mapping  Val  St  │ │
│  │ ───────────────────────────────────────────────────────────────────────│ │
│  │                                                                         │ │
│  │  AE       Adverse Events       Events         423     14/18    2⚠   ●  │ │
│  │  CM       Concomitant Meds     Interventions  312     22/22    —    ✓  │ │
│  │  DA       Drug Accountability  Interventions   45     8/12     —    ○  │ │
│  │  DM       Demographics         Special         150     25/25    —    ✓  │ │
│  │  DS       Disposition          Events          150     10/10    —    ✓  │ │
│  │  EG       ECG Results          Findings       1205    18/24    5⚠   ●  │ │
│  │  EX       Exposure             Interventions   150     10/12    —    ○  │ │
│  │  IE       Incl/Excl Criteria   Findings        150     8/8      —    ✓  │ │
│  │  LB       Lab Results          Findings       2340    28/30    3✕   ✕  │ │
│  │  MH       Medical History      Events          890     15/15    —    ✓  │ │
│  │  PE       Physical Exam        Findings        450     12/14    1⚠   ●  │ │
│  │  QS       Questionnaires       Findings        780     20/20    —    ✓  │ │
│  │  SC       Subject Character.   Findings        150     6/6      —    ✓  │ │
│  │  SU       Substance Use        Interventions   210     8/10     —    ○  │ │
│  │  VS       Vital Signs          Findings        890     15/15    —    ✓  │ │
│  │  ...                                                                    │ │
│  │                                                                         │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│                                                                              │
│  Summary                                                                     │
│  ──────                                                                      │
│  ✓ 10 complete    ● 3 in progress    ○ 3 not started    ✕ 1 has errors      │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                             Export All  →    │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

##### List Columns

| Column  | Description                                             |
| ------- | ------------------------------------------------------- |
| Domain  | 2-letter domain code                                    |
| Label   | Human-readable name                                     |
| Class   | SDTM class (Events, Findings, Interventions, Special)   |
| Rows    | Record count in source file                             |
| Mapping | Variables mapped / total (e.g., `14/18`)                |
| Val     | Validation issues: `—` none, `2⚠` warnings, `3✕` errors |
| St      | Overall status icon                                     |

---

##### Status Icons

| Icon | Meaning                       | Color  |
| ---- | ----------------------------- | ------ |
| `○`  | Not started                   | Gray   |
| `●`  | In progress (needs attention) | Yellow |
| `✓`  | Complete                      | Green  |
| `✕`  | Has blocking errors           | Red    |

---

##### Sorting & Filtering

- **Default sort**: Status (errors first, then in progress, then not started,
  then complete)
- **Click column header** to sort by that column
- **Search box** filters by domain code or label
- **Keyboard**: Arrow keys to navigate, Enter to open domain

---

##### Row Interaction

| Action       | Result                              |
| ------------ | ----------------------------------- |
| Click row    | Opens Domain Editor for that domain |
| Hover row    | Subtle highlight                    |
| Double-click | Opens Domain Editor                 |

---

### Screen 2: Domain Editor

**Purpose**: The main workspace where 85% of user time is spent.

**Layout**: Header + Tab bar + Content area

**Tab Order**: Mapping → Transform → Validation → Preview → SUPP (workflow
sequence)

**Tab Badges**: Each tab shows a status badge to indicate pending work:

| Badge    | Meaning                |
| -------- | ---------------------- |
| `(3)`    | 3 items pending review |
| `(2⚠)`   | 2 warnings             |
| `(1✕)`   | 1 blocking error       |
| `✓`      | Complete, no issues    |
| _(none)_ | Not yet started        |

#### Tab A: Mapping

Master-detail layout: 1/3 variable list + 2/3 detail panel.

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  AE — Adverse Events                                          ◐    ⚙     │
├──────────────────────────────────────────────────────────────────────────────┤
│  Mapping (3)     SUPP (2)     Validation (5⚠)     Preview                    │
│  ━━━━━━━━━━━                                                                 │
├────────────────────────────┬─────────────────────────────────────────────────┤
│                            │                                                 │
│  Variables            14   │  SDTM Target                                    │
│                            │ ─────────────────────────────────────────────── │
│  ┌──────────────────────┐  │                                                 │
│  │ Name     Core    St  │  │  AETERM                                         │
│  ├──────────────────────┤  │  Reported Term for the Adverse Event            │
│  │ STUDYID   —      ⚙   │  │                                                 │
│  │ DOMAIN    —      ⚙   │  │  ┌─────────────┬─────────────────────────────┐  │
│  │ USUBJID  Req     ✓   │  │  │ Core        │ Required                    │  │
│  │ AESEQ     —      ⚙   │  │  │ Type        │ Char(200)                   │  │
│  │ AETERM   Req     ○  ◀│  │  │ Role        │ Topic                       │  │
│  │ AEDECOD  Req     ✓   │  │  │ Codelist    │ —                           │  │
│  │ AECAT    Perm    —   │  │  └─────────────┴─────────────────────────────┘  │
│  │ AEBODSYS Exp     ✓   │  │                                                 │
│  │ AESEV    Exp     ○   │  │  SDTM Examples                                  │
│  │ AESER    Exp     ✓   │  │  HEADACHE · NAUSEA · INJECTION SITE PAIN        │
│  │ AEREL    Exp     —   │  │                                                 │
│  │ AESTDTC  Req     ✓   │  │                                                 │
│  │ AEENDTC  Exp     ○   │  │  Source Column                                  │
│  │ ...                  │  │ ─────────────────────────────────────────────── │
│  │                      │  │                                                 │
│  │                      │  │  ┌─────────────────────────────────────────┐    │
│  │                      │  │  │ ADVERSE_EVENT_TERM              92% ●●○ │    │
│  │                      │  │  └─────────────────────────────────────────┘    │
│  │                      │  │                                                 │
│  │                      │  │  ┌─────────────┬─────────────────────────────┐  │
│  │                      │  │  │ Label       │ "Adverse Event Term"        │  │
│  │                      │  │  │ Type        │ Text                        │  │
│  │                      │  │  │ Unique      │ 847 values (68%)            │  │
│  │                      │  │  │ Missing     │ 12 rows (0.9%)              │  │
│  │                      │  │  └─────────────┴─────────────────────────────┘  │
│  │                      │  │                                                 │
│  │                      │  │  Sample Values                                  │
│  │                      │  │  Headache · Nausea · Fatigue · Dizziness        │
│  │                      │  │                                                 │
│  │                      │  │                                                 │
│  └──────────────────────┘  │  ┌─────────────────────────────────────────┐    │
│                            │  │ Select different column...           ▼  │    │
│                            │  └─────────────────────────────────────────┘    │
│                            │                                                 │
│                            │         Accept               Clear              │
│                            │                                                 │
╰────────────────────────────┴─────────────────────────────────────────────────╯
```

---

##### Left Panel (1/3) — Variable List

| Column | Description                                       |
| ------ | ------------------------------------------------- |
| Name   | SDTM variable name                                |
| Core   | `Req` / `Exp` / `Perm` (blank for auto-generated) |
| St     | Status icon                                       |

**Status Icons:**

| Icon | Meaning        | Color  |
| ---- | -------------- | ------ |
| `⚙`  | Auto-generated | Gray   |
| `✓`  | Mapped         | Green  |
| `○`  | Pending        | Yellow |
| `—`  | Skipped        | Gray   |

---

##### Right Panel (2/3) — Detail View

**Section 1: SDTM Target**

Shows what the source column needs to map TO:

| Field         | Description                                 |
| ------------- | ------------------------------------------- |
| Variable name | e.g., `AETERM`                              |
| Label         | e.g., "Reported Term for the Adverse Event" |
| Core          | Required / Expected / Permissible           |
| Type          | Char(length) or Num                         |
| Role          | Identifier, Topic, Qualifier, Timing        |
| Codelist      | NCI code if CT-controlled (e.g., C66767)    |
| SDTM Examples | Example values from SDTM documentation      |

**Section 2: Source Column**

Shows the suggested/selected source column:

| Field         | Description                                |
| ------------- | ------------------------------------------ |
| Column name   | e.g., `ADVERSE_EVENT_TERM`                 |
| Confidence    | Score with visual indicator (●●○ = Medium) |
| Label         | Column description from source metadata    |
| Type          | Text or Numeric                            |
| Unique        | Count and percentage of unique values      |
| Missing       | Count and percentage of null/empty rows    |
| Sample Values | 5-10 actual values from the data           |

**Confidence Indicator:**

| Score  | Visual | Level                       |
| ------ | ------ | --------------------------- |
| ≥ 95%  | `●●●`  | High — likely correct       |
| 80-94% | `●●○`  | Medium — review recommended |
| 60-79% | `●○○`  | Low — needs verification    |

**Actions:**

| Button   | Action                           |
| -------- | -------------------------------- |
| Accept   | Confirms the mapping             |
| Clear    | Removes the mapping              |
| Dropdown | Select a different source column |

**Mapping Method:**

| Method   | Description                                       |
| -------- | ------------------------------------------------- |
| Column   | Map directly to a source column (default)         |
| Constant | Assign a hardcoded value (e.g., "USA")            |
| Derived  | Calculated from other columns (via Transform tab) |

---

#### Tab B: Transform (NEW)

Configure value transformations and bulk patterns.

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  AE — Adverse Events                                          ◐    ⚙     │
├──────────────────────────────────────────────────────────────────────────────┤
│  Mapping ✓       Transform (2)     Validation (5⚠)     Preview     SUPP      │
│                  ━━━━━━━━━━━━━                                               │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Value Transformations                                                       │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ Variable   Source Column       Transformation               Sample     │  │
│  ├────────────────────────────────────────────────────────────────────────┤  │
│  │ AESTDTC    START_DATE          Date (MM/DD/YYYY → ISO)      2024-01-15 │  │
│  │ AEENDTC    END_DATE            Date (MM/DD/YYYY → ISO)      2024-01-20 │  │
│  │ AESEV      SEVERITY            CT Map (Grade 1 → MILD)      MILD       │  │
│  │ AETERM     ADVERSE_EVENT       Uppercase                    HEADACHE   │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  Bulk Patterns                                                               │
│                                                                              │
│  ┌─────────────────────────────────────────────────────┐                     │
│  │  Pattern Mapping                                     │                     │
│  │                                                      │                     │
│  │  Source Pattern:  *_DATE                            │                     │
│  │  Target Pattern:  {DOMAIN}*DTC                      │                     │
│  │                                                      │                     │
│  │  Preview:                                            │                     │
│  │    START_DATE  →  AESTDTC  ✓                        │                     │
│  │    END_DATE    →  AEENDTC  ✓                        │                     │
│  │                                                      │                     │
│  │                         Apply Pattern               │                     │
│  └─────────────────────────────────────────────────────┘                     │
│                                                                              │
╰──────────────────────────────────────────────────────────────────────────────╯
```

#### Tab C: Validation

Shows CT validation issues that must be resolved before export.

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  AE — Adverse Events                                          ◐    ⚙     │
├──────────────────────────────────────────────────────────────────────────────┤
│  Mapping (3)     SUPP (2)     Validation (5⚠)     Preview                    │
│                              ━━━━━━━━━━━━━━                                  │
├────────────────────────────────────┬─────────────────────────────────────────┤
│                                    │                                         │
│  3 issues need resolution          │                                         │
│                                    │   AESEV — Severity                      │
│  ┌──────────────────────────────┐  │   Codelist: C66769                      │
│  │                              │  │   Extensible: No                        │
│  │  ┃ AESEV                     │  │                                         │
│  │    Severity            ERROR │  │   This codelist is non-extensible.      │
│  │    5 invalid values          │  │   All values must match exactly.        │
│  │                              │  │                                         │
│  │    AEREL                     │  │                                         │
│  │    Causality           WARN  │  │   Invalid values found:                 │
│  │    1 sponsor extension       │  │                                         │
│  │                              │  │   ┌─────────────────────────────────┐   │
│  │    AEOUT                     │  │   │ Source        Count   Map to    │   │
│  │ **Searchable Combobox** to select valid CT term (fuzzy matching)
4. Apply button to save resolutions
```

┌─────────────────────────────────────┐ │ 🔍 Search CT values... │
├─────────────────────────────────────┤ │ ● MILD │ │ MODERATE │ │ SEVERE │
└─────────────────────────────────────┘

```│   │ "Mild"        45      MILD   ▼  │   │
│  │                              │  │   │ "Moderate"    38      MODERATE▼ │   │
│  └──────────────────────────────┘  │   │ "Severe"      12      SEVERE ▼  │   │
│                                    │   │ "Grade 1"      5      [Select]▼ │   │
│                                    │   │ "Grade 2"      3      [Select]▼ │   │
│                                    │   └─────────────────────────────────┘   │
│                                    │                                         │
│                                    │   Valid CT values:                      │
│                                    │   MILD, MODERATE, SEVERE                │
│                                    │                                         │
│                                    │                     Apply All           │
│                                    │                                         │
╰────────────────────────────────────┴─────────────────────────────────────────╯
```

**Left Panel: Issue List**

Each issue shows:

- Variable name
- Short description
- Severity badge (ERROR or WARN)
- Count of affected values

**Severity Meanings**:

| Severity | Codelist Type  | Impact                        |
| -------- | -------------- | ----------------------------- |
| ERROR    | Non-extensible | Blocks XPT export             |
| WARN     | Extensible     | Allowed but flagged in report |

**Right Panel: Resolution**

For the selected issue:

1. Codelist information
2. Explanation of the issue
3. Table of invalid values with:
   - Source value
   - Occurrence count
   - Dropdown to select valid CT term
4. Apply button to save resolutions

---

#### Tab D: Preview

Shows transformed data before export. This preview reflects the **Main Domain**
dataset (e.g., `AE`). Supplemental qualifiers are previewed in the SUPP tab.

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  AE — Adverse Events                                          ◐    ⚙     │
├──────────────────────────────────────────────────────────────────────────────┤
│  Mapping ✓       SUPP ✓       Validation ✓       Preview                     │
│                                                  ━━━━━━━                     │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ STUDYID   DOMAIN  USUBJID     AESEQ  AETERM      AESEV     AESTDTC    │  │
│  ├────────────────────────────────────────────────────────────────────────┤  │
│  │ DEMO      AE      DEMO-001    1      Headache    MILD      2024-01-15 │  │
│  │ DEMO      AE      DEMO-001    2      Nausea      MODERATE  2024-01-16 │  │
│  │ DEMO      AE      DEMO-002    1      Fatigue     MILD      2024-01-17 │  │
│  │ DEMO      AE      DEMO-002    2      Dizziness   SEVERE    2024-01-18 │  │
│  │ DEMO      AE      DEMO-003    1      Headache    MILD      2024-01-19 │  │
│  │ DEMO      AE      DEMO-003    2      Insomnia    MODERATE  2024-01-20 │  │
│  │ DEMO      AE      DEMO-004    1      Rash        MILD      2024-01-21 │  │
│  │ DEMO      AE      DEMO-004    2      Fatigue     MILD      2024-01-22 │  │
│  │                                                                        │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  Rows 1-50 of 423                                            ←   1  2  3  → │
│                                                                              │
│  Notes:                                                                      │
│  • STUDYID, DOMAIN, and AESEQ are auto-generated                            │
│  • AESEV values normalized to CDISC CT                                      │
│  • Dates converted to ISO 8601 format                                       │
│                                                                              │
╰──────────────────────────────────────────────────────────────────────────────╯
```

**FeatureE**:

- Scrollable data table with SDTM column headers
- Shows transformed values (CT normalized, dates formatted)
- Auto-generated columns populated
- Pagination for large datasets
- Notes section explaining transformations applied

---

#### Tab B: SUPP

Manages unmapped source columns as Supplemental Qualifiers (SUPPQUAL).

Source columns that don't map to standard SDTM variables can be included in
SUPP-- domains (e.g., SUPPAE, SUPPDM). This tab allows users to configure which
columns to include and define their QNAM/QLABEL.

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  AE — Adverse Events                                          ◐    ⚙      │
├──────────────────────────────────────────────────────────────────────────────┤
│  Mapping (3)     SUPP (2)     Validation (5⚠)     Preview                    │
│                 ━━━━━━━━                                                     │
├────────────────────────────┬─────────────────────────────────────────────────┤
│                            │                                                 │
│  Unmapped Columns      3   │  EXTRA_NOTES                                    │
│                            │  "Additional Notes"                             │
│  ┌──────────────────────┐  │                                                 │
│  │ Column       Action  │  │  ┌─────────────┬─────────────────────────────┐  │
│  ├──────────────────────┤  │  │ Type        │ Text                        │  │
│  │ EXTRA_NOTES  SUPP   ◀│  │  │ Unique      │ 312 values (25%)            │  │
│  │ INTERNAL_FL  Skip    │  │  │ Missing     │ 45 rows (3.6%)              │  │
│  │ CUSTOM_CODE  ?       │  │  └─────────────┴─────────────────────────────┘  │
│  │                      │  │                                                 │
│  └──────────────────────┘  │  Sample Values                                  │
│                            │  "Patient reported mild discomfort" ·           │
│                            │  "No issues noted" · "Follow-up required"       │
│                            │                                                 │
│                            │                                                 │
│                            │  Action                                         │
│                            │ ─────────────────────────────────────────────── │
│                            │                                                 │
│                            │  ● Add to SUPPAE                                │
│                            │  ○ Skip (exclude from output)                   │
│                            │                                                 │
│                            │                                                 │
│                            │  SUPPQUAL Configuration                         │
│                            │                                                 │
│                            │  QNAM     ┌─────────────────────────────────┐   │
│                            │           │ AENOTES                         │   │
│                            │           └─────────────────────────────────┘   │
│                            │           Max 8 characters, uppercase           │
│                            │                                                 │
│                            │  QLABEL   ┌─────────────────────────────────┐   │
│                            │           │ Additional Notes                │   │
│                            │           └─────────────────────────────────┘   │
│                            │           Max 40 characters                     │
│                            │                                                 │
│                            │                              Save               │
│                            │                                                 │
╰────────────────────────────┴─────────────────────────────────────────────────╯
```

---

##### Left Panel — Unmapped Columns

| Column | Description                     |
| ------ | ------------------------------- |
| Column | Source column name              |
| Action | `SUPP` / `Skip` / `?` (pending) |

---

##### Right Panel — Column Detail

**Source Column Info:**

| Field         | Description                             |
| ------------- | --------------------------------------- |
| Column name   | Source column name                      |
| Label         | Description from source metadata        |
| Type          | Text or Numeric                         |
| Unique        | Count and percentage of unique values   |
| Missing       | Count and percentage of null/empty rows |
| Sample Values | Preview of actual data                  |

**Action Selection:**

| Option      | Result                               |
| ----------- | ------------------------------------ |
| Add to SUPP | Include in SUPPAE/SUPPDM/etc. domain |
| Skip        | Exclude from all output              |

**SUPPQUAL Configuration** (when Add to SUPP selected):

| Field  | Constraint             | Description                                |
| ------ | ---------------------- | ------------------------------------------ |
| QNAM   | Max 8 chars, uppercase | Qualifier variable name (e.g., `AENOTES`)  |
| QLABEL | Max 40 chars           | Qualifier label (e.g., "Additional Notes") |

The system auto-suggests QNAM based on domain prefix + abbreviated column name.

---

##### Empty State

When all source columns are mapped to SDTM variables:

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  AE — Adverse Events                                          ◐    ⚙      │
├──────────────────────────────────────────────────────────────────────────────┤
│  Mapping ✓       SUPP ✓       Validation        Preview                      │
│                 ━━━━━━                                                       │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                                                                              │
│                                                                              │
│                                    ✓                                         │
│                                                                              │
│                     No unmapped source columns                               │
│                                                                              │
│              All source columns mapped to SDTM variables                     │
│                                                                              │
│                                                                              │
│                                                                              │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

### Screen 3: Export

**Purpose**: Final review and file generation.

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  Export                                                     ◐    ⚙        │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                                                                              │
│     Summary                                                                  │
│                                                                              │
│     ┌────────────────────────────────────────────────────────────────────┐   │
│     │  Domain     Variables    Mapped      Issues     Ready              │   │
│     ├────────────────────────────────────────────────────────────────────┤   │
│     │  DM         25           25/25       0          ✓                  │   │
│     │  AE         18           16/18       2 warn     ✓                  │   │
│     │  CM         22           22/22       0          ✓                  │   │
│     │  LB         30           28/30       3 error    ✕                  │   │
│     │  VS         15           15/15       0          ✓                  │   │
│     │  EX         12           10/12       0          ○                  │   │
│     └────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│     ⚠ LB has 3 CT errors that must be resolved before XPT export.            │
│     ○ EX has 2 unmapped Required variables.                                  │
│                                                                              │
│                                                                              │
│     Output                                                                   │
│                                                                              │
│     ┌────────────────────────────────────────────────────────────────────┐   │
│     │                                                                    │   │
│     │  Folder    ~/output/demo_study                         Browse      │   │
│     │                                                                    │   │
│     │  ☑  XPT files (SAS Transport v5)                                   │   │
│     │  ☑  Define-XML 2.0                                                 │   │
│     │  ☐  Dataset-XML                                                    │   │
│     │                                                                    │   │
│     │  ☐  Skip domains with errors                                       │   │
│     │  ☑  Include SUPP domains                                           │   │
│     │                                                                    │   │
│     └────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│                                                                              │
│                                                Generate Files                │
│                                                                              │
│                                                                              │
╰──────────────────────────────────────────────────────────────────────────────╯
```

**Summary Table Columns**:

| Column    | Description                                      |
| --------- | ------------------------------------------------ |
| Domain    | Domain code                                      |
| Variables | Total SDTM variables for this domain             |
| Mapped    | X/Y where X is mapped and Y is total             |
| Issues    | CT validation issues (errors block XPT)          |
| Ready     | ✓ = ready, ✕ = blocked by errors, ○ = incomplete |

**Output Options**:

- **XPT**: Standard submission format (blocked by errors)
- **Define-XML**: Metadata document
- **Dataset-XML**: Alternative to XPT
- **Skip domains with errors**: Export others even if some have issues
- **Include SUPP**: Generate supplemental qualifier domains

---

### Dialog: SUPP Assignment

```
╭─────────────────────────────────────────────────────╮
│                                                     │
│  Assign to SUPPAE                                   │
│                                                     │
│  These columns will be added to the                 │
│  supplemental qualifiers domain.                    │
│                                                     │
│  ┌─────────────────────────────────────────────┐    │
│  │                                             │    │
│  │  ☑  EXTRA_NOTES                             │    │
│  │      QNAM    AENOTES                        │    │
│  │      QLABEL  Extra Notes                    │    │
│  │                                             │    │
│  │  ☑  INTERNAL_FLAG                           │    │
│  │      QNAM    AEINTFL                        │    │
│  │      QLABEL  Internal Flag                  │    │
│  │                                             │    │
│  │  ☐  CUSTOM_CODE  (skip)                     │    │
│  │                                             │    │
│  └─────────────────────────────────────────────┘    │
│                                                     │
│  QNAM must be ≤8 characters, uppercase.             │
│                                                     │
│                        Cancel            Apply      │
│                                                     │
╰─────────────────────────────────────────────────────╯
```

---

## Part 5: Visual Design System

### Colors

```rust
pub mod colors {
    use egui::Color32;

    // Backgrounds
    pub const BG_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
    pub const BG_SECONDARY: Color32 = Color32::from_rgb(249, 250, 251);
    pub const BG_HOVER: Color32 = Color32::from_rgb(243, 244, 246);

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(17, 24, 39);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(107, 114, 128);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(156, 163, 175);

    // Semantic
    pub const ACCENT: Color32 = Color32::from_rgb(59, 130, 246);
    pub const SUCCESS: Color32 = Color32::from_rgb(16, 185, 129);
    pub const WARNING: Color32 = Color32::from_rgb(245, 158, 11);
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68);

    // Borders
    pub const BORDER: Color32 = Color32::from_rgb(229, 231, 235);
}
```

### Typography

| Use            | Size | Weight |
| -------------- | ---- | ------ |
| Page title     | 20px | 600    |
| Section header | 16px | 600    |
| Body           | 14px | 400    |
| Small/Label    | 12px | 500    |

### Spacing

| Token | Value |
| ----- | ----- |
| xs    | 4px   |
| sm    | 8px   |
| md    | 16px  |
| lg    | 24px  |
| xl    | 32px  |

### Components

| Component | Radius | Padding     |
| --------- | ------ | ----------- |
| Button    | 6px    | 16px × 10px |
| Card      | 8px    | 20px        |
| Input     | 6px    | 12px × 8px  |
| Badge     | 4px    | 8px × 4px   |

---

## Part 6: State Management

### Application State

```rust
pub struct AppState {
    pub view: View,
    pub study: Option<StudyState>,
    pub preferences: Preferences,
    pub toasts: Vec<Toast>,
}

pub enum View {
    Home,
    DomainEditor { domain: String, tab: EditorTab },
    Export,
}

pub enum EditorTab {
    Mapping,
    Transform,
    Validation,
    Preview,
    Supp,
}
```

### Study State

```rust
pub struct StudyState {
    pub study_id: String,
    pub path: PathBuf,
    pub domains: BTreeMap<String, DomainState>,
}

pub struct DomainState {
    pub code: String,
    pub label: String,
    pub source_file: PathBuf,
    pub row_count: usize,
    pub variables: Vec<VariableState>,
    pub unmapped_columns: Vec<UnmappedColumn>,
    pub ct_issues: Vec<CtIssue>,
    pub selected_variable: Option<usize>,
}
```

### Variable State

```rust
pub struct VariableState {
    pub spec: Variable,           // From SDTM standards
    pub mapping: MappingState,
}

pub enum MappingState {
    /// Auto-generated by system (STUDYID, DOMAIN, --SEQ)
    Auto,

    /// Mapped to a source column
    Mapped {
        source_column: String,
        confidence: f32,
    },

    /// Assigned a constant value
    Constant {
        value: String,
    },

    /// Derived via transformation logic
    Derived {
        logic: String,
    },

    /// Has suggestion(s) awaiting review
    Pending {
        suggestions: Vec<Suggestion>,
    },

    /// No mapping, no suggestions
    Unmapped,

    /// User explicitly skipped
    Skipped,
}

pub struct Suggestion {
    pub source_column: String,
    pub confidence: f32,
    pub sample_values: Vec<String>,
    pub match_reasons: Vec<String>,
}
```

### Unmapped Column

```rust
pub struct UnmappedColumn {
    pub name: String,
    pub assignment: UnmappedAssignment,
}

pub enum UnmappedAssignment {
    /// Not yet decided
    Pending,

    /// Assigned to SUPP domain
    Supp { qnam: String, qlabel: String },

    /// Explicitly skipped
    Skip,
}
```

### CT Issue

```rust
pub struct CtIssue {
    pub variable: String,
    pub codelist_code: String,
    pub extensible: bool,
    pub invalid_values: Vec<InvalidValue>,
}

pub struct InvalidValue {
    pub source_value: String,
    pub count: usize,
    pub resolution: Option<String>,  // Selected CT term
}
```

---

## Part 7: Keyboard Shortcuts

### Global

| Shortcut | Action                 |
| -------- | ---------------------- |
| `Cmd+O`  | Open study             |
| `Cmd+S`  | Save mappings          |
| `Cmd+E`  | Go to Export           |
| `Cmd+,`  | Settings               |
| `Esc`    | Go back / Close dialog |

### Domain Editor

| Shortcut    | Action                    |
| ----------- | ------------------------- |
| `↑` `↓`     | Navigate variable list    |
| `Enter`     | Accept suggestion         |
| `Backspace` | Clear mapping             |
| `Tab`       | Next field / Switch focus |

---

## Part 8: File Structure

```text
.
├── Cargo.toml
├── crates/
│   ├── sdtm-core/             # Domain models & business logic
│   ├── sdtm-ingest/           # File reading (CSV, SAS7bdat)
│   ├── sdtm-pipeline/         # NEW: Shared pipeline orchestration (extracted from CLI)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ingest.rs      # Stage 1: File discovery
│   │       ├── mapping.rs     # Stage 2: Column mapping
│   │       ├── processing.rs  # Stage 3-4: Domain rules
│   │       ├── validation.rs  # Stage 5: Conformance
│   │       ├── output.rs      # Stage 6: File generation
│   │       └── state.rs       # Pipeline state (for GUI progress)
│   └── sdtm-gui/              # NEW: GUI application
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── app.rs             # Main eframe::App implementation
│           ├── theme.rs           # Colors, spacing, fonts
│           ├── state/
│           │   ├── mod.rs
│           │   ├── app_state.rs   # Global application state
│           │   ├── study_state.rs # Loaded study data
│           │   ├── domain_state.rs # Per-domain working state
│           │   ├── mapping_state.rs # Interactive mapping session
│           │   └── validation_state.rs # CT resolution state
│           ├── views/
│           │   ├── mod.rs
│           │   ├── home.rs        # Home screen (selection + overview)
│           │   ├── domain_editor.rs # Main editor (delegates to tabs)
│           │   ├── tabs/
│           │   │   ├── mapping.rs # Mapping tab content
│           │   │   ├── transform.rs # NEW: Value transformations
│           │   │   ├── validation.rs # Validation tab content
│           │   │   ├── preview.rs # Preview tab content
│           │   │   └── supp.rs    # SUPP tab content
│           │   └── export.rs      # Export screen
│           ├── components/
│           │   ├── mod.rs
│           │   ├── domain_card.rs
│           │   ├── variable_list.rs
│           │   ├── mapping_card.rs
│           │   ├── ct_picker.rs   # Searchable CT selector
│           │   ├── data_table.rs
│           │   └── progress_bar.rs
│           └── dialogs/
│               ├── mod.rs
│               ├── supp_config.rs
│               └── pattern_mapping.rs # NEW: Bulk mapping pattern
```

---

## Part 9: Implementation Phases

### Phase 1: Foundation

- [ ] Create sdtm-gui crate
- [ ] Set up eframe window
- [ ] Implement theme system
- [ ] Create state structures
- [ ] Implement view routing

### Phase 2: Home Screen

- [ ] Drop zone with folder picker
- [ ] Recent studies persistence
- [ ] Domain card grid
- [ ] Study loading with progress

### Phase 3: Mapping Tab

- [ ] Variable list with status indicators
- [ ] Detail panel with suggestions
- [ ] Accept/reject flow
- [ ] Manual column selection
- [ ] Constant value assignment

### Phase 3.5: Transform Tab

- [ ] Transformation list
- [ ] Bulk pattern editor
- [ ] Derivation logic editor

### Phase 4: Validation Tab

- [ ] Issue list
- [ ] Resolution panel
- [ ] CT term selection

### Phase 5: Preview Tab

- [ ] Data table component
- [ ] Pagination
- [ ] Transformation notes

### Phase 6: Export

- [ ] Summary table
- [ ] Output options
- [ ] File generation

### Phase 7: Polish

- [ ] Keyboard shortcuts
- [ ] Toast notifications
- [ ] Error handling
- [ ] Settings dialog

---

## Summary

This GUI is designed around one core insight: **the user's job is to fill SDTM
variables with source data**.

The interface reflects this by:

1. **Centering on SDTM variables** — the left panel always shows what needs to
   be filled
2. **Highlighting what needs attention** — clear status indicators and filtering
3. **Providing contextual help** — suggestions with confidence scores and sample
   data
4. **Minimizing navigation** — everything for a domain happens in one place
5. **Progressive disclosure** — simple list view with details on selection

The five-tab design (Mapping → Transform → Validation → Preview → SUPP) follows
the natural workflow:

1. **Mapping** — Map source columns to SDTM variables
2. **Transform** — Apply value transformations and bulk patterns
3. **Validation** — Validate all mapped values against CT
4. **Preview** — See the final transformed output
5. **SUPP** — Decide what to do with unmapped columns

---

## Technical Implementation

### egui + eframe Architecture

**Framework Choice:** egui with eframe for cross-platform desktop deployment

**Why egui:**

- Pure Rust, integrates seamlessly with existing crates
- Immediate mode UI - simple state management
- Cross-platform (Windows, macOS, Linux)
- Good performance for data-heavy UIs
- Built-in widgets + easy custom widgets

**Application Structure:**

```rust
// In crates/sdtm-gui/src/main.rs

use eframe::egui;
use sdtm_session::StudySessionService;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1024.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "CDISC Transpiler",
        options,
        Box::new(|cc| Ok(Box::new(CdiscApp::new(cc)))),
    )
}

struct CdiscApp {
    // Application state
    session: Option<StudySessionService>,
    current_screen: Screen,
    current_domain: Option<String>,
    current_tab: DomainTab,
    
    // UI state
    selected_variable: Option<String>,
    search_query: String,
    error_message: Option<String>,
}

enum Screen {
    Home,
    DomainEditor,
    Export,
}

enum DomainTab {
    Mapping,
    Transform,
    Validation,
    Preview,
    Supp,
}

impl eframe::App for CdiscApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.current_screen {
            Screen::Home => self.render_home(ctx),
            Screen::DomainEditor => self.render_domain_editor(ctx),
            Screen::Export => self.render_export(ctx),
        }
    }
}
```

---

### State Management Pattern

**Separation of Concerns:**

```rust
// Application State (in GUI crate)
struct AppState {
    session: Option<StudySessionService>,
    ui_state: UiState,
}

// UI State (ephemeral, not persisted)
struct UiState {
    current_screen: Screen,
    selected_domain: Option<String>,
    selected_variable: Option<String>,
    search_filter: String,
    scroll_position: f32,
}

// Session State (persisted, in sdtm-session crate)
pub struct StudySession {
    study_id: String,
    domains: HashMap<String, DomainState>,
    // ... business state only
}
```

**Benefits:**

- Clean separation between business and UI state
- Session state can be saved/loaded independently
- UI state is ephemeral and disposable

---

### Layout Implementation

**Master-Detail Pattern:**

```rust
impl CdiscApp {
    fn render_domain_editor(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            self.render_header(ui);
        });
        
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            self.render_tabs(ui);
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("variable_list")
                .resizable(true)
                .default_width(250.0)
                .width_range(200.0..=400.0)
                .show_inside(ui, |ui| {
                    self.render_variable_list(ui);
                });
            
            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.render_detail_panel(ui);
            });
        });
    }
}
```

---

### Data Table Rendering

For large DataFrames (Preview tab):

```rust
use egui_extras::{TableBuilder, Column};

fn render_preview_table(&self, ui: &mut egui::Ui, df: &DataFrame) {
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
    
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto()) // Row number
        .columns(Column::auto(), df.width()) // Data columns
        .min_scrolled_height(0.0)
        .header(20.0, |mut header| {
            header.col(|ui| { ui.strong("#"); });
            for col_name in df.get_column_names() {
                header.col(|ui| { ui.strong(col_name); });
            }
        })
        .body(|mut body| {
            for row_idx in 0..df.height().min(1000) { // Limit rows
                body.row(text_height, |mut row| {
                    row.col(|ui| { ui.label(format!("{}", row_idx + 1)); });
                    for col_idx in 0..df.width() {
                        row.col(|ui| {
                            let value = df.column(col_idx)
                                .and_then(|s| s.get(row_idx).ok())
                                .map(|v| format!("{}", v))
                                .unwrap_or_default();
                            ui.label(value);
                        });
                    }
                });
            }
        });
}
```

---

### Async Operations

For long-running operations (loading data, validation):

```rust
use std::sync::mpsc::{channel, Receiver};
use std::thread;

enum BackgroundTask {
    LoadStudy(PathBuf),
    ValidateDomain(String),
    ExportDomains(Vec<String>),
}

struct TaskHandle {
    receiver: Receiver<TaskResult>,
    progress: Arc<AtomicU32>,
}

impl CdiscApp {
    fn start_background_task(&mut self, task: BackgroundTask) {
        let (tx, rx) = channel();
        let progress = Arc::new(AtomicU32::new(0));
        let progress_clone = progress.clone();
        
        thread::spawn(move || {
            let result = match task {
                BackgroundTask::LoadStudy(path) => {
                    // Load study with progress updates
                    // ...
                }
                // ... other tasks
            };
            tx.send(result).ok();
        });
        
        self.current_task = Some(TaskHandle { receiver: rx, progress });
    }
    
    fn check_background_task(&mut self, ctx: &egui::Context) {
        if let Some(handle) = &self.current_task {
            // Update progress indicator
            let progress = handle.progress.load(Ordering::Relaxed);
            // ... render progress bar ...
            
            // Check for completion
            if let Ok(result) = handle.receiver.try_recv() {
                self.handle_task_result(result);
                self.current_task = None;
            }
            
            // Request repaint for progress updates
            ctx.request_repaint();
        }
    }
}
```

---

### Styling and Theming

```rust
fn configure_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Professional color scheme
    style.visuals.window_fill = egui::Color32::from_rgb(248, 249, 250);
    style.visuals.panel_fill = egui::Color32::WHITE;
    
    // Status colors
    style.visuals.error_fg_color = egui::Color32::from_rgb(220, 53, 69);
    style.visuals.warn_fg_color = egui::Color32::from_rgb(255, 193, 7);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(13, 110, 253);
    
    // Typography
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    
    ctx.set_style(style);
}
```

---

## Migration Strategy

### Phase 1: Foundation (Weeks 1-2)

**Goal:** Set up infrastructure without breaking existing CLI

1. **Create `sdtm-session` crate**
   - Define `StudySession`, `DomainState`, and persistence
   - Add serialization with `serde_json`
   - Write unit tests for session management

2. **Create `sdtm-gui` crate skeleton**
   - Set up eframe application structure
   - Implement basic navigation (Home ↔ Domain Editor ↔ Export)
   - Create mock UI layouts with placeholder data

3. **Set up integration points**
   - Add feature flags to existing crates for GUI support
   - Ensure CLI still works with no regressions

**Deliverable:** Empty GUI shell that compiles and runs

---

### Phase 2: Decouple Mapping (Weeks 3-4)

**Goal:** Make mapping engine work independently

1. **Refactor `sdtm-map`**
   ```rust
   // Add new public APIs
   pub fn suggest_mappings(...) -> Vec<MappingSuggestion>
   pub fn apply_mapping(...) -> Result<DataFrame>
   pub fn preview_mapping(...) -> Vec<(String, String)>
   ```

2. **Integrate with GUI**
   - Implement Mapping tab UI
   - Connect to mapping service
   - Show suggestions with confidence scores
   - Allow user to accept/reject/modify

3. **Test both paths**
   - Verify CLI still uses `build_mapped_domain_frame`
   - Verify GUI can suggest and apply mappings independently

**Deliverable:** Working Mapping tab in GUI

---

### Phase 3: Decouple Validation (Weeks 5-6)

**Goal:** Make validation work incrementally

1. **Refactor `sdtm-validate`**
   ```rust
   pub fn validate_domain(...) -> ValidationReport
   pub fn validate_variable(...) -> Vec<ValidationIssue>
   pub fn preview_ct_normalization(...) -> Vec<(String, Option<String>)>
   ```

2. **Integrate with GUI**
   - Implement Validation tab UI
   - Show CT mapping suggestions
   - Allow user to resolve mismatches
   - Real-time validation feedback

3. **Add Transform tab**
   - Define `TransformRule` enum
   - Implement transform preview
   - Allow user to configure transforms

**Deliverable:** Working Validation and Transform tabs

---

### Phase 4: Processing and Preview (Weeks 7-8)

**Goal:** Extract pure processing functions

1. **Refactor `sdtm-core`**
   ```rust
   // Extract from process_domain into smaller functions
   pub fn apply_usubjid_prefix(...)
   pub fn assign_sequence_numbers(...)
   pub fn normalize_ct_column(...)
   ```

2. **Implement Preview tab**
   - Apply all transformations
   - Show DataFrame with pagination
   - Highlight CT-normalized values
   - Show before/after comparisons

3. **Make SUPPQUAL user-controlled**
   - Identify candidates
   - Show in SUPP tab
   - Let user approve/reject
   - Generate SUPP DataFrame

**Deliverable:** Full domain processing in GUI

---

### Phase 5: Export and Polish (Weeks 9-10)

**Goal:** Complete export functionality and UX polish

1. **Refactor `sdtm-report`**
   ```rust
   pub fn export_domain(...)
   pub fn export_domains_selective(...)
   pub fn preview_export(...)
   ```

2. **Implement Export screen**
   - Domain summary table
   - Output format selection
   - Selective export
   - Progress indicators

3. **Add session persistence**
   - Save/load session state
   - Recent studies list
   - Auto-save on changes

4. **Polish UI/UX**
   - Keyboard shortcuts
   - Error handling and validation
   - Help tooltips
   - Undo/redo (if time permits)

**Deliverable:** Complete GUI application

---

### Phase 6: Testing and Documentation (Weeks 11-12)

**Goal:** Ensure quality and maintainability

1. **Integration testing**
   - End-to-end GUI workflows
   - Comparison with CLI outputs
   - Performance testing with large datasets

2. **Documentation**
   - User guide with screenshots
   - Developer documentation for services
   - API documentation for refactored crates

3. **Packaging and distribution**
   - Build scripts for all platforms
   - Installation instructions
   - Release preparation

**Deliverable:** Production-ready GUI

---

## Success Criteria

### Functional Requirements

✅ User can:

- [ ] Load a study folder and see discovered domains
- [ ] Map variables with AI-assisted suggestions
- [ ] Configure value transformations
- [ ] Validate against Controlled Terminology
- [ ] Preview final output before export
- [ ] Control SUPPQUAL generation
- [ ] Export subsets of domains
- [ ] Save and resume work

### Technical Requirements

✅ Architecture:

- [ ] Services decoupled from pipeline orchestration
- [ ] Each domain can be processed independently
- [ ] Operations are reversible/undoable
- [ ] Session state can be persisted
- [ ] CLI remains functional with no regressions

### Performance Requirements

✅ Performance:

- [ ] Load 100+ domain files in < 10 seconds
- [ ] Mapping suggestions appear in < 1 second
- [ ] Validation updates in < 2 seconds
- [ ] Preview renders 1000 rows smoothly
- [ ] Export completes in reasonable time (< 30s for typical study)

### UX Requirements

✅ User Experience:

- [ ] Clear visual hierarchy
- [ ] Intuitive workflow (no getting stuck)
- [ ] Helpful error messages
- [ ] Progress indicators for long operations
- [ ] Keyboard shortcuts for common actions

---

## Risks and Mitigation

| Risk                                         | Impact | Probability | Mitigation                                           |
| -------------------------------------------- | ------ | ----------- | ---------------------------------------------------- |
| **Refactoring breaks CLI**                   | High   | Medium      | Maintain feature parity tests, incremental changes   |
| **Performance issues with large DataFrames** | High   | Low         | Lazy loading, pagination, virtual scrolling          |
| **egui learning curve**                      | Medium | Medium      | Start with simple layouts, iterate                   |
| **State management complexity**              | Medium | Medium      | Keep business logic in services, UI state minimal    |
| **Session persistence bugs**                 | Medium | Low         | Comprehensive unit tests, use stable serialization   |
| **Cross-platform issues**                    | Low    | Low         | Test early on all platforms, use eframe abstractions |

---

## Conclusion

This architecture document defines a comprehensive plan to transform the CDISC
Transpiler from a linear CLI pipeline into a modular, GUI-friendly application
while maintaining the existing CLI functionality.

**Key Principles:**

1. **Modularity**: Each operation can work independently
2. **Separation**: Business logic decoupled from orchestration
3. **Flexibility**: Users control the workflow, not forced into a pipeline
4. **Transparency**: Show what's happening, allow inspection and modification
5. **Reversibility**: Changes can be undone or adjusted

**Expected Outcomes:**

- Users can work more efficiently with visual feedback
- Complex mapping decisions are easier with preview
- Validation errors are resolved interactively
- The codebase is more maintainable and testable
- Both CLI and GUI share the same robust services

The migration is designed to be incremental and low-risk, with each phase
delivering tangible value. **By removing the CLI entirely, we eliminate 63% of
the codebase complexity while gaining a more flexible, maintainable
architecture.**

---

**Document Version:** 2.0 (GUI-Only)\
**Last Updated:** December 30, 2025\
**Status:** Ready for Implementation - Optimized for GUI-Only Development\
**Expected Timeline:** 8 weeks to production
