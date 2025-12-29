# CDISC Transpiler GUI Architecture & Workflow Design

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Why egui?](#why-egui)
3. [Current Architecture Analysis](#current-architecture-analysis)
4. [Proposed GUI Architecture](#proposed-gui-architecture)
5. [Data Flow & State Management](#data-flow--state-management)
6. [UI/UX Workflow Design](#uiux-workflow-design)
7. [Screen Layouts & Wireframes](#screen-layouts--wireframes)
8. [Component Architecture](#component-architecture)
9. [Mapping Confidence System](#mapping-confidence-system)
10. [SUPP Domain Fallback Workflow](#supp-domain-fallback-workflow)
11. [Technical Implementation Roadmap](#technical-implementation-roadmap)
12. [Appendix: Data Structures](#appendix-data-structures)

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

Transform the CLI into an **interactive GUI (Graphical User Interface)** using [`egui`](https://github.com/emilk/egui) that:

1. **Loads all metadata first** - Standards, CT, source data schema
2. **Presents mapping suggestions** - Shows confidence-scored mapping options with visual indicators
3. **Requires user confirmation** - High-confidence mappings shown for approval with single-click
4. **Offers alternatives** - Low-confidence mappings show dropdown alternatives
5. **Handles unmapped columns** - Clear workflow for SUPP domain fallback
6. **Displays rich context** - Source column description alongside SDTM variable metadata
7. **Shows Required Variables** - Visual indicators for mapping completion status
8. **Modern UX** - Drag-and-drop, tooltips, search/filter, and responsive layouts

### Data Integrity Principles

**Important**: The transpiler NEVER modifies source data or renames source columns.

| What Changes (Output) | What Does NOT Change (Source) |
|-----------------------|-------------------------------|
| Output variable names (SDTM-compliant) | Source CSV column names |
| CT-normalized VALUES in output | Original source data values |
| Output file format (XPT, XML) | Source CSV file structure |

**Mapping ≠ Renaming**: When we "map" a source column to an SDTM variable, we are:
- **Directing** which source column's data flows to which output variable
- **NOT** renaming the source column
- **NOT** modifying the source file

**CT Normalization** (per SDTMIG v3.4 Section 4.3): Only applies to OUTPUT values:
- Source value "Male" stays as "Male" in source CSV
- Output SDTM variable gets CT-normalized value "M" (via codelist C66731 lookup)
- The GUI shows this transformation: `Source: "Male" → Output: "M"`
- Non-extensible codelists: Values MUST match CT exactly (error if not found)
- Extensible codelists: Non-CT values allowed as sponsor extensions (warning only)

### Key Benefits

| Aspect | Current CLI | Proposed GUI |
|--------|-------------|--------------|
| **Mapping Accuracy** | ~70-80% automated | 100% user-verified |
| **Error Handling** | Silent failures or warnings | Interactive resolution with dialogs |
| **SUPP Decisions** | Automatic (may be wrong) | User-guided with visual context |
| **User Confidence** | Low (black box) | High (transparent, visual) |
| **Learning Curve** | Steep (CLI flags) | Intuitive (point-and-click) |
| **Required Variables** | Not visible | Visual indicators with progress bars |
| **CT Compliance** | Silent normalization | Visual CT mapping review |
| **Data Preview** | None | Live data tables with sample values |
| **Accessibility** | Terminal only | Modern desktop application |

---

## Why egui?

[egui](https://github.com/emilk/egui) is chosen as the GUI framework for the following reasons:

### Advantages

| Feature | Benefit |
|---------|---------|
| **Pure Rust** | No external dependencies, integrates seamlessly with existing crates |
| **Immediate Mode** | Simple state management, no complex widget hierarchies |
| **Cross-Platform** | Windows, macOS, Linux support out of the box |
| **Native Performance** | Fast rendering with GPU acceleration via eframe |
| **Rich Widgets** | Tables, trees, plots, drag-and-drop built-in |
| **Theming** | Dark/light themes, customizable styling |
| **WebAssembly** | Future option to deploy as web application |
| **Active Community** | Well-maintained, extensive documentation |

### egui vs Alternatives

| Framework | Pros | Cons |
|-----------|------|------|
| **egui** ✓ | Pure Rust, simple, immediate mode | Less "native" look |
| iced | Elm-like, native look | Steeper learning curve |
| tauri | Web UI, native wrapper | Requires web stack |
| gtk-rs | Native GTK widgets | Complex, heavy dependency |
| druid | Native, data-driven | Development slowed |

### Immediate Mode GUI Paradigm

egui uses **immediate mode** GUI, which means:

```rust
// Every frame, you describe what the UI should look like
fn update(&mut self, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("SDTM Mapping");
        
        // If button is clicked, handle immediately
        if ui.button("Accept Mapping").clicked() {
            self.accept_current_mapping();
        }
        
        // Conditional rendering based on state
        if self.show_details {
            ui.label(format!("Confidence: {:.0}%", self.confidence * 100.0));
        }
    });
}
```

This simplifies state management compared to retained-mode GUIs.

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

The `standards/` directory contains rich metadata that should be leveraged in the GUI:

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

### Study Metadata Files (Items.csv & CodeLists.csv)

**Important**: Each study folder contains metadata files that provide rich context
about source columns. This information is essential for accurate mapping.

Study folders (e.g., `mockdata/DEMO_GDISC_*`) contain:

#### Items.csv - Source Column Definitions

Provides metadata about EVERY column in the source CSVs:

| Column | Description | Example |
|--------|-------------|---------|
| `ID` | Source column name | `SEX`, `CMTRT`, `AETERM` |
| `Label` | Human-readable description | `"Gender"`, `"Medication"` |
| `Data Type` | Data type | `text`, `integer`, `double`, `date` |
| `Mandatory` | Is value required? | `True`, `False` |
| `Format Name` | Link to CodeLists.csv | `SEX`, `ROUTE`, `YESNO` |

**Example:**
```csv
"ID","Label","Data Type","Mandatory","Format Name"
"SEX","Gender","text","True","SEX"
"CMTRT","Medication","text","True",""
"CMROUTE","Route","text","True","ROUTE"
```

#### CodeLists.csv - Study-Specific Value Sets

Provides allowed values for coded columns (links to `Format Name` in Items.csv):

| Column | Description | Example |
|--------|-------------|---------|
| `Format Name` | Links to Items.csv | `SEX`, `ROUTE` |
| `Code Value` | The actual code | `M`, `F`, `ORAL` |
| `Code Text` | Display text | `Male`, `Female`, `Oral` |

**Example:**
```csv
"Format Name","Data Type","Code Value","Code Text"
"SEX","text","F","Female"
"SEX","text","M","Male"
"ROUTE","text","ORAL","Oral"
"ROUTE","text","NASAL","Nasal"
```

#### How GUI Uses Study Metadata

The GUI loads Items.csv and CodeLists.csv to provide:

1. **Rich Source Column Context**: Display the Label from Items.csv so users understand what each column represents
2. **Value Preview**: Show sample values AND their meanings from CodeLists in data tables
3. **Smarter Mapping Suggestions**: Use Labels for better fuzzy matching
4. **CT Comparison**: Visual comparison of study CodeLists values against CDISC CT values

### CT Relationships (SDTM_CT_relationships.md)

The `SDTM_CT_relationships.md` file documents how SDTM variables link to CT codelists:

```
SDTM Variable → CT Codelist Code → CT Terms → Allowed Values

Example: DM.SEX
  ├── CDISC CT Codelist Code: C66731
  ├── Codelist Name: "Sex"
  ├── Extensible: No (CLOSED list)
  └── Terms: F, M, U, INTERSEX
```

**CT Validation Rules:**

| Extensible | Source Value | Action |
|------------|--------------|--------|
| No (Closed) | In CT | ✓ Pass |
| No (Closed) | NOT in CT | ✗ ERROR |
| Yes (Open) | In CT | ✓ Pass + normalize |
| Yes (Open) | NOT in CT | ⚠ WARNING (sponsor extension) |

### Required Variables (Core Designations)

Per SDTMIG v3.4, variables have Core designations that determine if they must be mapped:

| Core | Meaning | GUI Behavior |
|------|---------|--------------|
| **Req** | Required - must be present, cannot be null | Must map or auto-generate |
| **Exp** | Expected - should be present when applicable | Warning if unmapped |
| **Perm** | Permissible - optional | No warning if unmapped |

**Variables that don't need source columns** (auto-generated):
- `DOMAIN` - Auto-filled with domain code ("CM", "AE", etc.)
- `--SEQ` - Auto-incremented sequence number
- `STUDYID` - From study configuration

---

## Proposed GUI Architecture

### New Crate Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              sdtm-gui (NEW)                             │
│                     (Graphical User Interface Layer)                    │
│  • Window management (eframe/egui)                                      │
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
                        │   (+ GUI State Types) │
                        │   MappingDecision     │
                        │   ColumnReviewState   │
                        │   SuppDecision        │
                        └───────────────────────┘
```

### GUI Component Architecture

```
sdtm-gui/
├── Cargo.toml
└── src/
    ├── main.rs                   # Entry point with eframe
    ├── app.rs                    # Main App struct implementing eframe::App
    ├── state/
    │   ├── mod.rs
    │   ├── app_state.rs          # Global application state
    │   ├── mapping_state.rs      # Mapping review state
    │   ├── domain_state.rs       # Domain-specific state
    │   └── ui_state.rs           # UI-specific state (selected panels, etc.)
    ├── views/
    │   ├── mod.rs
    │   ├── welcome.rs            # Study selection view
    │   ├── domain_select.rs      # Domain selection view
    │   ├── mapping_review.rs     # Main mapping review view
    │   ├── variable_detail.rs    # SDTM variable detail panel
    │   ├── source_detail.rs      # Source column detail panel
    │   ├── ct_validation.rs      # CT value validation view
    │   ├── supp_decision.rs      # SUPP domain fallback dialog
    │   ├── summary.rs            # Final review before output
    │   └── output_progress.rs    # Output generation progress
    ├── widgets/
    │   ├── mod.rs
    │   ├── variable_table.rs     # SDTM variables table with sorting
    │   ├── source_table.rs       # Source columns table
    │   ├── mapping_row.rs        # Single mapping row widget
    │   ├── confidence_badge.rs   # Visual confidence indicator
    │   ├── progress_indicator.rs # Overall mapping progress
    │   ├── data_preview.rs       # Sample data preview table
    │   └── ct_comparison.rs      # CT value comparison widget
    ├── dialogs/
    │   ├── mod.rs
    │   ├── file_picker.rs        # Native file/folder picker
    │   ├── confirmation.rs       # Confirmation dialogs
    │   ├── error_dialog.rs       # Error display dialogs
    │   └── about.rs              # About dialog
    └── theme.rs                  # Custom styling and colors
```

### Dependencies for GUI Crate

```toml
[dependencies]
# GUI Framework
eframe = "0.29"                   # egui framework with native window
egui = "0.29"                     # Immediate mode GUI
egui_extras = "0.29"              # Extra widgets (tables, etc.)

# File dialogs
rfd = "0.15"                      # Native file dialogs

# Async runtime
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }

# Error handling
anyhow = "1.0"

# Serialization (for saving/loading mappings)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Internal crates
sdtm-model = { path = "../sdtm-model" }
sdtm-core = { path = "../sdtm-core" }
sdtm-map = { path = "../sdtm-map" }
sdtm-standards = { path = "../sdtm-standards" }
sdtm-ingest = { path = "../sdtm-ingest" }
sdtm-validate = { path = "../sdtm-validate" }
sdtm-report = { path = "../sdtm-report" }
```

### Main Application Structure

```rust
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1000.0, 600.0])
            .with_title("CDISC Transpiler"),
        ..Default::default()
    };
    
    eframe::run_native(
        "CDISC Transpiler",
        options,
        Box::new(|cc| Ok(Box::new(CdiscTranspilerApp::new(cc)))),
    )
}

pub struct CdiscTranspilerApp {
    state: AppState,
    current_view: View,
}

impl eframe::App for CdiscTranspilerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        self.render_menu_bar(ctx);
        
        // Main content based on current view
        match &self.current_view {
            View::Welcome => self.render_welcome(ctx),
            View::DomainSelect => self.render_domain_select(ctx),
            View::MappingReview => self.render_mapping_review(ctx),
            View::Summary => self.render_summary(ctx),
            View::Output => self.render_output(ctx),
        }
        
        // Status bar at bottom
        self.render_status_bar(ctx);
    }
}
```

---

## Data Flow & State Management

### Application State Machine

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         GUI Application States                          │
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

### GUI Design Principles

Unlike terminal-based CLI tools, the desktop GUI offers:
- **Resizable windows and panels** - Users can adjust layout to their needs
- **Drag-and-drop** - Intuitive mapping operations
- **Tooltips and hover states** - Contextual help without cluttering the interface
- **Modal dialogs** - Focused decisions without losing context
- **Data tables** - Sortable, filterable, scrollable tables for large datasets
- **Rich formatting** - Colors, icons, badges for visual feedback

### Screen 1: Welcome Screen

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ CDISC Transpiler                                              [─] [□] [×]       │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│                    ┌───────────────────────────────────────┐                    │
│                    │     🏥 CDISC SDTM Transpiler          │                    │
│                    │           Version 0.1.0               │                    │
│                    └───────────────────────────────────────┘                    │
│                                                                                 │
│     Convert clinical trial data to CDISC SDTM format with guided mapping.      │
│                                                                                 │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │  📂 Recent Studies                                                      │   │
│   ├─────────────────────────────────────────────────────────────────────────┤   │
│   │  📁 DEMO_CF1234_NL_20250120                      2025-01-20 10:48 AM    │   │
│   │  📁 DEMO_GDISC_20240903                          2024-09-03 07:29 AM    │   │
│   │  📁 TRIAL_XYZ_2024                               2024-08-15 02:45 PM    │   │
│   └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│   ┌─────────────────────────────┐   ┌─────────────────────────────┐            │
│   │   📂 Open Study Folder...   │   │   📋 Paste Folder Path...   │            │
│   └─────────────────────────────┘   └─────────────────────────────┘            │
│                                                                                 │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │  ℹ️ Standards: SDTMIG v3.4 • CT 2024-03-29 • 52 domains loaded          │   │
│   └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│  Ready                                                      Light ◉ | Dark ○   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

**egui Implementation:**
```rust
fn render_welcome(&mut self, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.heading("🏥 CDISC SDTM Transpiler");
            ui.label("Version 0.1.0");
            ui.add_space(20.0);
            ui.label("Convert clinical trial data to CDISC SDTM format with guided mapping.");
            ui.add_space(30.0);
            
            // Recent studies list
            ui.group(|ui| {
                ui.heading("📂 Recent Studies");
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for study in &self.state.recent_studies {
                        if ui.selectable_label(false, &study.name).clicked() {
                            self.open_study(&study.path);
                        }
                    }
                });
            });
            
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                if ui.button("📂 Open Study Folder...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.open_study(&path);
                    }
                }
                if ui.button("📋 Paste Folder Path...").clicked() {
                    self.show_paste_dialog = true;
                }
            });
        });
    });
}
```

### Screen 2: Domain Selection

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ CDISC Transpiler - DEMO_CF1234_NL_20250120                    [─] [□] [×]       │
├──────────────────────────────┬──────────────────────────────────────────────────┤
│ File  Edit  View  Help       │  Progress: ████████████░░░░░░░░ 2/8 domains      │
├──────────────────────────────┴──────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─ Domains ─────────────────────────────────────────────────────────────────┐  │
│  │                                                                           │  │
│  │  ┌────┬──────────────────┬──────────────────┬─────────┬────────────────┐  │  │
│  │  │    │ Domain           │ Source Files     │ Columns │ Status         │  │  │
│  │  ├────┼──────────────────┼──────────────────┼─────────┼────────────────┤  │  │
│  │  │ ✅ │ DM Demographics  │ dm.csv           │ 12      │ ✓ Complete     │  │  │
│  │  │ ✅ │ AE Adverse Events│ ae.csv           │ 24      │ ✓ Complete     │  │  │
│  │  │ ⏳ │ CM Concom. Meds  │ cm.csv           │ 18      │ ○ Pending      │  │  │
│  │  │ ⏳ │ LB Laboratory    │ lb_chem.csv,     │ 45      │ ○ Pending      │  │  │
│  │  │    │                  │ lb_hemat.csv     │         │                │  │  │
│  │  │ ⏳ │ VS Vital Signs   │ vs.csv           │ 15      │ ○ Pending      │  │  │
│  │  │ ⏳ │ EX Exposure      │ ex.csv           │ 20      │ ○ Pending      │  │  │
│  │  │ ⏳ │ MH Medical Hist. │ mh.csv           │ 16      │ ○ Pending      │  │  │
│  │  │ ⏳ │ DS Disposition   │ ds.csv           │ 8       │ ○ Pending      │  │  │
│  │  └────┴──────────────────┴──────────────────┴─────────┴────────────────┘  │  │
│  │                                                                           │  │
│  │  [🔄 Refresh]  [📝 Review Selected]  [⚡ Auto-Map All]                     │  │
│  └───────────────────────────────────────────────────────────────────────────┘  │
│                                                                                 │
│  ┌─ Domain Preview ──────────────────────────────────────────────────────────┐  │
│  │  CM - Concomitant Medications                                             │  │
│  │  ─────────────────────────────────────────────────────────────────────    │  │
│  │  Class: Interventions          Structure: One record per concom. med      │  │
│  │  Variables: Required 5 • Expected 8 • Permissible 12                      │  │
│  │                                                                           │  │
│  │  Mapping Confidence:  🟢 14 High  🟡 3 Medium  🔴 1 Low  ⚪ 0 Unmapped     │  │
│  │  Metadata: Items.csv ✓  CodeLists.csv ✓                                   │  │
│  └───────────────────────────────────────────────────────────────────────────┘  │
│                                                                                 │
│  ┌─────────────────────────────┐   ┌─────────────────────────────┐             │
│  │   📊 View Summary Report    │   │   💾 Generate SDTM Output   │             │
│  └─────────────────────────────┘   └─────────────────────────────┘             │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│  Study: DEMO_CF1234_NL_20250120 │ 8 domains │ 158 source columns               │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Screen 3: Main Mapping Review

This is the core screen where users review and confirm mappings. The SDTM-first design
shows SDTM variables as the primary navigation, with source columns mapped TO them.

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ CDISC Transpiler - CM (Concomitant Medications)                           [─] [□] [×]       │
├────────────────────────────────────────────────────────────────────────────────────────────-┤
│ File  Edit  View  Help                                    Progress: ███████████░░░ 12/25   │
├────────────────────────────────┬────────────────────────────────────────────────────────────┤
│                                │                                                            │
│  ┌─ SDTM Variables ──────────┐ │  ┌─ Selected Variable Details ─────────────────────────┐  │
│  │                           │ │  │                                                     │  │
│  │  🔍 Filter: [___________] │ │  │  CMROUTE                                            │  │
│  │                           │ │  │  ─────────────────────────────────────────────────  │  │
│  │  ── Required ───────────  │ │  │  Label: Route of Administration                     │  │
│  │  ✅ STUDYID    [AUTO]     │ │  │  Type: Char (200)    Role: Record Qualifier         │  │
│  │  ✅ DOMAIN     [AUTO]     │ │  │  Core: Permissible                                  │  │
│  │  ✅ USUBJID    SUBJID     │ │  │                                                     │  │
│  │  ✅ CMSEQ      [AUTO]     │ │  │  📚 CDISC CT: C66729 - Route of Administration      │  │
│  │  ⚠️ CMTRT      [NEED]     │ │  │     Extensible: Yes (142 terms)                     │  │
│  │                           │ │  │     Examples: ORAL, NASAL, TOPICAL, INTRAVENOUS     │  │
│  │  ── Expected ───────────  │ │  │                                                     │  │
│  │  ✅ CMDOSE     DOSE_AMT   │ │  │  📖 Description:                                    │  │
│  │  🔶 CMDOSU     DOSE_UNIT  │ │  │     Route of administration for the intervention.  │  │
│  │  ✅ CMROUTE    CMROUTE    │ │  │                                                     │  │
│  │  ⚪ CMDOSFRQ   --         │ │  └─────────────────────────────────────────────────────┘  │
│  │  ✅ CMSTDTC    START_DT   │ │                                                            │
│  │  ⚪ CMENDTC    --         │ │  ┌─ Mapped Source Column ──────────────────────────────┐  │
│  │                           │ │  │                                                     │  │
│  │  ── Permissible ────────  │ │  │  Source: CMROUTE                                    │  │
│  │  ⚪ CMCAT      --         │ │  │  Confidence: ████████████████░░░░ 94% 🟢 HIGH       │  │
│  │  ⚪ CMSCAT     --         │ │  │                                                     │  │
│  │  ...                      │ │  │  ── From Items.csv ─────────────────────────────    │  │
│  │                           │ │  │  Label: "Route"                                     │  │
│  │                           │ │  │  Data Type: text    Mandatory: Yes                  │  │
│  │                           │ │  │  Format: ROUTE (has CodeList)                       │  │
│  └───────────────────────────┘ │  │                                                     │  │
│                                │  │  ── Study CodeList Values ──────────────────────    │  │
│  Legend:                       │  │  ORAL → "Oral"                                      │  │
│  ✅ Mapped  🔶 Review          │  │  NASAL → "Nasal"                                    │  │
│  ⚠️ Required  ⚪ Unmapped      │  │  TOPICAL → "Topical"                                │  │
│  [AUTO] = System Generated     │  │                                                     │  │
│                                │  │  ── CT Validation ──────────────────────────────    │  │
│                                │  │  ✅ ORAL → ORAL (exact match)                       │  │
│                                │  │  ✅ NASAL → NASAL (exact match)                     │  │
│                                │  │  ✅ TOPICAL → TOPICAL (exact match)                 │  │
│                                │  │                                                     │  │
│                                │  │  ✅ All 3 source values are valid CT terms          │  │
│                                │  │                                                     │  │
│                                │  │  ┌────────────────┐  ┌────────────────────────┐     │  │
│                                │  │  │ ✓ Accept       │  │ 🔄 Change Source...    │     │  │
│                                │  │  └────────────────┘  └────────────────────────┘     │  │
│                                │  └─────────────────────────────────────────────────────┘  │
│                                │                                                            │
├────────────────────────────────┴────────────────────────────────────────────────────────────┤
│  ◀ Back to Domains │ Required: 4/5 ✅  Expected: 6/8  Permissible: 2/12 │ [💾 Save Mapping] │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Screen 3a: Source Column Selection Dialog

When user clicks "Change Source..." or needs to map an unmapped variable:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Select Source Column for CMTRT                                        [×]       │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  🔍 Search: [________________________]                                          │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                                                                         │    │
│  │   Confidence  │ Source Column │ Label (Items.csv) │ Sample Values       │    │
│  │  ─────────────┼───────────────┼───────────────────┼─────────────────────│    │
│  │  🟢 98%       │ CMTRT         │ "Medication"      │ ASPIRIN, METFORMIN  │    │
│  │  🟢 89%       │ MEDICATION    │ "Drug Name"       │ Aspirin 100mg       │    │
│  │  🟡 82%       │ DRUG_NAME     │ "Name of Drug"    │ aspirin, metformin  │    │
│  │  🟡 71%       │ MED_DESC      │ "Med Description" │ Pain reliever       │    │
│  │  🔴 45%       │ COMMENTS      │ "Comments"        │ Take with food      │    │
│  │  ─────────────┼───────────────┼───────────────────┼─────────────────────│    │
│  │  ⚪ --        │ [No mapping - leave empty]                              │    │
│  │  📝 --        │ [Use constant value...]                                 │    │
│  │  📤 --        │ [Send to SUPP domain...]                                │    │
│  │                                                                         │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                 │
│  ┌─ Preview Selected: CMTRT ────────────────────────────────────────────────┐   │
│  │                                                                          │   │
│  │  Column: CMTRT     Type: text     Rows: 342                              │   │
│  │  Label: "Medication" (from Items.csv)                                    │   │
│  │                                                                          │   │
│  │  Sample Values (first 5 rows):                                           │   │
│  │  ┌─────────────────────────────────────────────────────────────────┐     │   │
│  │  │ 1. ASPIRIN                                                      │     │   │
│  │  │ 2. METFORMIN                                                    │     │   │
│  │  │ 3. LISINOPRIL                                                   │     │   │
│  │  │ 4. ATORVASTATIN                                                 │     │   │
│  │  │ 5. OMEPRAZOLE                                                   │     │   │
│  │  └─────────────────────────────────────────────────────────────────┘     │   │
│  │                                                                          │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│              ┌──────────────────────┐    ┌──────────────────────┐               │
│              │    ✓ Select          │    │      Cancel          │               │
│              └──────────────────────┘    └──────────────────────┘               │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Screen 3b: CT Value Mismatch Warning

When source values don't match CDISC CT:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ ⚠️ CT Value Mismatch - SEX                                            [×]       │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  The source column contains values that don't match CDISC Controlled            │
│  Terminology.                                                                   │
│                                                                                 │
│  CDISC CT: C66731 (Sex)                                                         │
│  Extensible: ❌ NO - Values MUST match exactly!                                 │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                                                                         │    │
│  │  Source Value   │ CT Match    │ SDTM Output │ Status                    │    │
│  │  ──────────────-┼─────────────┼─────────────┼───────────────────────    │    │
│  │  Male           │ M           │ M           │ ✅ Will normalize         │    │
│  │  Female         │ F           │ F           │ ✅ Will normalize         │    │
│  │  Unknown        │ U           │ U           │ ✅ Will normalize         │    │
│  │  Other          │ ???         │ ???         │ ❌ NOT IN CT!             │    │
│  │                                                                         │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                 │
│  ⚠️ ERROR: 1 source value ("Other") cannot be mapped to a valid CT term.        │
│                                                                                 │
│  This codelist is NOT extensible - you must:                                    │
│  • Fix the source data before proceeding, OR                                    │
│  • Manually map "Other" to a valid CT value                                     │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  Map "Other" to:  [ Select CT value...          ▼]                      │    │
│  │  Valid options: F, INTERSEX, M, U, UNDIFFERENTIATED                     │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                 │
│       ┌─────────────────────┐  ┌─────────────────────┐  ┌───────────────┐       │
│       │ 📝 Apply Value Map  │  │ ⏭️ Skip & Continue  │  │   Cancel      │       │
│       └─────────────────────┘  └─────────────────────┘  └───────────────┘       │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Screen 4: Unmapped Source Columns View

Shows source columns that haven't been mapped to any SDTM variable:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Unmapped Source Columns - CM                                          [×]       │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  These source columns are not mapped to any standard SDTM variable.             │
│  Choose what to do with each column:                                            │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                                                                         │    │
│  │  Source Column  │ Label (Items.csv)        │ Action                     │    │
│  │  ───────────────┼──────────────────────────┼────────────────────────    │    │
│  │  COMMENTS       │ "Additional comments"    │ [Send to SUPP ▼]           │    │
│  │  NOTES          │ "Internal notes"         │ [Skip - not needed ▼]      │    │
│  │  CUSTOM_FIELD   │ "Custom data field"      │ [Send to SUPP ▼]           │    │
│  │  BATCH_NUMBER   │ "Drug batch number"      │ [Assign to variable ▼]     │    │
│  │                                                                         │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                 │
│  Actions:                                                                       │
│  • Send to SUPP: Column data will be stored in SUPPCM                           │
│  • Skip: Column will not be included in SDTM output                             │
│  • Assign to variable: Map to an SDTM variable                                  │
│                                                                                 │
│  ┌─ SUPP Configuration for COMMENTS ────────────────────────────────────────┐   │
│  │                                                                          │   │
│  │  RDOMAIN: CM    IDVAR: CMSEQ                                             │   │
│  │  QNAM:   [CMCOMM______]   (max 8 chars)                                  │   │
│  │  QLABEL: [Comments about medication_________________]                    │   │
│  │  QORIG:  [ CRF              ▼]                                           │   │
│  │  QEVAL:  [__________________] (optional)                                 │   │
│  │                                                                          │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│                    ┌────────────────────┐    ┌──────────────────────┐           │
│                    │   ✓ Apply All      │    │      Cancel          │           │
│                    └────────────────────┘    └──────────────────────┘           │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Screen 5: Summary & Output Generation

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ CDISC Transpiler - Generate SDTM Output                                   [─] [□] [×]       │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│ File  Edit  View  Help                                                                      │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                             │
│  ┌─ Mapping Summary ───────────────────────────────────────────────────────────────────┐    │
│  │                                                                                     │    │
│  │  Domain │ Required │ Expected │ Permissible │ SUPP     │ Status                     │    │
│  │  ───────┼──────────┼──────────┼─────────────┼──────────┼────────────────────────    │    │
│  │  DM     │ 8/8 ✅   │ 5/5 ✅   │ 3/10        │ 0        │ ✅ Ready                   │    │
│  │  AE     │ 6/6 ✅   │ 12/15    │ 5/20        │ 2        │ ✅ Ready                   │    │
│  │  CM     │ 5/5 ✅   │ 6/8      │ 2/12        │ 3        │ ✅ Ready                   │    │
│  │  LB     │ 10/10 ✅ │ 8/12     │ 10/25       │ 1        │ ✅ Ready                   │    │
│  │  VS     │ 5/5 ✅   │ 4/6      │ 2/8         │ 0        │ ✅ Ready                   │    │
│  │  EX     │ 7/7 ✅   │ 5/8      │ 3/15        │ 0        │ ✅ Ready                   │    │
│  │  MH     │ 4/4 ✅   │ 6/8      │ 1/6         │ 1        │ ✅ Ready                   │    │
│  │  DS     │ 4/4 ✅   │ 3/4      │ 0/3         │ 0        │ ✅ Ready                   │    │
│  │                                                                                     │    │
│  └─────────────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                             │
│  ┌─ Output Options ────────────────────────────────────────────────────────────────────┐    │
│  │                                                                                     │    │
│  │  Output Formats:                              Output Location:                      │    │
│  │  ☑ XPT (SAS Transport v5)                    📂 [./output/sdtm          ] [...]    │    │
│  │  ☑ Dataset-XML                                                                     │    │
│  │  ☑ Define-XML v2.1                           Include SUPP datasets: ☑              │    │
│  │  ☐ SAS Program Files                         Generate validation report: ☑          │    │
│  │                                                                                     │    │
│  └─────────────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                             │
│  ┌─ Validation Issues ─────────────────────────────────────────────────────────────────┐    │
│  │                                                                                     │    │
│  │  ⚠️ 3 Warnings (will not block output):                                             │    │
│  │  • AE: AESEV has 2 values not in CT (extensible codelist - allowed)                 │    │
│  │  • LB: LBORRESU missing for 12 records                                              │    │
│  │  • CM: CMENDTC is null for all records                                              │    │
│  │                                                                                     │    │
│  │  ❌ 0 Errors                                                                        │    │
│  │                                                                                     │    │
│  └─────────────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                             │
│            ┌─────────────────────────────┐    ┌─────────────────────────────┐               │
│            │   🚀 Generate SDTM Output   │    │   ◀ Back to Mapping         │               │
│            └─────────────────────────────┘    └─────────────────────────────┘               │
│                                                                                             │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│  All required variables mapped ✅ │ 8 domains │ 7 SUPP variables                            │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Screen 5a: Output Progress

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Generating SDTM Output...                                             [×]       │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│                                                                                 │
│  ████████████████████████████░░░░░░░░░░░░░░░░░░░░  65%                          │
│                                                                                 │
│  Current: Processing LB domain (45 variables, 2,450 records)                    │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                                                                         │    │
│  │  ✅ DM     dm.xpt, dm.xml                     342 records     0.2s      │    │
│  │  ✅ AE     ae.xpt, ae.xml, suppae.xpt         1,205 records   0.8s      │    │
│  │  ✅ CM     cm.xpt, cm.xml, suppcm.xpt         892 records     0.5s      │    │
│  │  ⏳ LB     Processing...                      2,450 records   ...       │    │
│  │  ⏸️ VS     Waiting...                                                   │    │
│  │  ⏸️ EX     Waiting...                                                   │    │
│  │  ⏸️ MH     Waiting...                                                   │    │
│  │  ⏸️ DS     Waiting...                                                   │    │
│  │                                                                         │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                 │
│  Elapsed: 00:01:32                                                              │
│                                                                                 │
│                         ┌──────────────────────┐                                │
│                         │      ⏹️ Cancel       │                                │
│                         └──────────────────────┘                                │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Screen 5b: Output Complete

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ ✅ SDTM Output Generated Successfully!                                [×]       │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│                          ✅ Generation Complete!                                │
│                                                                                 │
│  Output Location: ./output/sdtm/DEMO_CF1234_NL_20250120                         │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                                                                         │    │
│  │  Files Generated:                                                       │    │
│  │                                                                         │    │
│  │  📦 XPT Files (8):                                                      │    │
│  │     dm.xpt, ae.xpt, cm.xpt, lb.xpt, vs.xpt, ex.xpt, mh.xpt, ds.xpt     │    │
│  │                                                                         │    │
│  │  📦 SUPP Files (3):                                                     │    │
│  │     suppae.xpt, suppcm.xpt, supplb.xpt                                  │    │
│  │                                                                         │    │
│  │  📄 Dataset-XML (8):                                                    │    │
│  │     dm.xml, ae.xml, cm.xml, lb.xml, vs.xml, ex.xml, mh.xml, ds.xml     │    │
│  │                                                                         │    │
│  │  📋 Define-XML:                                                         │    │
│  │     define.xml, define-stylesheet.xsl                                   │    │
│  │                                                                         │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                 │
│  Total Records: 5,894 across 8 domains                                          │
│  Processing Time: 00:02:45                                                      │
│                                                                                 │
│  ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────────────┐    │
│  │ 📂 Open Folder    │  │ 📊 View Report    │  │ 🏠 Back to Welcome        │    │
│  └───────────────────┘  └───────────────────┘  └───────────────────────────┘    │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Mapping Confidence System

### Confidence Scoring Algorithm

The existing `sdtm-map` crate provides confidence scoring. The GUI displays this with visual indicators:

```rust
/// Confidence thresholds for GUI display
pub struct GuiConfidenceThresholds {
    pub auto_accept: f32,  // 0.95 - Very high confidence (green badge)
    pub high: f32,         // 0.85 - Good match (green)
    pub medium: f32,       // 0.70 - Review recommended (yellow)
    pub low: f32,          // 0.50 - Weak match (orange)
    pub minimum: f32,      // 0.40 - Below this, not shown (red)
}
```

### Visual Indicators

| Confidence | Color | Badge | Behavior |
|------------|-------|-------|----------|
| ≥95% | 🟢 Green | "HIGH" | Auto-suggested, single-click accept |
| ≥85% | 🟢 Green | "GOOD" | Recommended, review suggested |
| ≥70% | 🟡 Yellow | "REVIEW" | Show alternatives dropdown |
| ≥50% | 🟠 Orange | "LOW" | Manual selection required |
| <50% | 🔴 Red | "?" | Not shown as option |

---

## SUPP Domain Fallback Workflow

### When to Use SUPP

Source columns that cannot map to standard SDTM variables are stored in Supplemental Qualifier (SUPP--) datasets per SDTMIG guidelines.

### QNAM Generation Algorithm

```rust
fn generate_qnam(domain: &str, source_column: &str) -> String {
    // 1. Try domain prefix + abbreviated column name
    let prefix = &domain[..2]; // "CM", "AE"
    let abbrev = abbreviate_column_name(source_column, 6); // Max 6 chars
    
    // 2. Ensure uniqueness
    let qnam = format!("{}{}", prefix, abbrev);
    ensure_unique(&qnam)
}
```

---

## Technical Implementation Roadmap

### Phase 1: Foundation (Week 1-2)

**Setup & Basic Structure:**
- [ ] Create `sdtm-gui` crate with eframe/egui dependencies
- [ ] Implement basic window management
- [ ] Create welcome screen with study folder selection
- [ ] Implement file picker dialog

**Deliverable:** GUI that launches, shows welcome screen, can select folders

### Phase 2: Data Loading (Week 3-4)

**Standards & Study Data:**
- [ ] Load SDTM standards from `standards/` directory
- [ ] Load CT codelists and parse relationships
- [ ] Discover and parse source CSV files
- [ ] Load Items.csv and CodeLists.csv metadata

**Deliverable:** GUI that loads all data and shows domain list

### Phase 3: Mapping Review UI (Week 5-6)

**Core Mapping Interface:**
- [ ] Implement SDTM-first variable list panel
- [ ] Implement variable detail panel
- [ ] Implement source mapping panel
- [ ] Implement confidence badges and progress bars
- [ ] Implement CT validation display

**Deliverable:** Functional mapping review screen

### Phase 4: User Interactions (Week 7-8)

**Mapping Operations:**
- [ ] Source column selection dialog
- [ ] CT value mismatch warning dialog
- [ ] Unmapped source columns view
- [ ] SUPP decision dialog
- [ ] Keyboard shortcuts and tooltips

**Deliverable:** Complete interactive mapping workflow

### Phase 5: Output Generation (Week 9-10)

**Report & Export:**
- [ ] Summary view with validation issues
- [ ] Output options selection
- [ ] Progress indicator during generation
- [ ] Success/error dialogs
- [ ] Integration with existing `sdtm-report` crate

**Deliverable:** End-to-end working GUI application

### Phase 6: Polish & Testing (Week 11-12)

**Quality & UX:**
- [ ] Dark/light theme support
- [ ] Responsive layout for different window sizes
- [ ] Error handling and user feedback
- [ ] Save/load mapping sessions
- [ ] Integration tests

**Deliverable:** Production-ready GUI application

---

## Appendix: Data Structures

### Core State Types

```rust
/// Main application state
pub struct AppState {
    pub study_path: Option<PathBuf>,
    pub study_name: String,
    pub domains: Vec<DomainState>,
    pub current_domain_idx: Option<usize>,
    pub standards: Standards,
    pub recent_studies: Vec<RecentStudy>,
}

/// Per-domain state
pub struct DomainState {
    pub code: String,
    pub label: String,
    pub source_files: Vec<PathBuf>,
    pub variables: Vec<VariableMapping>,
    pub unmapped_sources: Vec<String>,
    pub supp_decisions: Vec<SuppDecision>,
    pub status: MappingStatus,
}

/// Variable mapping state
pub struct VariableMapping {
    pub variable: Variable,
    pub source_column: Option<String>,
    pub confidence: Option<f32>,
    pub ct_validation: Option<CtValidation>,
    pub status: MappingStatus,
}

/// Mapping status enum
pub enum MappingStatus {
    AutoGenerated,
    Mapped,
    NeedsReview,
    Unmapped,
    Skipped,
}

/// SUPP decision
pub struct SuppDecision {
    pub source_column: String,
    pub qnam: String,
    pub qlabel: String,
    pub qorig: String,
    pub qeval: Option<String>,
}
```

---

## Next Steps

1. **Create `sdtm-gui` crate** with basic eframe setup
2. **Implement welcome screen** with folder picker
3. **Build mapping review** using SDTM-first design
4. **Integrate with existing crates** for data loading and output

This document serves as the architectural blueprint for transforming the CLI tool into a modern, user-friendly GUI application using egui.
