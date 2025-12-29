# CDISC Transpiler TUI Architecture & Workflow Design

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current Architecture Analysis](#current-architecture-analysis)
3. [Proposed TUI Architecture](#proposed-tui-architecture)
4. [Data Flow & State Management](#data-flow--state-management)
5. [UI/UX Workflow Design](#uiux-workflow-design)
6. [Component Architecture](#component-architecture)
7. [Screen Layouts & Wireframes](#screen-layouts--wireframes)
8. [Mapping Confidence System](#mapping-confidence-system)
9. [SUPP Domain Fallback Workflow](#supp-domain-fallback-workflow)
10. [Technical Implementation Roadmap](#technical-implementation-roadmap)
11. [Appendix: Data Structures](#appendix-data-structures)

---

## Executive Summary

### Problem Statement

The current CDISC Transpiler operates as a fully automated CLI tool that attempts to:

1. Discover source CSV files
2. Automatically map source columns to SDTM variables using fuzzy matching
3. Apply transformations and validations
4. Generate output files (XPT, Dataset-XML, Define-XML)

**The fundamental issue**: Full automation cannot achieve 100% accuracy because:

- Source data has non-standardized column names
- Domain-specific business logic requires human judgment
- Controlled Terminology (CT) mappings have ambiguous cases
- SUPP (Supplemental Qualifier) decisions need domain expertise

### Proposed Solution

Transform the CLI into an **interactive TUI (Terminal User Interface)** using `ratatui` that:

1. **Loads all metadata first** - Standards, CT, source data schema
2. **Presents mapping suggestions** - Shows confidence-scored mapping options
3. **Requires user confirmation** - High-confidence mappings shown for approval
4. **Offers alternatives** - Low-confidence mappings show ranked options
5. **Handles unmapped columns** - Clear workflow for SUPP domain fallback
6. **Displays rich context** - Source column description alongside SDTM variable metadata

### Key Benefits

| Aspect | Current CLI | Proposed TUI |
|--------|-------------|--------------|
| **Mapping Accuracy** | ~70-80% automated | 100% user-verified |
| **Error Handling** | Silent failures or warnings | Interactive resolution |
| **SUPP Decisions** | Automatic (may be wrong) | User-guided with context |
| **User Confidence** | Low (black box) | High (transparent) |
| **Learning Curve** | Steep (CLI flags) | Gentle (guided workflow) |

---

## Current Architecture Analysis

### Crate Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              sdtm-cli                                   │
│                         (Entry Point - CLI)                             │
│  • Parses CLI arguments                                                 │
│  • Initializes logging                                                  │
│  • Orchestrates pipeline                                                │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
┌───────────────────────┐ ┌─────────────────┐ ┌─────────────────────────┐
│      sdtm-core        │ │  sdtm-report    │ │     sdtm-validate       │
│  (Business Logic)     │ │ (Output Gen)    │ │   (Conformance)         │
│ • Domain processors   │ │ • XPT writer    │ │ • CT value checks       │
│ • CT normalization    │ │ • Dataset-XML   │ │ • Required variables    │
│ • USUBJID prefixing   │ │ • Define-XML    │ │ • Output gating         │
│ • --SEQ assignment    │ │ • SAS programs  │ │                         │
└───────────────────────┘ └─────────────────┘ └─────────────────────────┘
          │                       │
          ▼                       ▼
┌───────────────────────┐ ┌─────────────────┐
│     sdtm-ingest       │ │    sdtm-xpt     │
│  (Data Loading)       │ │ (XPT Format)    │
│ • CSV discovery       │ │ • SAS Transport │
│ • Schema detection    │ │   v5 format     │
│ • Metadata loading    │ │                 │
└───────────────────────┘ └─────────────────┘
          │
          ▼
┌───────────────────────┐ ┌─────────────────┐ ┌─────────────────────────┐
│       sdtm-map        │ │ sdtm-standards  │ │      sdtm-model         │
│   (Column Mapping)    │ │ (Standards IO)  │ │    (Pure Types)         │
│ • Fuzzy matching      │ │ • Load SDTMIG   │ │ • Domain, Variable      │
│ • Confidence scoring  │ │ • Load CT       │ │ • Term, Codelist        │
│ • Synonym detection   │ │ • Offline CSVs  │ │ • ValidationIssue       │
└───────────────────────┘ └─────────────────┘ └─────────────────────────┘
```

### Current Processing Pipeline

```
┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│   Discover   │──▶│    Ingest    │──▶│     Map      │──▶│   Process    │
│  CSV Files   │   │  Load Data   │   │   Columns    │   │   Domains    │
└──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘
                                             │                  │
                                             │ Automated        │
                                             │ (No Review)      │
                                             ▼                  ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│    Output    │◀──│     Gate     │◀──│   Validate   │◀──│  Transform   │
│    Files     │   │   Outputs    │   │     CT       │   │    Data      │
└──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘
```

### Key Data Structures (from `sdtm-model`)

```rust
// Domain definition from SDTMIG
pub struct Domain {
    pub code: String,                    // "AE", "DM", "LB"
    pub description: Option<String>,     // "Adverse Events"
    pub class_name: Option<String>,      // "Events"
    pub dataset_class: Option<DatasetClass>,
    pub label: Option<String>,
    pub structure: Option<String>,       // "One record per subject"
    pub variables: Vec<Variable>,
}

// Variable specification
pub struct Variable {
    pub name: String,                    // "AEDECOD"
    pub label: Option<String>,           // "Dictionary-Derived Term"
    pub data_type: VariableType,         // Char, Num
    pub role: Option<String>,            // "Topic", "Identifier"
    pub core: Option<String>,            // "Req", "Exp", "Perm"
    pub codelist_code: Option<String>,   // "C66729" (CT reference)
    pub order: Option<u32>,
}

// Mapping suggestion from sdtm-map
pub struct MappingSuggestion {
    pub source_column: String,           // Original column name
    pub target_variable: String,         // SDTM variable name
    pub confidence: f32,                 // 0.0 to 1.0+
    pub transformation: Option<String>,  // "uppercase", "date_iso8601"
}

// Confidence levels
pub enum ConfidenceLevel {
    High,    // ≥0.95 - Near-certain match
    Medium,  // ≥0.80 - Good match, review recommended
    Low,     // ≥0.60 - Weak match, needs verification
}
```

### Standards Metadata Available

The `standards/` directory contains rich metadata that should be leveraged in the TUI:

```
standards/
├── ct/                         # Controlled Terminology
│   └── SDTM_CT_*.csv          # Term definitions with codes
├── sdtmig/v3_4/
│   ├── Datasets.csv           # Domain metadata (class, structure)
│   ├── Variables.csv          # Variable definitions with:
│   │   • Variable Name        # AEDECOD, USUBJID
│   │   • Variable Label       # "Dictionary-Derived Term"
│   │   • Type                 # Char, Num
│   │   • CDISC CT Codelist    # C66729 (links to CT)
│   │   • Role                 # Identifier, Topic, Qualifier
│   │   • Core                 # Req, Exp, Perm
│   │   • Description          # Full description text
│   └── chapters/              # SDTMIG documentation
└── sdtm/                      # SDTM model specifications
```

---

## Proposed TUI Architecture

### New Crate Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              sdtm-tui (NEW)                             │
│                     (Terminal User Interface Layer)                     │
│  • Screen management (ratatui)                                          │
│  • User input handling                                                  │
│  • State management                                                     │
│  • Event loop                                                           │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
        ┌───────────────────────────┼───────────────────────────┐
        │                           │                           │
        ▼                           ▼                           ▼
┌───────────────┐         ┌─────────────────┐         ┌─────────────────┐
│  sdtm-cli     │         │   sdtm-core     │         │  sdtm-report    │
│ (Batch Mode)  │         │ (Business Logic)│         │ (Output Gen)    │
│ (Keep for CI) │         │  (Unchanged)    │         │  (Unchanged)    │
└───────────────┘         └─────────────────┘         └─────────────────┘
        │                           │                           │
        └───────────────────────────┼───────────────────────────┘
                                    ▼
                        ┌───────────────────────┐
                        │     sdtm-model        │
                        │   (+ TUI State Types) │
                        │   MappingDecision     │
                        │   ColumnReviewState   │
                        │   SuppDecision        │
                        └───────────────────────┘
```

### TUI Component Architecture

```
sdtm-tui/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Crate root
    ├── app.rs                    # Application state & lifecycle
    ├── event.rs                  # Event handling (keyboard, mouse, resize)
    ├── ui/
    │   ├── mod.rs
    │   ├── layout.rs             # Screen layout management
    │   ├── widgets/
    │   │   ├── mod.rs
    │   │   ├── column_list.rs    # Scrollable column list
    │   │   ├── mapping_panel.rs  # Mapping suggestion display
    │   │   ├── metadata_panel.rs # Source/target metadata
    │   │   ├── confidence_bar.rs # Visual confidence indicator
    │   │   ├── help_footer.rs    # Context-sensitive help
    │   │   └── progress_bar.rs   # Overall mapping progress
    │   └── screens/
    │       ├── mod.rs
    │       ├── welcome.rs        # Study selection screen
    │       ├── domain_select.rs  # Domain selection screen
    │       ├── mapping_review.rs # Main mapping review screen
    │       ├── supp_decision.rs  # SUPP domain fallback screen
    │       ├── summary.rs        # Final review before output
    │       └── output.rs         # Output generation progress
    ├── state/
    │   ├── mod.rs
    │   ├── app_state.rs          # Global application state
    │   ├── mapping_state.rs      # Mapping review state
    │   └── navigation.rs         # Screen navigation state
    └── actions.rs                # User action handlers
```

### Dependencies for TUI Crate

```toml
[dependencies]
ratatui = "0.29"                  # TUI framework
crossterm = "0.28"                # Terminal manipulation
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
anyhow = "1.0"

# Internal crates
sdtm-model = { path = "../sdtm-model" }
sdtm-core = { path = "../sdtm-core" }
sdtm-map = { path = "../sdtm-map" }
sdtm-standards = { path = "../sdtm-standards" }
sdtm-ingest = { path = "../sdtm-ingest" }
sdtm-validate = { path = "../sdtm-validate" }
sdtm-report = { path = "../sdtm-report" }
```

---

## Data Flow & State Management

### Application State Machine

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         TUI Application States                          │
└─────────────────────────────────────────────────────────────────────────┘

    ┌──────────┐
    │  Start   │
    └────┬─────┘
         │
         ▼
┌────────────────┐     ┌────────────────┐
│   Loading      │────▶│   Welcome      │
│   Standards    │     │   Screen       │
└────────────────┘     └───────┬────────┘
                               │ Select Study Folder
                               ▼
                       ┌────────────────┐
                       │   Discovering  │
                       │   Files        │
                       └───────┬────────┘
                               │ Domain files found
                               ▼
                       ┌────────────────┐
                       │   Domain       │◀─────┐
                       │   Selection    │      │
                       └───────┬────────┘      │
                               │ Select domain │
                               ▼               │
                       ┌────────────────┐      │
                       │   Loading      │      │
                       │   Domain Data  │      │
                       └───────┬────────┘      │
                               │ Compute mappings
                               ▼               │
┌────────────────┐     ┌────────────────┐      │
│   SUPP         │◀───▶│   Mapping      │──────┘
│   Decision     │     │   Review       │ Back to domain list
└───────┬────────┘     └───────┬────────┘
        │ Confirm SUPP         │ All columns mapped
        └──────────┬───────────┘
                   ▼
           ┌────────────────┐
           │   Summary      │
           │   Review       │
           └───────┬────────┘
                   │ Confirm & Generate
                   ▼
           ┌────────────────┐
           │   Output       │
           │   Generation   │
           └───────┬────────┘
                   │
                   ▼
              ┌─────────┐
              │  Done   │
              └─────────┘
```

### Core State Structures

```rust
/// Global application state
pub struct AppState {
    /// Current screen being displayed
    pub screen: Screen,
    
    /// Loaded SDTM standards (domains, variables)
    pub standards: Vec<Domain>,
    
    /// Loaded Controlled Terminology registry
    pub ct_registry: TerminologyRegistry,
    
    /// Study being processed
    pub study: Option<StudyState>,
    
    /// User preferences (saved between sessions)
    pub preferences: UserPreferences,
}

/// State for a study being processed
pub struct StudyState {
    /// Study identifier
    pub study_id: String,
    
    /// Path to study folder
    pub study_folder: PathBuf,
    
    /// Discovered domain files
    pub discovered_domains: BTreeMap<String, Vec<DomainFile>>,
    
    /// Mapping states per domain
    pub domain_mappings: BTreeMap<String, DomainMappingState>,
    
    /// Overall progress (domains completed / total)
    pub progress: (usize, usize),
}

/// Discovered domain file
pub struct DomainFile {
    pub path: PathBuf,
    pub filename: String,
    pub row_count: usize,
    pub columns: Vec<SourceColumn>,
}

/// Source column with metadata
pub struct SourceColumn {
    /// Original column name from CSV
    pub name: String,
    
    /// Label from CSV header row 2 (if present)
    pub label: Option<String>,
    
    /// Sample values (first 5 non-null)
    pub sample_values: Vec<String>,
    
    /// Data characteristics
    pub hints: ColumnHint,
}

/// Mapping state for a single domain
pub struct DomainMappingState {
    /// Domain code (e.g., "AE", "DM")
    pub domain_code: String,
    
    /// Domain metadata from standards
    pub domain: Domain,
    
    /// Source columns from CSV files
    pub source_columns: Vec<SourceColumn>,
    
    /// Mapping decisions (one per source column)
    pub decisions: Vec<MappingDecision>,
    
    /// Currently selected column index
    pub selected_index: usize,
    
    /// Filter/search text
    pub filter_text: String,
    
    /// View mode (all, pending, confirmed, supp)
    pub view_mode: ViewMode,
}

/// User's decision for a single column mapping
pub enum MappingDecision {
    /// Awaiting user review
    Pending {
        suggestions: Vec<RankedSuggestion>,
    },
    
    /// User confirmed a mapping
    Confirmed {
        target_variable: String,
        confidence: f32,
        was_auto: bool,  // High confidence, auto-suggested
    },
    
    /// User decided to map to SUPP domain
    SuppQual {
        qnam: String,       // SUPP variable name
        qlabel: String,     // SUPP variable label
        reason: String,     // Why mapped to SUPP
    },
    
    /// User decided to skip/ignore column
    Skipped {
        reason: Option<String>,
    },
}

/// Ranked mapping suggestion with full context
pub struct RankedSuggestion {
    /// Target SDTM variable
    pub variable: Variable,
    
    /// Confidence score (0.0 to 1.0+)
    pub confidence: f32,
    
    /// Confidence level category
    pub level: ConfidenceLevel,
    
    /// Why this mapping was suggested
    pub reasoning: Vec<String>,
    
    /// Potential issues with this mapping
    pub warnings: Vec<String>,
}
```

---

## UI/UX Workflow Design

### Workflow Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         User Workflow Overview                          │
└─────────────────────────────────────────────────────────────────────────┘

1. LOAD STUDY
   ├── Select study folder (file browser or path input)
   ├── System discovers CSV files
   └── System identifies potential domains

2. REVIEW DOMAIN MAPPINGS (per domain)
   ├── View all source columns with mapping suggestions
   ├── For HIGH confidence (≥95%):
   │   ├── Show auto-suggested mapping
   │   └── User confirms or overrides
   ├── For MEDIUM confidence (80-95%):
   │   ├── Show ranked alternatives
   │   └── User selects best option
   ├── For LOW confidence (<80%):
   │   ├── Show ranked alternatives + SUPP option
   │   └── User selects or maps to SUPP
   └── For UNMAPPED columns:
       ├── Show all possible targets
       ├── User selects target OR
       └── Maps to SUPP domain

3. REVIEW SUPP DECISIONS
   ├── Show all columns mapped to SUPP
   ├── User confirms QNAM and QLABEL
   └── User can change decision

4. SUMMARY & OUTPUT
   ├── Show mapping summary per domain
   ├── Show validation warnings/errors
   └── Generate outputs (XPT, XML, SAS)
```

### Key UI/UX Principles

1. **Progressive Disclosure**
   - Start with high-level overview
   - Drill down into details on demand
   - Never overwhelm with information

2. **Context Always Visible**
   - Source column name + description always shown
   - Target variable name + description always shown
   - Confidence indicator always visible

3. **Clear Visual Hierarchy**
   - High confidence: Green indicators, minimal attention needed
   - Medium confidence: Yellow/amber indicators, review recommended
   - Low confidence: Red indicators, action required
   - SUPP candidates: Blue indicators, special handling

4. **Keyboard-First Design**
   - All actions accessible via keyboard
   - Consistent keybindings across screens
   - Escape always goes back
   - Enter always confirms

5. **Error Prevention**
   - Warn before overwriting decisions
   - Confirm before generating outputs
   - Show impact of decisions

---

## Screen Layouts & Wireframes

### Screen 1: Welcome Screen

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        CDISC SDTM Transpiler v0.1.0                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│                     ╔═══════════════════════════════╗                   │
│                     ║   Welcome to SDTM Transpiler  ║                   │
│                     ╚═══════════════════════════════╝                   │
│                                                                         │
│   Convert clinical trial data to CDISC SDTM format with guided mapping. │
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  Recent Studies:                                                │   │
│   │                                                                 │   │
│   │  > DEMO_CF1234_NL_20250120    Last opened: 2025-01-20 10:48    │   │
│   │    DEMO_GDISC_20240903        Last opened: 2024-09-03 07:29    │   │
│   │                                                                 │   │
│   │  ──────────────────────────────────────────────────────────────│   │
│   │  [B] Browse for study folder                                   │   │
│   │  [P] Paste folder path                                         │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│   Standards loaded: SDTMIG v3.4 • CT 2024-03-29 • 52 domains           │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  [↑↓] Navigate  [Enter] Select  [B] Browse  [P] Paste  [Q] Quit        │
└─────────────────────────────────────────────────────────────────────────┘
```

### Screen 2: Domain Selection

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Study: DEMO_CF1234_NL_20250120          Progress: 2/8 domains mapped   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Discovered Domains:                                    Mapping Status  │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │                                                                     ││
│  │  ✓ DM   Demographics           dm.csv          12 cols   Complete  ││
│  │  ✓ AE   Adverse Events         ae.csv          24 cols   Complete  ││
│  │  > CM   Concomitant Meds       cm.csv          18 cols   Pending   ││
│  │    LB   Laboratory Results     lb_chemistry.csv 45 cols  Pending   ││
│  │                                lb_hematology.csv                   ││
│  │    VS   Vital Signs            vs.csv          15 cols   Pending   ││
│  │    EX   Exposure               ex.csv          20 cols   Pending   ││
│  │    MH   Medical History        mh.csv          16 cols   Pending   ││
│  │    DS   Disposition            ds.csv           8 cols   Pending   ││
│  │                                                                     ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                                                                         │
│  Domain Preview (CM - Concomitant Medications):                         │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │  Class: Interventions    Structure: One record per conc. med        ││
│  │  Variables: 41 defined   Source rows: 342                           ││
│  │  Confidence: 14 high • 3 medium • 1 low • 0 unmapped               ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  [↑↓] Navigate  [Enter] Review Domain  [S] Summary  [G] Generate  [Q]  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Screen 3: Mapping Review (Main Screen)

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Domain: CM (Concomitant Medications)        Progress: 14/18 confirmed  │
├─────────────────────────────────────────────────────────────────────────┤
│ Source Columns                 │ Mapping Suggestion                     │
│ (18 columns)                   │                                        │
│ ┌─────────────────────────────┐│┌─────────────────────────────────────┐│
│ │ Filter: [____________] [A]ll││                                      ││
│ │                             │││ Source: MEDICATION_NAME              ││
│ │ ✓ STUDYID      ────▶ STUDYID│││ Label:  "Name of medication taken"  ││
│ │ ✓ SUBJID       ────▶ USUBJID│││ Sample: "ASPIRIN", "METFORMIN"...   ││
│ │ ✓ MEDICATION_N ────▶ CMTRT  │││                                      ││
│ │ > DOSE_AMOUNT  ────▶ CMDOSE │││──────────────────────────────────────││
│ │ ? START_DATE   ────▶ ?      │││                                      ││
│ │ ? ROUTE        ────▶ ?      │││ Suggested: CMTRT (Topic Variable)    ││
│ │ ○ COMMENTS     ──▶ SUPP     │││ Confidence: ████████████░░ 94%       ││
│ │                             │││ Level: HIGH                          ││
│ │                             │││                                      ││
│ │                             │││ Target: CMTRT                        ││
│ │                             │││ Label:  "Reported Name of Drug"      ││
│ │                             │││ Type:   Char    Core: Req            ││
│ │                             │││ Role:   Topic                        ││
│ │                             │││ CT:     None (free text)             ││
│ │                             │││                                      ││
│ │                             │││──────────────────────────────────────││
│ │                             │││ Reasoning:                           ││
│ │                             │││ • Name similarity: 87%               ││
│ │                             │││ • Label match: "medication" + "name" ││
│ │                             │││ • Data type match: Char → Char       ││
│ │                             │││                                      ││
│ └─────────────────────────────┘│└─────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────────────┤
│  Legend: ✓ Confirmed  > Selected  ? Needs Review  ○ SUPP  ✗ Skipped    │
├─────────────────────────────────────────────────────────────────────────┤
│  [↑↓] Navigate  [Enter] Confirm  [Tab] Alternatives  [U] Supp  [Esc]   │
└─────────────────────────────────────────────────────────────────────────┘
```

### Screen 3b: Mapping Alternatives (When Tab Pressed)

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Domain: CM (Concomitant Medications)        Progress: 14/18 confirmed  │
├─────────────────────────────────────────────────────────────────────────┤
│ Source Columns                 │ Alternative Mappings for DOSE_AMOUNT   │
│ (18 columns)                   │                                        │
│ ┌─────────────────────────────┐│┌─────────────────────────────────────┐│
│ │ Filter: [____________] [A]ll│││ Source: DOSE_AMOUNT                  ││
│ │                             │││ Label:  "Numeric dose value"         ││
│ │ ✓ STUDYID      ────▶ STUDYID│││ Sample: 100, 250, 500, 50, 200      ││
│ │ ✓ SUBJID       ────▶ USUBJID│││ Type:   Numeric                      ││
│ │ ✓ MEDICATION_N ────▶ CMTRT  │││                                      ││
│ │ > DOSE_AMOUNT  ────▶ ?      │││──────────────────────────────────────││
│ │ ? START_DATE   ────▶ ?      │││                                      ││
│ │ ? ROUTE        ────▶ ?      │││ Ranked Alternatives:                 ││
│ │ ○ COMMENTS     ──▶ SUPP     │││                                      ││
│ │                             │││ > 1. CMDOSE   ██████████░░ 92%       ││
│ │                             │││     "Dose"    Num  Exp               ││
│ │                             │││                                      ││
│ │                             │││   2. CMDOSTOT █████████░░░ 78%       ││
│ │                             │││     "Total Daily Dose" Num Perm      ││
│ │                             │││                                      ││
│ │                             │││   3. (Map to SUPP)                   ││
│ │                             │││                                      ││
│ │                             │││──────────────────────────────────────││
│ │                             │││ Selected: CMDOSE                     ││
│ │                             │││ Label: "Dose"                        ││
│ │                             │││ Description: Amount of CMTRT         ││
│ │                             │││   administered. Not populated when   ││
│ │                             │││   CMDOSTXT is populated.             ││
│ └─────────────────────────────┘│└─────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────────────┤
│  [↑↓] Select Alternative  [Enter] Confirm  [Tab] Back  [U] Supp  [Esc] │
└─────────────────────────────────────────────────────────────────────────┘
```

### Screen 4: SUPP Decision

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Map to Supplemental Qualifier (SUPPCM)                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Source Column: COMMENTS                                                │
│  Label: "Additional comments about medication"                          │
│  Sample Values: "Patient reported nausea", "Taken with food"...        │
│                                                                         │
│  This column cannot be mapped to a standard SDTM variable.              │
│  It will be stored as a Supplemental Qualifier (SUPPQUAL).              │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │  RDOMAIN:  CM                        (Parent domain)                ││
│  │  IDVAR:    CMSEQ                     (Record identifier)            ││
│  │                                                                     ││
│  │  QNAM:     [CMCOMM__________]        (Max 8 chars)                  ││
│  │            Suggested: CMCOMM                                        ││
│  │                                                                     ││
│  │  QLABEL:   [Comments about medication_______________________]       ││
│  │            Suggested: "Additional comments about medication"        ││
│  │                                                                     ││
│  │  QORIG:    CRF                       (Data origin)                  ││
│  │  QEVAL:    [______________]          (Evaluator, optional)          ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                                                                         │
│  Preview (first 3 records):                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │  USUBJID         CMSEQ  QNAM    QVAL                                ││
│  │  DEMO-001        1      CMCOMM  "Patient reported nausea"           ││
│  │  DEMO-001        2      CMCOMM  "Taken with food"                   ││
│  │  DEMO-002        1      CMCOMM  "Self-administered"                 ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  [Tab] Next Field  [Enter] Confirm SUPP  [Esc] Cancel  [?] Help        │
└─────────────────────────────────────────────────────────────────────────┘
```

### Screen 5: Summary Before Output

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Mapping Summary - Ready to Generate                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Study: DEMO_CF1234_NL_20250120                                         │
│  Domains: 8 mapped • 0 skipped                                          │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │ Domain   Records   Mapped   SUPP   Skipped   Validation            ││
│  │ ──────────────────────────────────────────────────────────────────  ││
│  │ DM       50        12       0      0         ✓ Pass                ││
│  │ AE       127       22       2      0         ⚠ 2 warnings          ││
│  │ CM       342       16       2      0         ✓ Pass                ││
│  │ LB       2,456     42       3      0         ✓ Pass                ││
│  │ VS       850       14       1      0         ✓ Pass                ││
│  │ EX       200       18       2      0         ✓ Pass                ││
│  │ MH       89        15       1      0         ✓ Pass                ││
│  │ DS       50        7        1      0         ✓ Pass                ││
│  │ ──────────────────────────────────────────────────────────────────  ││
│  │ SUPP*    --        --       --     --        ✓ Pass                ││
│  │ TOTAL    4,164     146      12     0                               ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                                                                         │
│  Validation Warnings:                                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │ • AE.AESER: 3 values not in CT codelist (extensible)               ││
│  │ • AE.AEREL: 5 values not in CT codelist (extensible)               ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                                                                         │
│  Output Formats: [x] XPT  [x] Dataset-XML  [x] Define-XML  [ ] SAS     │
│  Output Directory: ./output/                                            │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  [G] Generate Outputs  [E] Edit Mappings  [V] View Warnings  [Esc]     │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Mapping Confidence System

### Confidence Scoring Algorithm

The existing `sdtm-map` crate provides confidence scoring. The TUI will display this clearly:

```rust
/// Confidence thresholds for TUI display
pub struct TuiConfidenceThresholds {
    /// Auto-accept threshold (show for confirmation)
    pub auto_accept: f32,      // 0.95 - Very high confidence
    
    /// High confidence (green, recommended)
    pub high: f32,             // 0.85 - Good match
    
    /// Medium confidence (yellow, alternatives shown)
    pub medium: f32,           // 0.70 - Review recommended
    
    /// Low confidence (red, manual selection needed)
    pub low: f32,              // 0.50 - Weak match
    
    /// Minimum to show as option
    pub minimum: f32,          // 0.40 - Below this, not shown
}

impl Default for TuiConfidenceThresholds {
    fn default() -> Self {
        Self {
            auto_accept: 0.95,
            high: 0.85,
            medium: 0.70,
            low: 0.50,
            minimum: 0.40,
        }
    }
}
```

### Visual Confidence Indicators

```
Auto-Accept (≥95%):
████████████████████ 98%  ✓ STUDYID → STUDYID
[Shown in green, checkmark indicates auto-suggested]

High (85-95%):
█████████████████░░░ 89%    MEDICATION → CMTRT
[Shown in bright green, no auto-checkmark]

Medium (70-85%):
█████████████░░░░░░░ 76%    DOSE → CMDOSE
[Shown in yellow/amber, alternatives emphasized]

Low (50-70%):
████████░░░░░░░░░░░░ 58%  ? START_DT → CMSTDTC
[Shown in red/orange, question mark indicates uncertainty]

Below threshold (<50%):
[Not shown as primary suggestion, only in alternatives if any]
```

### Reasoning Display

For each mapping suggestion, show why it was suggested:

```
Reasoning for MEDICATION_NAME → CMTRT (94%):

✓ Name similarity:     87% (Jaro-Winkler)
✓ Label match:         "medication" + "name" found in both
✓ Synonym match:       "MEDICATION" is known synonym for CMTRT
✓ Data type:           Char → Char (compatible)
✓ Sample values:       Drug names match expected format

Potential issues:
⚠ Length:              Some values exceed 200 chars (CMTRT max)
```

---

## SUPP Domain Fallback Workflow

### When to Suggest SUPP

A column should be offered for SUPP mapping when:

1. **No match found** - Confidence below minimum threshold (0.40)
2. **Non-standard column** - Not in any SDTM domain's variable list
3. **User choice** - User explicitly selects "Map to SUPP" option

### SUPP Decision Process

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         SUPP Decision Workflow                          │
└─────────────────────────────────────────────────────────────────────────┘

1. IDENTIFY SUPP CANDIDATES
   ├── Columns with no mapping (confidence < 0.40)
   ├── Columns not matching any standard variable
   └── Columns user explicitly marks for SUPP

2. GENERATE SUPP METADATA
   ├── RDOMAIN: Parent domain code (e.g., "CM", "AE")
   ├── IDVAR: Sequence variable (e.g., "CMSEQ", "AESEQ")  
   ├── QNAM: Derive from column name (max 8 chars)
   │   └── Algorithm: Prefix + first chars + suffix
   │       Example: "COMMENTS" → "CMCOMM" (CM + COMM)
   ├── QLABEL: From column label or name
   ├── QORIG: Default to "CRF" (can be changed)
   └── QEVAL: Optional evaluator

3. USER REVIEW
   ├── Show suggested QNAM, QLABEL
   ├── Allow editing
   ├── Show preview of SUPPQUAL records
   └── Validate QNAM uniqueness

4. CONFIRMATION
   ├── Add to SUPPQUAL mapping list
   ├── Mark column as "SUPP" in mapping state
   └── Include in output generation
```

### QNAM Generation Algorithm

```rust
/// Generate SUPP variable name (QNAM) from source column
pub fn generate_qnam(
    column_name: &str,
    domain_code: &str,
    existing_qnams: &HashSet<String>,
) -> String {
    let prefix = domain_code.to_uppercase();
    let max_suffix_len = 8 - prefix.len();
    
    // Clean column name: remove non-alphanumeric, uppercase
    let cleaned: String = column_name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase();
    
    // Take first N characters
    let suffix: String = cleaned.chars().take(max_suffix_len).collect();
    let base_qnam = format!("{}{}", prefix, suffix);
    
    // Ensure uniqueness
    let mut qnam = base_qnam.clone();
    let mut counter = 1;
    while existing_qnams.contains(&qnam) {
        let num_str = counter.to_string();
        let available = 8 - prefix.len() - num_str.len();
        let suffix: String = cleaned.chars().take(available).collect();
        qnam = format!("{}{}{}", prefix, suffix, num_str);
        counter += 1;
    }
    
    qnam
}
```

### SUPP Review Screen State

```rust
/// State for SUPP decision screen
pub struct SuppDecisionState {
    /// Source column being mapped to SUPP
    pub source_column: SourceColumn,
    
    /// Parent domain code
    pub domain_code: String,
    
    /// Generated/edited QNAM
    pub qnam: String,
    
    /// Generated/edited QLABEL
    pub qlabel: String,
    
    /// Data origin (default: "CRF")
    pub qorig: String,
    
    /// Optional evaluator
    pub qeval: Option<String>,
    
    /// Currently focused field
    pub focused_field: SuppField,
    
    /// Validation errors
    pub validation_errors: Vec<String>,
    
    /// Preview records (first 5)
    pub preview_records: Vec<SuppPreviewRecord>,
}

pub enum SuppField {
    Qnam,
    Qlabel,
    Qorig,
    Qeval,
}

pub struct SuppPreviewRecord {
    pub usubjid: String,
    pub seq: i64,
    pub qnam: String,
    pub qval: String,
}
```

---

## Technical Implementation Roadmap

### Phase 1: Foundation (Week 1-2)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Phase 1: Foundation                              │
└─────────────────────────────────────────────────────────────────────────┘

Tasks:
□ Create sdtm-tui crate with basic structure
□ Set up ratatui + crossterm dependencies
□ Implement basic terminal setup/teardown
□ Create AppState structure
□ Implement event loop (keyboard, resize, quit)
□ Create basic screen navigation system
□ Implement Welcome screen (static)

Deliverable: TUI that launches, shows welcome screen, handles quit
```

### Phase 2: Data Loading (Week 2-3)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Phase 2: Data Loading                            │
└─────────────────────────────────────────────────────────────────────────┘

Tasks:
□ Implement standards loading on startup
□ Add file browser widget for study selection
□ Implement domain discovery (reuse sdtm-ingest)
□ Create SourceColumn extraction with metadata
□ Display domain list with file info
□ Add loading indicators/progress

Deliverable: TUI that loads standards, selects study, discovers domains
```

### Phase 3: Mapping Engine Integration (Week 3-4)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                   Phase 3: Mapping Engine Integration                   │
└─────────────────────────────────────────────────────────────────────────┘

Tasks:
□ Integrate sdtm-map MappingEngine
□ Compute suggestions for all columns on domain load
□ Create RankedSuggestion with reasoning
□ Implement confidence thresholds for TUI
□ Create MappingDecision state machine
□ Store mapping state per domain

Deliverable: Mapping suggestions computed and stored for display
```

### Phase 4: Mapping Review UI (Week 4-6)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      Phase 4: Mapping Review UI                         │
└─────────────────────────────────────────────────────────────────────────┘

Tasks:
□ Implement column list widget (left panel)
□ Implement mapping panel widget (right panel)
□ Add confidence bar visualization
□ Implement metadata display (source + target)
□ Add keyboard navigation within list
□ Implement column filtering/search
□ Add view mode switching (all/pending/confirmed)
□ Implement alternatives view (Tab to show)
□ Add confirm mapping action (Enter)
□ Add skip column action

Deliverable: Full mapping review screen with navigation and confirmation
```

### Phase 5: SUPP Workflow (Week 6-7)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Phase 5: SUPP Workflow                           │
└─────────────────────────────────────────────────────────────────────────┘

Tasks:
□ Implement QNAM generation algorithm
□ Create SuppDecisionState
□ Implement SUPP decision screen
□ Add editable form fields (QNAM, QLABEL)
□ Add SUPP preview generation
□ Validate QNAM uniqueness
□ Integrate SUPP into MappingDecision
□ Show SUPP columns in column list

Deliverable: Complete SUPP mapping workflow
```

### Phase 6: Summary & Output (Week 7-8)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       Phase 6: Summary & Output                         │
└─────────────────────────────────────────────────────────────────────────┘

Tasks:
□ Implement summary screen
□ Display domain statistics
□ Show validation warnings/errors
□ Add output format selection
□ Integrate sdtm-report for output generation
□ Add progress display during generation
□ Show completion status
□ Handle generation errors gracefully

Deliverable: Complete flow from mapping to output generation
```

### Phase 7: Polish & Testing (Week 8-9)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      Phase 7: Polish & Testing                          │
└─────────────────────────────────────────────────────────────────────────┘

Tasks:
□ Add help screens/overlays
□ Implement session persistence (save/resume)
□ Add undo/redo for mapping decisions
□ Improve error handling and messaging
□ Add keyboard shortcut reference
□ Performance optimization (large files)
□ Integration testing
□ User acceptance testing

Deliverable: Production-ready TUI
```

### Estimated Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| 1. Foundation | 1-2 weeks | None |
| 2. Data Loading | 1 week | Phase 1 |
| 3. Mapping Engine | 1 week | Phase 2 |
| 4. Mapping Review UI | 2 weeks | Phase 3 |
| 5. SUPP Workflow | 1 week | Phase 4 |
| 6. Summary & Output | 1 week | Phase 5 |
| 7. Polish & Testing | 1-2 weeks | Phase 6 |
| **Total** | **8-10 weeks** | |

---

## Appendix: Data Structures

### Complete State Types

```rust
// ============================================================================
// Application State
// ============================================================================

/// Root application state
pub struct App {
    /// Current screen
    pub screen: Screen,
    
    /// Global state shared across screens
    pub state: AppState,
    
    /// Should the app quit?
    pub should_quit: bool,
    
    /// Error message to display (if any)
    pub error: Option<String>,
}

pub enum Screen {
    Welcome(WelcomeState),
    DomainSelection(DomainSelectionState),
    MappingReview(MappingReviewState),
    SuppDecision(SuppDecisionState),
    Summary(SummaryState),
    Output(OutputState),
}

pub struct AppState {
    pub standards: Vec<Domain>,
    pub ct_registry: TerminologyRegistry,
    pub preferences: UserPreferences,
}

pub struct UserPreferences {
    pub recent_studies: Vec<RecentStudy>,
    pub default_output_formats: Vec<OutputFormat>,
    pub confidence_thresholds: TuiConfidenceThresholds,
}

pub struct RecentStudy {
    pub path: PathBuf,
    pub study_id: String,
    pub last_opened: DateTime<Utc>,
}

// ============================================================================
// Welcome Screen State
// ============================================================================

pub struct WelcomeState {
    pub selected_index: usize,
    pub input_mode: WelcomeInputMode,
    pub path_input: String,
}

pub enum WelcomeInputMode {
    Selection,
    PathInput,
}

// ============================================================================
// Domain Selection State
// ============================================================================

pub struct DomainSelectionState {
    pub study: StudyState,
    pub selected_index: usize,
}

pub struct StudyState {
    pub study_id: String,
    pub study_folder: PathBuf,
    pub discovered_domains: Vec<DiscoveredDomain>,
}

pub struct DiscoveredDomain {
    pub code: String,
    pub label: String,
    pub files: Vec<DomainFile>,
    pub status: DomainStatus,
    pub mapping_summary: Option<MappingSummary>,
}

pub struct DomainFile {
    pub path: PathBuf,
    pub filename: String,
    pub row_count: usize,
    pub column_count: usize,
}

pub enum DomainStatus {
    Pending,
    InProgress,
    Complete,
    Skipped,
}

pub struct MappingSummary {
    pub total_columns: usize,
    pub high_confidence: usize,
    pub medium_confidence: usize,
    pub low_confidence: usize,
    pub unmapped: usize,
    pub supp: usize,
    pub skipped: usize,
}

// ============================================================================
// Mapping Review State
// ============================================================================

pub struct MappingReviewState {
    pub domain_code: String,
    pub domain: Domain,
    pub columns: Vec<ColumnMapping>,
    pub selected_index: usize,
    pub view_mode: ViewMode,
    pub filter_text: String,
    pub show_alternatives: bool,
    pub alternative_index: usize,
}

pub struct ColumnMapping {
    pub source: SourceColumn,
    pub decision: MappingDecision,
    pub suggestions: Vec<RankedSuggestion>,
}

pub struct SourceColumn {
    pub name: String,
    pub label: Option<String>,
    pub sample_values: Vec<String>,
    pub hints: ColumnHint,
}

pub enum MappingDecision {
    Pending,
    Confirmed {
        target_variable: String,
        confidence: f32,
    },
    SuppQual {
        qnam: String,
        qlabel: String,
    },
    Skipped {
        reason: Option<String>,
    },
}

pub struct RankedSuggestion {
    pub variable: Variable,
    pub confidence: f32,
    pub level: ConfidenceLevel,
    pub reasoning: Vec<String>,
    pub warnings: Vec<String>,
}

pub enum ViewMode {
    All,
    Pending,
    Confirmed,
    Supp,
    Skipped,
}

// ============================================================================
// SUPP Decision State
// ============================================================================

pub struct SuppDecisionState {
    pub source_column: SourceColumn,
    pub domain_code: String,
    pub qnam: String,
    pub qlabel: String,
    pub qorig: String,
    pub qeval: String,
    pub focused_field: SuppField,
    pub preview_records: Vec<SuppPreviewRecord>,
    pub validation_errors: Vec<String>,
}

pub enum SuppField {
    Qnam,
    Qlabel,
    Qorig,
    Qeval,
}

pub struct SuppPreviewRecord {
    pub usubjid: String,
    pub seq: i64,
    pub qval: String,
}

// ============================================================================
// Summary State
// ============================================================================

pub struct SummaryState {
    pub study_id: String,
    pub domain_summaries: Vec<DomainSummaryRow>,
    pub validation_issues: Vec<ValidationIssue>,
    pub output_formats: Vec<OutputFormat>,
    pub selected_formats: HashSet<OutputFormat>,
}

pub struct DomainSummaryRow {
    pub code: String,
    pub records: usize,
    pub mapped: usize,
    pub supp: usize,
    pub skipped: usize,
    pub validation_status: ValidationStatus,
}

pub enum ValidationStatus {
    Pass,
    Warnings(usize),
    Errors(usize),
}

// ============================================================================
// Output State
// ============================================================================

pub struct OutputState {
    pub stage: OutputStage,
    pub progress: f32,
    pub current_domain: Option<String>,
    pub completed: Vec<OutputResult>,
    pub errors: Vec<String>,
}

pub enum OutputStage {
    Preparing,
    ProcessingDomains,
    GeneratingXpt,
    GeneratingXml,
    GeneratingDefine,
    Complete,
    Failed,
}

pub struct OutputResult {
    pub domain_code: String,
    pub paths: OutputPaths,
}
```

### Keyboard Bindings

```rust
/// Global keyboard bindings
pub enum GlobalKey {
    Quit,           // 'q' or Ctrl+C
    Help,           // '?' or F1
    Back,           // Esc
}

/// Mapping review screen bindings
pub enum MappingKey {
    NavigateUp,     // ↑ or 'k'
    NavigateDown,   // ↓ or 'j'
    Confirm,        // Enter
    ShowAlternatives, // Tab
    MapToSupp,      // 'u' or 's'
    Skip,           // 'x'
    Filter,         // '/'
    ClearFilter,    // Esc (when filtering)
    ViewAll,        // 'a'
    ViewPending,    // 'p'
    ViewConfirmed,  // 'c'
}

/// SUPP decision screen bindings
pub enum SuppKey {
    NextField,      // Tab
    PrevField,      // Shift+Tab
    Confirm,        // Enter
    Cancel,         // Esc
}

/// Summary screen bindings
pub enum SummaryKey {
    ToggleFormat,   // Space
    Generate,       // 'g'
    EditMappings,   // 'e'
    ViewWarnings,   // 'v' or 'w'
}
```

---

## Next Steps

1. **Review this document** with stakeholders
2. **Validate UI/UX assumptions** with potential users
3. **Prioritize features** for MVP vs. future releases
4. **Create detailed tickets** for each phase
5. **Begin Phase 1** implementation

---

*Document created: 2025-12-29*
*Version: 1.0*
*Author: CDISC Transpiler Team*
