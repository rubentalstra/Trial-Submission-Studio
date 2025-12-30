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

**Architecture**: GUI-only desktop application with modular, state-driven design
supporting non-linear, interactive workflows.

---

## Table of Contents

1. [Architecture Transformation Overview](#architecture-transformation-overview)
2. [Crate Structure](#crate-structure)
3. [Understanding the Domain](#understanding-the-domain)
4. [User Goals & Workflow](#user-goals--workflow)
5. [Information Architecture](#information-architecture)
6. [Detailed Screen Specifications](#detailed-screen-specifications)
7. [State Management](#state-management)
8. [Error Handling Strategy](#error-handling-strategy)
9. [Technical Implementation](#technical-implementation)
10. [Performance Optimization](#performance-optimization)
11. [Testing Strategy](#testing-strategy)
12. [Accessibility](#accessibility)
13. [Migration Strategy](#migration-strategy)
14. [Success Criteria](#success-criteria)

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

The GUI requires **independent, reusable components** with **shared runtime state**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          Runtime Study State                            │
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
- Validate individual domains on demand
- Preview transformations before applying
- Undo/redo individual changes within session
- Export subsets of domains

---

## Crate Structure

```
sdtm-transpiler/
├── crates/
│   ├── sdtm-model/          # Core types (NO changes needed)
│   ├── sdtm-standards/      # Standards loading (NO changes needed)
│   ├── sdtm-xpt/            # XPT format (NO changes needed)
│   │
│   ├── sdtm-ingest/         # CSV reading (minor: add preview methods)
│   ├── sdtm-map/            # Mapping engine (split suggest/apply)
│   ├── sdtm-core/           # Domain processing (extract pure functions)
│   ├── sdtm-validate/       # Validation (support incremental)
│   ├── sdtm-report/         # Output generation (support selective export)
│   │
│   └── sdtm-gui/            # NEW: egui application + runtime state
```

### Crate Responsibilities

| Crate            | Purpose                                                |
|------------------|--------------------------------------------------------|
| `sdtm-model`     | Types only (Domain, Variable, Term, etc.). No I/O.     |
| `sdtm-standards` | Load SDTM/CT from offline CSV files.                   |
| `sdtm-ingest`    | CSV discovery, parsing, schema detection.              |
| `sdtm-map`       | Fuzzy column mapping with suggest/apply separation.    |
| `sdtm-core`      | Pure domain processing functions (USUBJID, --SEQ, CT). |
| `sdtm-validate`  | Incremental conformance checks.                        |
| `sdtm-report`    | Multi-format output generation.                        |
| `sdtm-xpt`       | SAS Transport v5 format writer.                        |
| `sdtm-gui`       | egui + eframe app, runtime state, services, undo/redo. |

---

## Understanding the Domain

### What is SDTM?

SDTM (Study Data Tabulation Model) is an FDA-required standard for organizing
clinical trial data. Key concepts:

| Concept                         | Description                          | Example                                  |
|---------------------------------|--------------------------------------|------------------------------------------|
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
3. **Missing required variables**: USUBJID is required but may have a different name
4. **Unmapped columns**: Source columns with no SDTM equivalent go to SUPP domain
5. **Auto-generated fields**: STUDYID, DOMAIN, --SEQ are computed, not mapped

---

## User Goals & Workflow

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

| Screen        | Time | Reason                 |
|---------------|------|------------------------|
| Home          | 5%   | Quick selection        |
| Domain Editor | 85%  | Main work happens here |
| Export        | 10%  | Review and generate    |

**Implication**: The Domain Editor must be exceptionally well-designed.

---

## Information Architecture

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
│   │   ┌─────────┐ ┌───────────┐ ┌────────────┐ ┌─────────┐ ┌──────┐        │
│   │   │ Mapping │ │ Transform │ │ Validation │ │ Preview │ │ SUPP │        │
│   │   └─────────┘ └───────────┘ └────────────┘ └─────────┘ └──────┘        │
│   │                                                         │                │
│   └────────────────────────────────────────────────────────┘                │
│                                │                                             │
│                                │ (done with all domains)                     │
│                                ↓                                             │
│                             EXPORT                                           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Tab Order** (follows workflow): Mapping → Transform → Validation → Preview → SUPP

### Information Hierarchy

1. **Home Screen**: Which domains exist? What's their status? Where to focus?
2. **Domain Editor - Mapping Tab**: Which SDTM variables need attention?
3. **Domain Editor - Transform Tab**: How should values be transformed?
4. **Domain Editor - Validation Tab**: Which values fail CT validation?
5. **Domain Editor - Preview Tab**: What will the output look like?
6. **Domain Editor - SUPP Tab**: Which source columns are unmapped?
7. **Export Screen**: Are all domains ready? What formats to generate?

---

## Detailed Screen Specifications

### Screen 1: Home

**Purpose**: Study selection and domain overview.

#### Section A: Study Selection (no study loaded)

```
╭──────────────────────────────────────────────────────────────────────────────╮
│                                                                ◐    ⚙       │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                                                                              │
│                          CDISC Transpiler                                    │
│                              v0.1.0                                          │
│                                                                              │
│                    ╭┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮                   │
│                    ┊                                      ┊                   │
│                    ┊     Drop study folder here          ┊                   │
│                    ┊        or click to browse           ┊                   │
│                    ┊                                      ┊                   │
│                    ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯                   │
│                                                                              │
│                    Recent                                                    │
│                                                                              │
│                    DEMO_STUDY_001                     2 days ago        →    │
│                    PHASE3_TRIAL_XYZ                  1 week ago        →    │
│                                                                              │
╰──────────────────────────────────────────────────────────────────────────────╯
```

**Interactions**:

- Drop zone: Drag folder or click to open native picker
- Recent items: Click to load directly
- Settings gear: Opens preferences (including theme toggle)

#### Section B: Domain Overview (study loaded)

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
│                                                           + Add Domain       │
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
│  │  ...                                                                    │ │
│  │                                                                         │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  Summary                                                                     │
│  ✓ 10 complete    ● 3 in progress    ○ 3 not started    ✕ 1 has errors      │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                             Export All  →    │
╰──────────────────────────────────────────────────────────────────────────────╯
```

##### Manual Domain Creation

If a domain wasn't auto-detected, users can create it manually:

1. Click "+ Add Domain" button
2. Select domain type from SDTM dropdown (e.g., "Custom FINDINGS")
3. Assign source CSV file
4. System loads file and initializes mapping

##### List Columns

| Column  | Description                                             |
|---------|---------------------------------------------------------|
| Domain  | 2-letter domain code                                    |
| Label   | Human-readable name                                     |
| Class   | SDTM class (Events, Findings, Interventions, Special)   |
| Rows    | Record count in source file                             |
| Mapping | Variables mapped / total (e.g., `14/18`)                |
| Val     | Validation issues: `—` none, `2⚠` warnings, `3✕` errors |
| St      | Overall status icon                                     |

##### Status Icons

| Icon | Meaning                       | Color  |
|------|-------------------------------|--------|
| `○`  | Not started                   | Gray   |
| `●`  | In progress (needs attention) | Yellow |
| `✓`  | Complete                      | Green  |
| `✕`  | Has blocking errors           | Red    |

---

### Screen 2: Domain Editor

**Purpose**: The main workspace where 85% of user time is spent.

**Layout**: Header + Tab bar + Content area

**Tab Order**: Mapping → Transform → Validation → Preview → SUPP

**Tab Badges**:

| Badge    | Meaning                |
|----------|------------------------|
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
│  Mapping (3)     Transform     Validation (5⚠)     Preview     SUPP (2)     │
│  ━━━━━━━━━━━                                                                 │
├────────────────────────────┬─────────────────────────────────────────────────┤
│                            │                                                 │
│  ┌──────────────────────┐  │  SDTM Target                                    │
│  │ Search variables... 🔍│  │ ─────────────────────────────────────────────── │
│  └──────────────────────┘  │                                                 │
│                            │  AETERM                                         │
│  Variables            14   │  Reported Term for the Adverse Event            │
│                            │                                                 │
│  ┌──────────────────────┐  │  ┌─────────────┬─────────────────────────────┐  │
│  │ Name     Core    St  │  │  │ Core        │ Required                    │  │
│  ├──────────────────────┤  │  │ Type        │ Char(200)                   │  │
│  │ STUDYID   —      ⚙   │  │  │ Role        │ Topic                       │  │
│  │ DOMAIN    —      ⚙   │  │  │ Codelist    │ —                           │  │
│  │ USUBJID  Req     ✓   │  │  └─────────────┴─────────────────────────────┘  │
│  │ AESEQ     —      ⚙   │  │                                                 │
│  │ AETERM   Req     ○  ◀│  │  SDTM Examples                                  │
│  │ AEDECOD  Req     ✓   │  │  HEADACHE · NAUSEA · INJECTION SITE PAIN        │
│  │ AECAT    Perm    —   │  │                                                 │
│  │ AEBODSYS Exp     ✓   │  │                                                 │
│  │ AESEV    Exp     ○   │  │  Source Column                                  │
│  │ AESER    Exp     ✓   │  │ ─────────────────────────────────────────────── │
│  │ AEREL    Exp     —   │  │                                                 │
│  │ AESTDTC  Req     ✓   │  │  ┌─────────────────────────────────────────┐    │
│  │ AEENDTC  Exp     ○   │  │  │ ADVERSE_EVENT_TERM              92% ●●○ │    │
│  │ ...                  │  │  └─────────────────────────────────────────┘    │
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
│  └──────────────────────┘  │  ┌─────────────────────────────────────────┐    │
│                            │  │ Select different column...           ▼  │    │
│   Show Source Columns      │  └─────────────────────────────────────────┘    │
│                            │                                                 │
│                            │         Accept               Clear              │
│                            │                                                 │
╰────────────────────────────┴─────────────────────────────────────────────────╯
```

##### Drag-and-Drop Mapping

Users can drag source columns from a floating palette onto SDTM variables:

1. Click "Show Source Columns" button
2. Palette appears with draggable column chips
3. Drag a chip onto a variable row
4. Drop to assign mapping

##### Confidence Indicator

| Score  | Visual | Level                       |
|--------|--------|-----------------------------|
| ≥ 95%  | `●●●`  | High — likely correct       |
| 80-94% | `●●○`  | Medium — review recommended |
| 60-79% | `●○○`  | Low — needs verification    |

##### Mapping Method

| Method   | Description                                       |
|----------|---------------------------------------------------|
| Column   | Map directly to a source column (default)         |
| Constant | Assign a hardcoded value (e.g., "USA")            |
| Derived  | Calculated from other columns (via Transform tab) |

---

#### Tab B: Transform

Configure value transformations and bulk patterns.

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  AE — Adverse Events                                          ◐    ⚙     │
├──────────────────────────────────────────────────────────────────────────────┤
│  Mapping ✓       Transform (2)     Validation (5⚠)     Preview     SUPP     │
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

##### Transform Types

```rust
pub enum TransformRule {
    DateFormat { from_pattern: String, to_pattern: String },
    CtNormalization { variable: String, matching_mode: CtMatchingMode },
    Uppercase { variable: String },
    Lowercase { variable: String },
    Concatenate { target: String, sources: Vec<String>, separator: String },
    Constant { target: String, value: String },
    Custom { expression: String },
}
```

---

#### Tab C: Validation

Shows CT validation issues that must be resolved before export.

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  AE — Adverse Events                                          ◐    ⚙     │
├──────────────────────────────────────────────────────────────────────────────┤
│  Mapping ✓       Transform ✓     Validation (5⚠)     Preview     SUPP       │
│                                  ━━━━━━━━━━━━━━                              │
├────────────────────────────────────┬─────────────────────────────────────────┤
│                                    │                                         │
│  3 issues need resolution          │   AESEV — Severity                      │
│                                    │   Codelist: C66769                      │
│  ┌──────────────────────────────┐  │   Extensible: No                        │
│  │                              │  │                                         │
│  │  ┃ AESEV                     │  │   This codelist is non-extensible.      │
│  │    Severity            ERROR │  │   All values must match exactly.        │
│  │    5 invalid values          │  │                                         │
│  │                              │  │                                         │
│  │    AEREL                     │  │   Invalid values found:                 │
│  │    Causality           WARN  │  │                                         │
│  │    1 sponsor extension       │  │   ┌─────────────────────────────────┐   │
│  │                              │  │   │ Source        Count   Map to    │   │
│  │    AEOUT                     │  │   ├─────────────────────────────────┤   │
│  │    Outcome             WARN  │  │   │ "Mild"        45      MILD   ▼  │   │
│  │    2 sponsor extensions      │  │   │ "Moderate"    38      MODERATE▼ │   │
│  │                              │  │   │ "Severe"      12      SEVERE ▼  │   │
│  └──────────────────────────────┘  │   │ "Grade 1"      5      [Select]▼ │   │
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

**Severity Meanings**:

| Severity | Codelist Type  | Impact                        |
|----------|----------------|-------------------------------|
| ERROR    | Non-extensible | Blocks XPT export             |
| WARN     | Extensible     | Allowed but flagged in report |

---

#### Tab D: Preview

Shows transformed data before export.

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  AE — Adverse Events                                          ◐    ⚙     │
├──────────────────────────────────────────────────────────────────────────────┤
│  Mapping ✓       Transform ✓     Validation ✓       Preview     SUPP        │
│                                                     ━━━━━━━                  │
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

---

#### Tab E: SUPP

Manages unmapped source columns as Supplemental Qualifiers.

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  AE — Adverse Events                                          ◐    ⚙      │
├──────────────────────────────────────────────────────────────────────────────┤
│  Mapping ✓       Transform ✓     Validation ✓       Preview     SUPP (2)    │
│                                                                  ━━━━━━━━    │
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
│                            │  Action                                         │
│                            │                                                 │
│                            │  ● Add to SUPPAE                                │
│                            │  ○ Skip (exclude from output)                   │
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

### Screen 3: Export

**Purpose**: Final review and file generation.

```
╭──────────────────────────────────────────────────────────────────────────────╮
│  ←  Export                                                     ◐    ⚙        │
├──────────────────────────────────────────────────────────────────────────────┤
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
│                                                Generate Files                │
│                                                                              │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

## State Management

### Application State

```rust
pub struct AppState {
    pub view: View,
    pub study: Option<StudyState>,
    pub preferences: Preferences,
    pub toasts: ToastManager,
    pub undo_stack: UndoStack,
    pub cache: CacheLayer,
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

### Study State (Runtime Only)

```rust
pub struct StudyState {
    pub study_id: String,
    pub study_folder: PathBuf,
    pub domains: HashMap<String, DomainState>,
    pub global_settings: GlobalSettings,
}

pub struct DomainState {
    pub code: String,
    pub source_file: PathBuf,
    pub source_data: DataFrame,
    pub status: DomainStatus,
    pub mapping: Option<MappingConfig>,
    pub transforms: Vec<TransformRule>,
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

### Undo/Redo System

Command pattern for reversible operations within the current session:

```rust
pub trait UndoableCommand: Send + Sync {
    fn execute(&self, state: &mut StudyState) -> Result<()>;
    fn undo(&self, state: &mut StudyState) -> Result<()>;
    fn description(&self) -> &str;
}

pub struct UndoStack {
    undo_stack: Vec<Box<dyn UndoableCommand>>,
    redo_stack: Vec<Box<dyn UndoableCommand>>,
    max_size: usize,
}

impl UndoStack {
    pub fn push(&mut self, cmd: Box<dyn UndoableCommand>);
    pub fn undo(&mut self, state: &mut StudyState) -> Result<()>;
    pub fn redo(&mut self, state: &mut StudyState) -> Result<()>;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
}
```

Undoable operations:

- Accept/reject mapping
- Set constant value
- Configure SUPPQUAL assignment
- Resolve CT mismatch
- Apply transform rule

---

## Error Handling Strategy

### Error Categories

| Category   | Example                            | UX Pattern                      |
|------------|------------------------------------|---------------------------------|
| File I/O   | CSV not found, permission denied   | Modal with retry button         |
| Validation | Invalid date format, CT mismatch   | Inline error in field           |
| Processing | Memory exhausted, transform failed | Toast with details link         |

### Error State Types

```rust
pub enum ErrorSeverity {
    Recoverable,  // User can retry or fix
    Fatal,        // Must restart operation
}

pub struct AppError {
    pub severity: ErrorSeverity,
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<String>,
    pub recovery_action: Option<RecoveryAction>,
}

pub enum RecoveryAction {
    Retry,
    Reload,
    OpenSettings,
}

pub enum ErrorCode {
    FileNotFound,
    PermissionDenied,
    InvalidCsv,
    OutOfMemory,
    ValidationFailed,
    ExportFailed,
}
```

### Error Recovery Flows

```
File Not Found
├── Show modal: "File not found: ae.csv"
├── Options: [Browse for file] [Remove domain] [Cancel]
└── User picks action → update state

Validation Errors
├── Highlight field with error
├── Show inline message
├── User fixes value → clear error
└── Auto-validate on change
```

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

### Application Entry Point

```rust
use eframe::egui;

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
    state: AppState,
    task_manager: TaskManager,
}

impl eframe::App for CdiscApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_background_tasks(ctx);
        self.state.file_watcher.check_changes();
        self.state.toasts.render(ctx);

        match &self.state.view {
            View::Home => self.render_home(ctx),
            View::DomainEditor { domain, tab } => {
                self.render_domain_editor(ctx, domain, tab);
            }
            View::Export => self.render_export(ctx),
        }
    }
}
```

### Visual Design System

#### Colors (Light Mode)

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

#### Colors (Dark Mode)

```rust
pub mod dark_colors {
    use egui::Color32;

    pub const BG_PRIMARY: Color32 = Color32::from_rgb(24, 24, 27);
    pub const BG_SECONDARY: Color32 = Color32::from_rgb(39, 39, 42);
    pub const BG_HOVER: Color32 = Color32::from_rgb(63, 63, 70);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(250, 250, 250);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(161, 161, 170);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(113, 113, 122);

    pub const BORDER: Color32 = Color32::from_rgb(63, 63, 70);
}
```

#### Typography

| Use            | Size | Weight |
|----------------|------|--------|
| Page title     | 20px | 600    |
| Section header | 16px | 600    |
| Body           | 14px | 400    |
| Small/Label    | 12px | 500    |

#### Spacing

| Token | Value |
|-------|-------|
| xs    | 4px   |
| sm    | 8px   |
| md    | 16px  |
| lg    | 24px  |
| xl    | 32px  |

### Keyboard Shortcuts

> **Note:** On macOS use `Cmd`, on Windows/Linux use `Ctrl`.

#### Global

| macOS           | Windows/Linux     | Action                 |
|-----------------|-------------------|------------------------|
| `Cmd+O`         | `Ctrl+O`          | Open study             |
| `Cmd+Z`         | `Ctrl+Z`          | Undo                   |
| `Cmd+Shift+Z`   | `Ctrl+Y`          | Redo                   |
| `Cmd+E`         | `Ctrl+E`          | Go to Export           |
| `Cmd+,`         | `Ctrl+,`          | Settings               |
| `Esc`           | `Esc`             | Go back / Close dialog |
| `?`             | `?`               | Show shortcuts help    |

#### Domain Editor

| Shortcut    | Action                      |
|-------------|-----------------------------|
| `↑` / `↓`   | Move up/down in list        |
| `←` / `→`   | Switch focus between panels |
| `Enter`     | Accept suggestion           |
| `Backspace` | Clear mapping               |
| `Tab`       | Next field                  |

### Background Task System

```rust
pub struct TaskManager {
    active_tasks: HashMap<TaskId, TaskHandle>,
    max_concurrent: usize,
}

pub struct TaskHandle {
    id: TaskId,
    cancel_token: CancellationToken,
    progress: Arc<AtomicU32>,
    result_receiver: Receiver<TaskResult>,
}

impl TaskManager {
    pub fn spawn(&mut self, task: BackgroundTask) -> TaskId;
    pub fn cancel(&mut self, id: TaskId);
    pub fn cancel_all(&mut self);
    pub fn get_progress(&self, id: TaskId) -> Option<u32>;
}

pub enum BackgroundTask {
    LoadStudy(PathBuf),
    ValidateDomain(String),
    ExportDomains(Vec<String>),
}
```

Thread safety:

- `StudyState` accessed only from main thread
- Background tasks receive cloned data, return results via channel
- UI updates via `ctx.request_repaint()` polling

### egui Component Patterns

#### Master-Detail Layout

```rust
pub struct MasterDetailPanel<M, D> {
    master_items: Vec<M>,
    selected_index: Option<usize>,
    master_width: f32,
    _detail: PhantomData<D>,
}

impl<M, D> MasterDetailPanel<M, D>
where
    M: MasterListItem,
    D: DetailView<M>,
{
    pub fn show(&mut self, ui: &mut egui::Ui, detail_ctx: &mut D::Context) {
        egui::SidePanel::left("master_panel")
            .resizable(true)
            .default_width(self.master_width)
            .width_range(200.0..=400.0)
            .show_inside(ui, |ui| {
                self.render_master_list(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(idx) = self.selected_index {
                D::render(ui, &self.master_items[idx], detail_ctx);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select an item from the list");
                });
            }
        });
    }
}
```

#### Searchable CT Picker

```rust
pub struct CtPicker {
    id: egui::Id,
    search_text: String,
    is_open: bool,
    filtered_terms: Vec<Term>,
}

impl CtPicker {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        codelist: &Codelist,
        selected: &mut Option<String>,
    ) -> bool {
        let mut changed = false;
        let button_text = selected.as_deref().unwrap_or("Select value...");

        let response = ui.add(
            egui::Button::new(format!("{} ▼", button_text))
                .min_size(egui::vec2(200.0, 0.0))
        );

        if response.clicked() {
            self.is_open = !self.is_open;
            if self.is_open {
                self.search_text.clear();
                self.filtered_terms = codelist.terms.clone();
            }
        }

        if self.is_open {
            egui::Area::new(self.id.with("popup"))
                .fixed_pos(response.rect.left_bottom())
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(200.0);
                        ui.set_max_height(300.0);

                        let search_response = ui.add(
                            egui::TextEdit::singleline(&mut self.search_text)
                                .hint_text("Search...")
                        );
                        search_response.request_focus();

                        if search_response.changed() {
                            self.filtered_terms = codelist.terms
                                .iter()
                                .filter(|t| t.submission_value.to_lowercase()
                                    .contains(&self.search_text.to_lowercase()))
                                .cloned()
                                .collect();
                        }

                        ui.separator();

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for term in &self.filtered_terms {
                                if ui.selectable_label(
                                    selected.as_ref() == Some(&term.submission_value),
                                    &term.submission_value
                                ).clicked() {
                                    *selected = Some(term.submission_value.clone());
                                    self.is_open = false;
                                    changed = true;
                                }
                            }
                        });
                    });
                });
        }

        changed
    }
}
```

#### Data Table with Pagination

```rust
pub struct DataTable {
    page: usize,
    page_size: usize,
    sort_column: Option<usize>,
    sort_ascending: bool,
}

impl DataTable {
    pub fn show(&mut self, ui: &mut egui::Ui, df: &DataFrame) {
        let total_rows = df.height();
        let total_pages = (total_rows + self.page_size - 1) / self.page_size;
        let start_row = self.page * self.page_size;
        let end_row = (start_row + self.page_size).min(total_rows);

        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .columns(egui_extras::Column::auto(), df.width())
            .header(24.0, |mut header| {
                for (col_idx, col_name) in df.get_column_names().iter().enumerate() {
                    header.col(|ui| {
                        let is_sorted = self.sort_column == Some(col_idx);
                        let arrow = if is_sorted {
                            if self.sort_ascending { " ▲" } else { " ▼" }
                        } else { "" };

                        if ui.button(format!("{}{}", col_name, arrow)).clicked() {
                            if is_sorted {
                                self.sort_ascending = !self.sort_ascending;
                            } else {
                                self.sort_column = Some(col_idx);
                                self.sort_ascending = true;
                            }
                        }
                    });
                }
            })
            .body(|body| {
                body.rows(text_height + 4.0, end_row - start_row, |mut row| {
                    let row_idx = start_row + row.index();
                    for col_idx in 0..df.width() {
                        row.col(|ui| {
                            if let Ok(series) = df.select_at_idx(col_idx) {
                                let value = format_cell_value(series, row_idx);
                                ui.label(value);
                            }
                        });
                    }
                });
            });

        // Pagination controls
        ui.horizontal(|ui| {
            ui.label(format!("Rows {}-{} of {}", start_row + 1, end_row, total_rows));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add_enabled(self.page < total_pages - 1, egui::Button::new("→")).clicked() {
                    self.page += 1;
                }
                ui.label(format!("{} / {}", self.page + 1, total_pages));
                if ui.add_enabled(self.page > 0, egui::Button::new("←")).clicked() {
                    self.page -= 1;
                }
            });
        });
    }
}
```

#### Toast Notifications

```rust
pub struct ToastManager {
    toasts: Vec<Toast>,
    next_id: u64,
}

pub struct Toast {
    id: u64,
    message: String,
    severity: ToastSeverity,
    created_at: Instant,
    duration: Duration,
}

pub enum ToastSeverity {
    Success,
    Warning,
    Error,
    Info,
}

impl ToastManager {
    pub fn show(&mut self, message: impl Into<String>, severity: ToastSeverity) {
        self.toasts.push(Toast {
            id: self.next_id,
            message: message.into(),
            severity,
            created_at: Instant::now(),
            duration: Duration::from_secs(4),
        });
        self.next_id += 1;
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        self.toasts.retain(|t| t.created_at.elapsed() < t.duration);

        let screen_rect = ctx.screen_rect();
        let mut y_offset = 20.0;

        for toast in self.toasts.iter().rev() {
            let elapsed = toast.created_at.elapsed().as_secs_f32();
            let fade = if elapsed > toast.duration.as_secs_f32() - 0.3 {
                1.0 - (elapsed - (toast.duration.as_secs_f32() - 0.3)) / 0.3
            } else {
                1.0
            };

            let (bg_color, icon) = match toast.severity {
                ToastSeverity::Success => (colors::SUCCESS, "✓"),
                ToastSeverity::Warning => (colors::WARNING, "⚠"),
                ToastSeverity::Error => (colors::ERROR, "✕"),
                ToastSeverity::Info => (colors::ACCENT, "ℹ"),
            };

            egui::Area::new(egui::Id::new(toast.id))
                .fixed_pos(egui::pos2(
                    screen_rect.right() - 320.0,
                    screen_rect.bottom() - y_offset - 60.0,
                ))
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(bg_color.linear_multiply(fade))
                        .rounding(8.0)
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(icon).color(Color32::WHITE));
                                ui.label(egui::RichText::new(&toast.message).color(Color32::WHITE));
                            });
                        });
                });

            y_offset += 70.0;
        }

        if !self.toasts.is_empty() {
            ctx.request_repaint();
        }
    }
}
```

#### Drag-and-Drop

```rust
pub struct DragDropState {
    dragging: Option<String>,
    drop_target: Option<String>,
}

impl DragDropState {
    pub fn render_draggable_source(&mut self, ui: &mut egui::Ui, column: &str) -> egui::Response {
        let response = ui.add(
            egui::Label::new(format!("⋮⋮ {}", column))
                .sense(egui::Sense::drag())
        );

        if response.drag_started() {
            self.dragging = Some(column.to_string());
        }

        if response.dragged() {
            if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                egui::Area::new(egui::Id::new("drag_preview"))
                    .fixed_pos(pointer + egui::vec2(10.0, 10.0))
                    .show(ui.ctx(), |ui| {
                        egui::Frame::popup(ui.style())
                            .fill(colors::ACCENT.linear_multiply(0.9))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(column).color(Color32::WHITE));
                            });
                    });
            }
        }

        if response.drag_stopped() {
            self.dragging = None;
        }

        response
    }

    pub fn render_drop_target(&mut self, ui: &mut egui::Ui, variable: &str) -> Option<String> {
        let response = ui.add(
            egui::Label::new(variable).sense(egui::Sense::hover())
        );

        let is_hovered = response.hovered() && self.dragging.is_some();

        if is_hovered {
            self.drop_target = Some(variable.to_string());
            ui.painter().rect_stroke(
                response.rect.expand(4.0),
                4.0,
                egui::Stroke::new(2.0, colors::ACCENT),
            );
        }

        if is_hovered && ui.input(|i| i.pointer.any_released()) {
            if let Some(column) = self.dragging.take() {
                self.drop_target = None;
                return Some(column);
            }
        }

        None
    }
}
```

#### Modal Dialog

```rust
pub struct Modal {
    is_open: bool,
    title: String,
    width: f32,
}

impl Modal {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            is_open: false,
            title: title.into(),
            width: 400.0,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn show<R>(
        &mut self,
        ctx: &egui::Context,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<R> {
        if !self.is_open {
            return None;
        }

        // Dim background
        egui::Area::new(egui::Id::new("modal_backdrop"))
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                let screen = ctx.screen_rect();
                ui.painter().rect_filled(
                    screen,
                    0.0,
                    Color32::from_black_alpha(128),
                );
            });

        let mut result = None;
        egui::Window::new(&self.title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([self.width, 0.0])
            .show(ctx, |ui| {
                result = Some(add_contents(ui));

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.is_open = false;
                    }
                });
            });

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.is_open = false;
        }

        result
    }
}
```

---

## Performance Optimization

### Caching Layer

```rust
use lru::LruCache;

pub struct CacheLayer {
    // CT lookups: codelist_code -> terms
    ct_cache: LruCache<String, Vec<Term>>,

    // Fuzzy matching: (needle, haystack) -> score
    fuzzy_cache: LruCache<(String, String), f32>,

    // Column stats: (file_path, column) -> stats
    stats_cache: LruCache<(PathBuf, String), ColumnStats>,
}

impl CacheLayer {
    pub fn new(capacity: usize) -> Self {
        Self {
            ct_cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
            fuzzy_cache: LruCache::new(NonZeroUsize::new(capacity * 10).unwrap()),
            stats_cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
        }
    }

    pub fn invalidate_for_file(&mut self, path: &Path) {
        self.stats_cache.retain(|k, _| &k.0 != path);
    }
}
```

Cache invalidation triggers:

- File reload
- CT version change
- Load new study

### Virtual Scrolling

For DataFrames > 1000 rows, use virtualized rendering:

```rust
fn render_virtualized_table(&self, ui: &mut egui::Ui, df: &DataFrame) {
    let row_height = 20.0;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, row_height, df.height(), |ui, row_range| {
            for row_idx in row_range {
                self.render_row(ui, df, row_idx);
            }
        });
}
```

Only rows in the visible viewport are rendered, keeping frame time < 16ms for 100k+ rows.

### Progress Indicators

| Operation      | Progress Type | Source                          |
|----------------|---------------|---------------------------------|
| File loading   | Determinate   | bytes_read / total_bytes        |
| CT validation  | Determinate   | rows_checked / total_rows       |
| Fuzzy matching | Indeterminate | N/A                             |
| Export         | Determinate   | domains_written / total_domains |

---

## Testing Strategy

### Unit Tests

| Layer    | Target                                | Approach                          |
|----------|---------------------------------------|-----------------------------------|
| Services | `MappingService`, `ValidationService` | Mock DataFrames, verify outputs   |
| State    | `UndoStack`, `DomainState`            | Property-based tests (quickcheck) |

### Integration Tests

| Scenario       | Approach                                             |
|----------------|------------------------------------------------------|
| Full workflow  | Load study → map → validate → export, compare output |
| Large datasets | 10k rows, measure memory/time                        |

### GUI Testing

- Manual QA checklist for each release
- Screenshot regression testing (egui snapshot crate)
- Accessibility audit (contrast, keyboard nav)

### Coverage Targets

- Services: 80%+
- State management: 90%+
- GUI rendering: Manual QA

---

## Accessibility

### Color Contrast

All color combinations meet WCAG 2.1 AA (4.5:1 for normal text, 3:1 for large):

| Combination                    | Ratio  | Status         |
|--------------------------------|--------|----------------|
| `TEXT_PRIMARY` on `BG_PRIMARY` | 14.5:1 | ✓              |
| `ERROR` on `BG_PRIMARY`        | 4.8:1  | ✓              |
| `WARNING` on `BG_PRIMARY`      | 3.2:1  | ✓ (large text) |

### Keyboard Navigation

- All interactive elements focusable via Tab
- Arrow keys navigate within lists/tables
- Escape closes dialogs/returns to previous view
- Enter activates focused element
- Vim-style navigation (j/k/h/l) in Domain Editor

### Screen Readers

- All buttons/inputs have accessibility labels
- Status changes announced via `egui::Response::changed()`
- Focus management on dialog open/close

---

## Migration Strategy

### Phase 1: Foundation ✅ IN PROGRESS

**Goal:** Set up infrastructure

**Status:**
- [x] Create `sdtm-gui` crate with egui 0.33.3
- [x] Set up eframe application structure
- [x] Define `AppState`, `StudyState`, `DomainState`
- [ ] Implement undo/redo stack
- [x] Implement basic navigation (Home ↔ Domain Editor ↔ Export)
- [x] Replace `sdtm-cli` with `sdtm-gui` in workspace

**Deliverable:** Empty GUI shell that compiles and runs ✅

### Phase 2: Extract Core Functions ⏳ NOT STARTED

**Goal:** Refactor `sdtm-core` into standalone functions

**Status:**
- [ ] Extract `apply_usubjid_prefix` from `processor.rs`
- [ ] Extract `assign_sequence_numbers` from `processor.rs`
- [ ] Extract `normalize_ct_column` from `processor.rs`
- [ ] Keep `domain_processors/` as-is (pure business logic)
- [ ] Implement `ProcessingService` in `sdtm-gui`

**Deliverable:** Core logic is modular and testable

### Phase 3: Mapping Service ⏳ NOT STARTED

**Goal:** Make mapping work independently

**Status:**
- [ ] Refactor `sdtm-map` to have `suggest_column_mappings()` (pure suggestions)
- [ ] Add `apply_single_mapping()` to apply one mapping
- [ ] Add `preview_mapping()` for sample rows
- [ ] Implement `MappingService` in `sdtm-gui`
- [ ] Build Mapping tab in GUI

**Deliverable:** Can map domains interactively

### Phase 4: Validation & Transforms ⏳ NOT STARTED

**Goal:** Real-time validation and configurable transforms

**Status:**
- [ ] Enhance `sdtm-validate` with `validate_variable_values()` for incremental checks
- [ ] Add `preview_ct_mapping()` for user confirmation
- [ ] Implement Transform tab
- [ ] Build Validation tab in GUI

**Deliverable:** Interactive validation with fix suggestions

### Phase 5: Processing & Preview ⏳ NOT STARTED

**Goal:** Show final output with all transforms applied

**Status:**
- [ ] Complete `ProcessingService`
- [ ] Handle SUPPQUAL user selection
- [ ] Implement Preview tab
- [ ] Implement SUPP tab

**Deliverable:** Full domain processing visible

### Phase 6: Export & Polish ⏳ NOT STARTED

**Goal:** Export functionality and UX refinement

**Status:**
- [ ] Enhance `sdtm-report` for selective export
- [ ] Implement Export screen
- [ ] Polish UI (keyboard shortcuts, toasts, help)

**Deliverable:** Production-ready GUI

---

## Success Criteria

### Functional Requirements

- [ ] Load a study folder and see discovered domains
- [ ] Manually create domains not auto-detected
- [ ] Map variables with AI-assisted suggestions
- [ ] Use drag-and-drop for column mapping
- [ ] Configure transforms and see previews
- [ ] Validate against CT with fix suggestions
- [ ] Preview final output before export
- [ ] Control SUPPQUAL generation
- [ ] Export individual or all domains
- [ ] Undo/redo mapping changes within session

### Technical Requirements

- [ ] Services decoupled from pipeline orchestration
- [ ] Each domain processes independently
- [ ] Operations are composable (no pipeline)
- [ ] Background tasks are cancellable

### Performance Requirements

- [ ] Load 100+ domains in < 10 seconds
- [ ] Mapping suggestions appear in < 500ms
- [ ] Validation updates in < 1 second
- [ ] Preview renders 1000 rows at 60 FPS
- [ ] Export completes with progress indicator

### UX Requirements

- [ ] Clear visual hierarchy
- [ ] Intuitive workflow (no getting stuck)
- [ ] Helpful error messages
- [ ] Progress indicators for long operations
- [ ] Keyboard shortcuts for common actions
- [ ] Dark mode support
- [ ] WCAG 2.1 AA accessibility

---

## Risks and Mitigation

| Risk                              | Impact | Probability | Mitigation                                     |
|-----------------------------------|--------|-------------|------------------------------------------------|
| Performance with large DataFrames | High   | Low         | Pagination, virtual scrolling, caching         |
| egui learning curve               | Medium | Medium      | Start with simple layouts, iterate             |
| State management complexity       | Medium | Medium      | Comprehensive tests, clear state boundaries    |
| Cross-platform issues             | Low    | Low         | Test on all platforms, use eframe abstractions |

---

## File Structure

```
crates/
└── sdtm-gui/
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── app.rs               # CdiscApp, eframe::App impl
        ├── theme.rs             # Colors, spacing, fonts
        ├── cache.rs             # CacheLayer
        ├── tasks.rs             # TaskManager, BackgroundTask
        ├── undo.rs              # UndoStack, UndoableCommand
        ├── state/
        │   ├── mod.rs
        │   ├── app_state.rs     # AppState, View
        │   ├── study_state.rs   # StudyState, DomainState
        │   └── domain_state.rs  # DomainStatus, transforms
        ├── services/
        │   ├── mod.rs
        │   ├── mapping.rs       # MappingService
        │   ├── processing.rs    # ProcessingService
        │   ├── validation.rs    # ValidationService
        │   └── export.rs        # ExportService
        ├── views/
        │   ├── mod.rs
        │   ├── home.rs          # Home screen
        │   ├── domain_editor.rs # Main editor
        │   └── export.rs        # Export screen
        ├── tabs/
        │   ├── mod.rs
        │   ├── mapping.rs       # Mapping tab
        │   ├── transform.rs     # Transform tab
        │   ├── validation.rs    # Validation tab
        │   ├── preview.rs       # Preview tab
        │   └── supp.rs          # SUPP tab
        ├── components/
        │   ├── mod.rs
        │   ├── master_detail.rs # MasterDetailPanel
        │   ├── ct_picker.rs     # CtPicker
        │   ├── data_table.rs    # DataTable
        │   ├── status_badge.rs  # StatusBadge
        │   ├── toast.rs         # ToastManager
        │   ├── drag_drop.rs     # DragDropState
        │   └── modal.rs         # Modal
        └── dialogs/
            ├── mod.rs
            ├── settings.rs      # Settings dialog
            └── shortcuts.rs     # Keyboard shortcuts help
```

---

## Summary

This GUI is designed around one core insight: **the user's job is to fill SDTM
variables with source data**.

The interface reflects this by:

1. **Centering on SDTM variables** — the left panel always shows what needs to be filled
2. **Highlighting what needs attention** — clear status indicators and filtering
3. **Providing contextual help** — suggestions with confidence scores and sample data
4. **Minimizing navigation** — everything for a domain happens in one place
5. **Progressive disclosure** — simple list view with details on selection

The five-tab design (Mapping → Transform → Validation → Preview → SUPP) follows
the natural workflow while allowing non-linear exploration.

**Architecture**: GUI-only, modular services, runtime state with undo/redo
