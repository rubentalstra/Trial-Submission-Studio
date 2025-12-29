# CDISC Transpiler GUI Architecture & Workflow Design

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Why egui?](#why-egui)
3. [Modern GUI Layout Design](#modern-gui-layout-design)
4. [Current Architecture Analysis](#current-architecture-analysis)
5. [Proposed GUI Architecture](#proposed-gui-architecture)
6. [Data Flow & State Management](#data-flow--state-management)
7. [UI/UX Workflow Design](#uiux-workflow-design)
8. [Screen Layouts & Wireframes](#screen-layouts--wireframes)
9. [Component Architecture](#component-architecture)
10. [Mapping Confidence System](#mapping-confidence-system)
11. [SUPP Domain Fallback Workflow](#supp-domain-fallback-workflow)
12. [Technical Implementation Roadmap](#technical-implementation-roadmap)
13. [Appendix: Data Structures](#appendix-data-structures)

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

## Modern GUI Layout Design

### Core Design Philosophy

Unlike terminal-based interfaces, a desktop GUI enables **rich navigation**, **data visualization**, and **scalable layouts**. The CDISC Transpiler GUI follows modern data management application patterns (similar to database tools, Excel, Tableau) that can handle **30+ domains** without overwhelming the user.

**Key Scalability Challenge**: A study can have 30+ domains. Putting all domains in a sidebar creates an unmanageably long list. Instead, we use:
- **Toolbar-based domain selection** with dropdown/search
- **Domain cards in main content** for overview
- **Wizard-style workflow** for sequential processing

### Master Layout: Toolbar + Main Content

The application uses a **top toolbar** for navigation and a **full-width main content area** that adapts to the current workflow phase.

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  🏥 CDISC Transpiler                                              [─] [□] [×]       │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  File   Edit   View   Tools   Help                                                  │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  📁 DEMO_CF1234  │  Domain: [▼ Select Domain...      ] │  ████░░░ 4/8  │ [Generate] │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│                                                                                     │
│                         ╔═══════════════════════════════╗                           │
│                         ║                               ║                           │
│                         ║      MAIN CONTENT AREA        ║                           │
│                         ║                               ║                           │
│                         ║   Full width for data tables  ║                           │
│                         ║   and mapping interfaces      ║                           │
│                         ║                               ║                           │
│                         ╚═══════════════════════════════╝                           │
│                                                                                     │
│                                                                                     │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  Status: Ready  │  Mapped: 89/158 vars  │  SUPP: 7  │  ⚠ 2 warnings                 │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### Toolbar Design (Always Visible)

The toolbar provides **compact navigation** that scales regardless of domain count:

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                     │
│  ┌─────────────┐   ┌──────────────────────────────┐   ┌─────────┐   ┌────────────┐  │
│  │📁 DEMO_CF1234│   │ Domain: [VS - Vital Signs ▼] │   │████░ 4/8│   │[🚀Generate]│  │
│  │ [Change...]  │   │ [🔍 Search domains...]       │   │ domains │   │            │  │
│  └─────────────┘   └──────────────────────────────┘   └─────────┘   └────────────┘  │
│                                                                                     │
│   Study Info        Domain Selector (dropdown)       Progress       Main Action    │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

**Domain Selector Dropdown** (handles 30+ domains elegantly):

```
┌──────────────────────────────────────┐
│ 🔍 [Search domains...            ]   │
├──────────────────────────────────────┤
│                                      │
│  ── Pending (4) ──────────────────   │
│  ⏳ VS  Vital Signs         0/11     │
│  ⏳ EX  Exposure            0/15     │
│  ⏳ MH  Medical History     0/12     │
│  ⏳ DS  Disposition         0/8      │
│                                      │
│  ── Complete (4) ─────────────────   │
│  ✅ DM  Demographics       13/13     │
│  ✅ AE  Adverse Events     21/21     │
│  ✅ CM  Concom Meds        16/18     │
│  ✅ LB  Laboratory         25/30     │
│                                      │
│  ── Not in Study (20) ────────────   │
│  ⚪ EC, EG, FA, IE, MB, MI...        │
│  [Show all 20...]                    │
│                                      │
└──────────────────────────────────────┘
```

### Workflow Phases

The application has distinct phases, each with an optimized layout:

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                     │
│   Phase 1           Phase 2              Phase 3            Phase 4                 │
│  ┌─────────┐      ┌─────────────┐      ┌───────────┐      ┌────────────┐            │
│  │ Welcome │ ──▶  │   Domain    │ ──▶  │  Mapping  │ ──▶  │  Generate  │            │
│  │  Load   │      │  Overview   │      │  Review   │      │   Output   │            │
│  │  Study  │      │  (Cards)    │      │ (per var) │      │            │            │
│  └─────────┘      └─────────────┘      └───────────┘      └────────────┘            │
│                                                                                     │
│  Full-width        Card grid           List+Detail         Summary +               │
│  centered          (scalable)          (2-panel)           Progress                │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### Main Content Area Patterns

#### Pattern 1: Welcome Screen (Full-Width Centered)

Clean, focused interface for study selection:

```
┌────────────────────────────────────────────────────────────────────────┐
│                                                                        │
│                                                                        │
│                        🏥 CDISC SDTM Transpiler                        │
│                             Version 0.1.0                              │
│                                                                        │
│         Convert clinical trial data to CDISC SDTM format               │
│                                                                        │
│                                                                        │
│       ┌──────────────────────────────────────────────────────┐         │
│       │  📂 Recent Studies                                   │         │
│       ├──────────────────────────────────────────────────────┤         │
│       │  📁 DEMO_CF1234_NL_20250120       Jan 20, 2025       │         │
│       │  📁 DEMO_GDISC_20240903           Sep 03, 2024       │         │
│       │  📁 TRIAL_XYZ_2024                Aug 15, 2024       │         │
│       └──────────────────────────────────────────────────────┘         │
│                                                                        │
│            ┌─────────────────────┐  ┌─────────────────────┐            │
│            │  📂 Open Folder...  │  │  📋 Paste Path...   │            │
│            └─────────────────────┘  └─────────────────────┘            │
│                                                                        │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

#### Pattern 2: Domain Overview (Card Grid - Scalable)

Shows ALL domains as cards in a responsive grid. Works for 8 or 50 domains:

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  📁 DEMO_CF1234  │  Domain: [▼ Overview          ] │  ████░░░ 4/8  │ [🚀 Generate]  │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│   Study: DEMO_CF1234_NL_20250120                                                    │
│   Path: /data/studies/DEMO_CF1234_NL_20250120                                       │
│   Domains Found: 8 of 52 possible  │  Progress: 4/8 complete (50%)                  │
│                                                                                     │
│   ┌─ Filter: [🔍 Search...        ] ─┐  ┌─ Show: ○ All  ● In Study  ○ Pending ─┐    │
│                                                                                     │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐│
│   │ ✅ DM           │  │ ✅ AE           │  │ ✅ CM           │  │ ✅ LB           ││
│   │ Demographics    │  │ Adverse Events  │  │ Concom Meds     │  │ Laboratory      ││
│   │ ────────────    │  │ ────────────    │  │ ────────────    │  │ ────────────    ││
│   │ dm.csv          │  │ ae.csv          │  │ cm.csv          │  │ lb_*.csv (3)    ││
│   │ 342 rows        │  │ 1,205 rows      │  │ 892 rows        │  │ 2,450 rows      ││
│   │                 │  │                 │  │                 │  │                 ││
│   │ Req: 8/8   ✓    │  │ Req: 6/6   ✓    │  │ Req: 5/5   ✓    │  │ Req: 10/10 ✓    ││
│   │ Exp: 5/5        │  │ Exp: 10/15      │  │ Exp: 6/8        │  │ Exp: 8/12       ││
│   │ SUPP: 0         │  │ SUPP: 2         │  │ SUPP: 3         │  │ SUPP: 1         ││
│   │                 │  │                 │  │                 │  │                 ││
│   │ [View Mapping]  │  │ [View Mapping]  │  │ [View Mapping]  │  │ [View Mapping]  ││
│   └─────────────────┘  └─────────────────┘  └─────────────────┘  └─────────────────┘│
│                                                                                     │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐│
│   │ ⏳ VS           │  │ ⏳ EX           │  │ ⏳ MH           │  │ ⏳ DS           ││
│   │ Vital Signs     │  │ Exposure        │  │ Medical History │  │ Disposition     ││
│   │ ────────────    │  │ ────────────    │  │ ────────────    │  │ ────────────    ││
│   │ vs.csv          │  │ ex.csv          │  │ mh.csv          │  │ ds.csv          ││
│   │ 856 rows        │  │ 567 rows        │  │ 234 rows        │  │ 342 rows        ││
│   │                 │  │                 │  │                 │  │                 ││
│   │ Req: 0/5   ⚠    │  │ Req: 0/7   ⚠    │  │ Req: 0/4   ⚠    │  │ Req: 0/4   ⚠    ││
│   │ Exp: 0/6        │  │ Exp: 0/8        │  │ Exp: 0/8        │  │ Exp: 0/4        ││
│   │                 │  │                 │  │                 │  │                 ││
│   │ [Start Mapping] │  │ [Start Mapping] │  │ [Start Mapping] │  │ [Start Mapping] ││
│   └─────────────────┘  └─────────────────┘  └─────────────────┘  └─────────────────┘│
│                                                                                     │
│   Click a card to start mapping, or use the domain selector dropdown above         │
│                                                                                     │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  Overview  │  8 domains in study  │  4 complete, 4 pending  │  Ready               │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

#### Pattern 3: Domain Mapping (List + Detail - 2 Panel)

The core mapping interface. Select an SDTM variable on the left, see details and assign source on the right:

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  📁 DEMO_CF1234  │  Domain: [▼ VS - Vital Signs  ] │  ████░░░ 4/8  │ [🚀 Generate]  │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│  VS - Vital Signs                                                                   │
│  Class: Findings  │  Structure: One record per vital sign per visit                 │
│  Source: vs.csv (856 rows, 15 columns)  │  Progress: ████████░░░░ 8/15 mapped       │
│                                                                                     │
│  ┌─ SDTM Variables ────────────────────┬─ Selected Variable ────────────────────────┐
│  │                                     │                                            │
│  │  🔍 [Filter variables...        ]   │  VSTESTCD                                  │
│  │                                     │  ═══════════════════════════════════════   │
│  │  Show: ● All  ○ Required  ○ Unmapped │                                            │
│  │                                     │  Label:  Vital Signs Test Short Name       │
│  │  ── Required (5) ─────────────────  │  Type:   Char (8)                          │
│  │                                     │  Core:   Required                          │
│  │  🔄 STUDYID     [AUTO]              │  Role:   Topic                             │
│  │  🔄 DOMAIN      [AUTO]              │                                            │
│  │  ✅ USUBJID     ← SUBJID            │  📚 Controlled Terminology                 │
│  │  🔄 VSSEQ       [AUTO]              │  ─────────────────────────────────────     │
│  │  ⚠️ VSTESTCD    [NEED] ◀            │  Codelist: C66741 (VSTESTCD)               │
│  │                                     │  Extensible: No (CLOSED)                   │
│  │  ── Expected (6) ─────────────────  │  Terms: SYSBP, DIABP, PULSE, TEMP,         │
│  │                                     │         RESP, HEIGHT, WEIGHT...            │
│  │  ⚠️ VSTEST      [NEED]              │                                            │
│  │  ✅ VSORRES     ← RESULT            │  📖 Description                            │
│  │  ✅ VSORRESU    ← UNIT              │  ─────────────────────────────────────     │
│  │  ⚪ VSSTRESC    --                  │  Short name of the test described          │
│  │  ⚪ VSSTRESN    --                  │  in VSTEST. (Examples: SYSBP, DIABP)       │
│  │  ✅ VSDTC       ← VISIT_DATE        │                                            │
│  │                                     │  ─────────────────────────────────────     │
│  │  ── Permissible (4) ──────────────  │                                            │
│  │                                     │  📊 Assign Source Column                   │
│  │  ⚪ VSCAT       --                  │  ┌────────────────────────────────────┐    │
│  │  ⚪ VSSCAT      --                  │  │                                    │    │
│  │  ⚪ VSPOS       --                  │  │  Best Matches:                     │    │
│  │  ⚪ VSLOC       --                  │  │                                    │    │
│  │                                     │  │  🟢 96%  VITAL_TEST               │    │
│  │                                     │  │         "Vital Test Code"          │    │
│  │  Legend:                            │  │                                    │    │
│  │  🔄 Auto-generated                  │  │  🟡 72%  TEST_CODE                 │    │
│  │  ✅ Mapped                          │  │         "Test Code"                │    │
│  │  ⚠️ Required - needs mapping        │  │                                    │    │
│  │  ⚪ Optional - not mapped           │  │  🔴 45%  PARAM                     │    │
│  │                                     │  │         "Parameter"                │    │
│  │                                     │  │                                    │    │
│  │                                     │  └────────────────────────────────────┘    │
│  │                                     │                                            │
│  │                                     │  [✓ Accept VITAL_TEST]  [Browse All...]    │
│  │                                     │                                            │
│  └─────────────────────────────────────┴────────────────────────────────────────────┘
│                                                                                     │
│  [◀ Back to Overview]                                     [Next Domain: EX ▶]       │
│                                                                                     │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  VS Domain  │  Req: 4/5  Exp: 2/6  │  ⚠️ 2 variables need mapping  │  [Mark Done]   │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### Responsive Design

The GUI adapts to different window sizes:

| Window Size | Toolbar | Main Content |
|-------------|---------|--------------|
| **Wide** (≥1400px) | Full toolbar | Card grid (4 columns) or 2-panel with extra detail |
| **Medium** (1000-1400px) | Full toolbar | Card grid (3 columns) or 2-panel |
| **Narrow** (<1000px) | Compact toolbar | Card grid (2 columns) or stacked panels |
| **Very Narrow** (<800px) | Icon toolbar | Single column, accordion sections |

### Visual Design Language

#### Color Scheme

| Element | Light Theme | Dark Theme | Purpose |
|---------|-------------|------------|---------|
| Background | #FFFFFF | #1E1E1E | Main background |
| Toolbar BG | #F5F5F5 | #252526 | Toolbar background |
| Card BG | #FAFAFA | #2D2D2D | Domain cards background |
| Primary | #0066CC | #4FC3F7 | Actions, links |
| Success | #28A745 | #4CAF50 | Completed, valid |
| Warning | #FFC107 | #FFB300 | Review needed |
| Error | #DC3545 | #F44336 | Issues, blocking |
| Text | #333333 | #E0E0E0 | Primary text |
| Muted | #6C757D | #9E9E9E | Secondary text |

#### Status Indicators

| Icon | Meaning | Used In |
|------|---------|---------|
| ✅ | Complete / Valid / Mapped | Domain list, variable list |
| 🔶 | In Progress / Current | Active domain |
| ⏳ | Pending / Waiting | Unvisited domains |
| ⚠️ | Warning / Review Needed | CT mismatches |
| ❌ | Error / Blocked | Validation errors |
| 🟢 | High Confidence (≥85%) | Mapping suggestions |
| 🟡 | Medium Confidence (70-85%) | Mapping suggestions |
| 🔴 | Low Confidence (<70%) | Mapping suggestions |
| ⚪ | Unmapped | Variables without source |
| 🔄 | Auto-generated | DOMAIN, --SEQ, STUDYID |

### Interaction Patterns

#### Keyboard Navigation

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate variable list |
| `Enter` | Accept suggested mapping |
| `Space` | Toggle selection |
| `Esc` | Close dialog / Go back |
| `Ctrl+S` | Save progress |
| `Ctrl+O` | Open study folder |
| `Tab` | Move between panels |
| `Ctrl+D` | Open domain selector dropdown |
| `Ctrl+G` | Generate SDTM output |

#### Mouse Interactions

| Action | Element | Behavior |
|--------|---------|----------|
| Click | Variable row | Select and show details |
| Double-click | Variable row | Open source selection dialog |
| Right-click | Variable row | Context menu |
| Drag | Source column | Drag-drop onto SDTM variable |
| Hover | Any item | Show tooltip with details |

#### Tooltips

Tooltips provide contextual information without cluttering the UI:

```
┌─────────────────────────────────────────┐
│  CMTRT                                  │
│  ─────────────────────────────          │
│  Label: Reported Name of Drug           │
│  Type: Char (200)                       │
│  Core: Required                         │
│  CT: None                               │
│                                         │
│  Description: Verbatim medication       │
│  name as reported by the investigator.  │
└─────────────────────────────────────────┘
```

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

The GUI uses a **toolbar-based navigation** design that scales well for studies with many domains. Navigation is done via a compact toolbar at the top, keeping the main content area full-width for data tables and mapping interfaces.

**Note**: The wireframes in Section 3 (Modern GUI Layout Design) show the complete screen layouts. This section provides additional detail on specific screens and interactions.

### Dialog: Source Selection (Modal)

When clicking "Browse All..." or double-clicking an SDTM variable, a modal dialog appears:

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                 │
│      ┌─ Select Source Column for VSTESTCD ────────────────────────────────────────────┐        │
│      │                                                                                 │        │
│      │  🔍 Search: [________________________]                                          │        │
│      │                                                                                 │        │
│      │  ┌───────────────────────────────────────────────────────────────────────────┐  │        │
│      │  │                                                                           │  │        │
│      │  │   Conf.  │ Source Column  │ Label (Items.csv)    │ Sample Values          │  │        │
│      │  │  ────────┼────────────────┼──────────────────────┼────────────────────    │  │        │
│      │  │  🟢 96%  │ VITAL_TEST     │ "Vital Test Code"    │ SYSBP, DIABP, PULSE    │  │        │
│      │  │  🟡 78%  │ TEST_CODE      │ "Test Code"          │ BP_SYS, BP_DIA, HR     │  │        │
│      │  │  🔴 45%  │ PARAM          │ "Parameter"          │ Systolic, Diastolic    │  │        │
│      │  │  ⚪ --   │ COMMENTS       │ "Comments"           │ Normal, High           │  │        │
│      │  │                                                                           │  │        │
│      │  │  ─────────────────────────────────────────────────────────────────────    │  │        │
│      │  │  ⚪ --   │ [No mapping - leave empty]                                      │  │        │
│      │  │  📝 --   │ [Use constant value...]                                         │  │        │
│      │  │  📤 --   │ [This column should go to SUPP...]                              │  │        │
│      │  │                                                                           │  │        │
│      │  └───────────────────────────────────────────────────────────────────────────┘  │        │
│      │                                                                                 │        │
│      │  ┌─ Selected: VITAL_TEST ──────────────────────────────────────────────────┐   │        │
│      │  │                                                                          │   │        │
│      │  │  Column: VITAL_TEST    Type: text    Non-null: 856/856 (100%)            │   │        │
│      │  │  Label: "Vital Test Code" (from Items.csv)                               │   │        │
│      │  │  CodeList: VS_TEST (from CodeLists.csv)                                  │   │        │
│      │  │                                                                          │   │        │
│      │  │  Sample Values (first 5 unique):                                         │   │        │
│      │  │  ┌──────────────────────────────────────────────────────────────────┐    │   │        │
│      │  │  │  SYSBP    →  SYSBP  ✅ Valid CT                                  │    │   │        │
│      │  │  │  DIABP    →  DIABP  ✅ Valid CT                                  │    │   │        │
│      │  │  │  PULSE    →  PULSE  ✅ Valid CT                                  │    │   │        │
│      │  │  │  TEMP     →  TEMP   ✅ Valid CT                                  │    │   │        │
│      │  │  │  RESP     →  RESP   ✅ Valid CT                                  │    │   │        │
│      │  │  └──────────────────────────────────────────────────────────────────┘    │   │        │
│      │  │                                                                          │   │        │
│      │  │  ✅ All 5 unique values are valid CT terms                               │   │        │
│      │  │                                                                          │   │        │
│      │  └──────────────────────────────────────────────────────────────────────────┘   │        │
│      │                                                                                 │        │
│      │                    ┌───────────────────┐     ┌───────────────────┐              │        │
│      │                    │   ✓ Select        │     │     Cancel        │              │        │
│      │                    └───────────────────┘     └───────────────────┘              │        │
│      │                                                                                 │        │
│      └─────────────────────────────────────────────────────────────────────────────────┘        │
│                                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Dialog: CT Mismatch Warning (Modal)

When source values don't match the required CT:

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                 │
│      ┌─ ⚠️ CT Value Mismatch ─────────────────────────────────────────────────────────┐        │
│      │                                                                                 │        │
│      │   Variable: SEX                                                                 │        │
│      │   Source Column: GENDER                                                         │        │
│      │   CT Codelist: C66731 (Sex)                                                     │        │
│      │   Extensible: ❌ NO - Values MUST match exactly                                 │        │
│      │                                                                                 │        │
│      │   Some source values do not match CDISC Controlled Terminology:                 │        │
│      │                                                                                 │        │
│      │   ┌─────────────────────────────────────────────────────────────────────────┐   │        │
│      │   │                                                                         │   │        │
│      │   │   Source Value   │ Records │ CT Match   │ Output    │ Status            │   │        │
│      │   │  ────────────────┼─────────┼────────────┼───────────┼─────────────────  │   │        │
│      │   │   Male           │ 156     │ M          │ M         │ ✅ Will normalize │   │        │
│      │   │   Female         │ 142     │ F          │ F         │ ✅ Will normalize │   │        │
│      │   │   Unknown        │ 8       │ U          │ U         │ ✅ Will normalize │   │        │
│      │   │   Other          │ 3       │ ???        │ ???       │ ❌ NOT IN CT      │   │        │
│      │   │                                                                         │   │        │
│      │   └─────────────────────────────────────────────────────────────────────────┘   │        │
│      │                                                                                 │        │
│      │   ❌ ERROR: "Other" (3 records) cannot be mapped to a valid CT term.            │        │
│      │                                                                                 │        │
│      │   This codelist is NOT extensible. You must resolve this:                       │        │
│      │                                                                                 │        │
│      │   ┌─────────────────────────────────────────────────────────────────────────┐   │        │
│      │   │  Map "Other" to:  [ Select CT value...                    ▼]            │   │        │
│      │   │  Valid options: F, INTERSEX, M, U, UNDIFFERENTIATED                     │   │        │
│      │   └─────────────────────────────────────────────────────────────────────────┘   │        │
│      │                                                                                 │        │
│      │           ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │        │
│      │           │  Apply Mapping  │  │  Skip & Flag    │  │     Cancel      │         │        │
│      │           └─────────────────┘  └─────────────────┘  └─────────────────┘         │        │
│      │                                                                                 │        │
│      └─────────────────────────────────────────────────────────────────────────────────┘        │
│                                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Screen: Unmapped Source Columns

After mapping all SDTM variables, show unmapped source columns (uses toolbar navigation):

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  📁 DEMO_CF1234  │  Domain: [▼ CM - Concom Meds  ] │  ████░░░ 4/8  │ [🚀 Generate]  │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│  CM - Unmapped Source Columns                                                       │
│  ═══════════════════════════════════════════════════════════════════════════════    │
│                                                                                     │
│  These source columns were not mapped to any SDTM variable.                         │
│  Choose what to do with each:                                                       │
│                                                                                     │
│  ┌──────────────────────────────────────────────────────────────────────────────┐   │
│  │                                                                              │   │
│  │   Source Column  │ Label (Items.csv)      │ Action                           │   │
│  │  ────────────────┼────────────────────────┼─────────────────────────────     │   │
│  │   COMMENTS       │ "Additional notes"     │ [📤 Send to SUPP           ▼]    │   │
│  │   BATCH_NUM      │ "Drug batch number"    │ [📤 Send to SUPP           ▼]    │   │
│  │   INTERNAL_ID    │ "Internal ID"          │ [⏭️ Skip - not needed       ▼]    │   │
│  │   REVIEWER       │ "Data reviewer"        │ [⏭️ Skip - not needed       ▼]    │   │
│  │                                                                              │   │
│  └──────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                     │
│  ┌─ SUPP Configuration for COMMENTS ────────────────────────────────────────────┐   │
│  │                                                                              │   │
│  │   RDOMAIN:  CM         IDVAR:  CMSEQ                                         │   │
│  │   QNAM:     [CMCOMM____]  (max 8 chars, auto-generated)                      │   │
│  │   QLABEL:   [Comments about medication__________________]                    │   │
│  │   QORIG:    [ CRF                                      ▼]                    │   │
│  │   QEVAL:    [______________________________________] (optional)              │   │
│  │                                                                              │   │
│  │   Preview: SUPPCM will contain 2 additional variables                        │   │
│  │                                                                              │   │
│  └──────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                     │
│  [◀ Back to Mapping]              [✓ Apply & Continue]              [Next: VS ▶]    │
│                                                                                     │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  CM Domain  │  All SDTM variables mapped ✅  │  4 source columns unmapped  │ Review │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### Screen: Output Generation

When all domains are mapped, the Generate view shows a summary and output options:

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  📁 DEMO_CF1234  │  Domain: [▼ Summary & Output  ] │  ████████ 8/8 │ [🚀 Generate]  │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│  Generate SDTM Output                                                               │
│  ═══════════════════════════════════════════════════════════════════════════════    │
│                                                                                     │
│  ┌─ Mapping Summary ─────────────────────────────────────────────────────────────┐  │
│  │                                                                               │  │
│  │   Domain │ Source         │ Required │ Expected  │ SUPP  │ Status            │  │
│  │  ────────┼────────────────┼──────────┼───────────┼───────┼─────────────────  │  │
│  │   DM     │ dm.csv         │ 8/8  ✅  │ 5/5   ✅  │ 0     │ ✅ Ready          │  │
│  │   AE     │ ae.csv         │ 6/6  ✅  │ 12/15     │ 2     │ ✅ Ready          │  │
│  │   CM     │ cm.csv         │ 5/5  ✅  │ 6/8       │ 3     │ ✅ Ready          │  │
│  │   LB     │ lb_*.csv (3)   │ 10/10✅  │ 8/12      │ 1     │ ✅ Ready          │  │
│  │   VS     │ vs.csv         │ 5/5  ✅  │ 4/6       │ 0     │ ✅ Ready          │  │
│  │   EX     │ ex.csv         │ 7/7  ✅  │ 5/8       │ 0     │ ✅ Ready          │  │
│  │   MH     │ mh.csv         │ 4/4  ✅  │ 6/8       │ 1     │ ✅ Ready          │  │
│  │   DS     │ ds.csv         │ 4/4  ✅  │ 3/4       │ 0     │ ✅ Ready          │  │
│  │                                                                               │  │
│  └───────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                     │
│  ┌─ Output Options ─────────────────────────────┬─ Validation Summary ────────────┐ │
│  │                                              │                                 │ │
│  │  Output Formats:                             │  ⚠️ 3 Warnings:                 │ │
│  │  ☑ XPT (SAS Transport v5)                   │  • AE: AESEV has 2 values       │ │
│  │  ☑ Dataset-XML                              │    using sponsor extension      │ │
│  │  ☑ Define-XML v2.1                          │  • LB: LBORRESU missing for     │ │
│  │  ☐ SAS Program Files                        │    12 records                   │ │
│  │                                              │  • CM: CMENDTC is null for      │ │
│  │  Output Location:                            │    all records                  │ │
│  │  📂 [./output/sdtm                  ] [...]  │                                 │ │
│  │                                              │  ❌ 0 Errors                     │ │
│  │  Additional Options:                         │  Ready to generate              │ │
│  │  ☑ Include SUPP datasets                    │                                 │ │
│  │  ☑ Generate validation report               │                                 │ │
│  │                                              │                                 │ │
│  └──────────────────────────────────────────────┴─────────────────────────────────┘ │
│                                                                                     │
│                        ┌──────────────────────────────────────┐                     │
│                        │       🚀 Generate SDTM Output        │                     │
│                        └──────────────────────────────────────┘                     │
│                                                                                     │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  All domains complete ✅  │  7 SUPP variables  │  3 warnings (non-blocking)  │ Ready│
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### Dialog: Output Progress (Modal)

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                 │
│      ┌─ Generating SDTM Output... ─────────────────────────────────────────────────────┐       │
│      │                                                                                 │       │
│      │                                                                                 │       │
│      │         ████████████████████████████████░░░░░░░░░░░░░  72%                      │       │
│      │                                                                                 │       │
│      │         Current: Processing LB domain (2,450 records)                           │       │
│      │                                                                                 │       │
│      │   ┌─────────────────────────────────────────────────────────────────────────┐   │       │
│      │   │                                                                         │   │       │
│      │   │   ✅  DM      dm.xpt, dm.xml                         342 records        │   │       │
│      │   │   ✅  AE      ae.xpt, ae.xml, suppae.xpt            1,205 records        │   │       │
│      │   │   ✅  CM      cm.xpt, cm.xml, suppcm.xpt              892 records        │   │       │
│      │   │   ⏳  LB      Processing...                         2,450 records        │   │       │
│      │   │   ⏸️  VS      Waiting...                               856 records        │   │       │
│      │   │   ⏸️  EX      Waiting...                               567 records        │   │       │
│      │   │   ⏸️  MH      Waiting...                               234 records        │   │       │
│      │   │   ⏸️  DS      Waiting...                               342 records        │   │       │
│      │   │                                                                         │   │       │
│      │   └─────────────────────────────────────────────────────────────────────────┘   │       │
│      │                                                                                 │       │
│      │         Elapsed: 00:01:45                                                       │       │
│      │                                                                                 │       │
│      │                        ┌──────────────────────────┐                             │       │
│      │                        │        ⏹️ Cancel         │                             │       │
│      │                        └──────────────────────────┘                             │       │
│      │                                                                                 │       │
│      └─────────────────────────────────────────────────────────────────────────────────┘       │
│                                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Screen 5b: Output Complete (Modal)

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                 │
│      ┌─ ✅ SDTM Output Generated Successfully! ────────────────────────────────────────┐       │
│      │                                                                                 │       │
│      │                                                                                 │       │
│      │                            ✅  Success!                                         │       │
│      │                                                                                 │       │
│      │         Output: ./output/sdtm/DEMO_CF1234_NL_20250120                           │       │
│      │                                                                                 │       │
│      │   ┌─────────────────────────────────────────────────────────────────────────┐   │       │
│      │   │                                                                         │   │       │
│      │   │   📦 XPT Files (8):                                                     │   │       │
│      │   │      dm.xpt, ae.xpt, cm.xpt, lb.xpt, vs.xpt, ex.xpt, mh.xpt, ds.xpt     │   │       │
│      │   │                                                                         │   │       │
│      │   │   📦 SUPP Files (3):                                                    │   │       │
│      │   │      suppae.xpt, suppcm.xpt, supplb.xpt                                 │   │       │
│      │   │                                                                         │   │       │
│      │   │   📄 Dataset-XML (8 files)                                              │   │       │
│      │   │                                                                         │   │       │
│      │   │   📋 Define-XML: define.xml, define-stylesheet.xsl                      │   │       │
│      │   │                                                                         │   │       │
│      │   └─────────────────────────────────────────────────────────────────────────┘   │       │
│      │                                                                                 │       │
│      │         Total Records: 6,888  │  Processing Time: 00:02:45                      │       │
│      │                                                                                 │       │
│      │    ┌────────────────┐  ┌────────────────┐  ┌────────────────────────────┐       │       │
│      │    │  📂 Open Folder │  │  📊 View Report │  │  🏠 Back to Study          │       │       │
│      │    └────────────────┘  └────────────────┘  └────────────────────────────┘       │       │
│      │                                                                                 │       │
│      └─────────────────────────────────────────────────────────────────────────────────┘       │
│                                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘
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
