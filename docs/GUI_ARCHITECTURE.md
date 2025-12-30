# CDISC Transpiler — GUI Architecture

## Executive Summary

The CDISC Transpiler GUI transforms clinical trial source data into
SDTM-compliant formats. This document defines the user experience, information
architecture, and technical implementation for a desktop application built with
egui.

**Target Users**: Clinical data programmers, biostatisticians, and data managers
who understand SDTM but need an intuitive tool for data transformation.

**Core Task**: Map source CSV columns to SDTM variables, validate against
Controlled Terminology, and export submission-ready files.

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
│   │   ┌─────────┐ ┌──────┐ ┌────────────┐ ┌─────────┐     │                │
│   │   │ Mapping │ │ SUPP │ │ Validation │ │ Preview │     │                │
│   │   └─────────┘ └──────┘ └────────────┘ └─────────┘     │                │
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

2. **Domain Editor - Mapping Tab**
   - Which SDTM variables need attention?
   - What's the suggested mapping for each?
   - How confident is the system?

3. **Domain Editor - SUPP Tab**
   - Which source columns are unmapped?
   - Should they be included in SUPPQUAL?
   - What are the QNAM/QLABEL values?

4. **Domain Editor - Validation Tab**
   - Which values fail CT validation?
   - What are the valid alternatives?
   - How many occurrences are affected?

5. **Domain Editor - Preview Tab**
   - What will the output look like?
   - Are transformations applied correctly?

6. **Export Screen**
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

**Tab Order**: Mapping → SUPP → Validation → Preview (workflow sequence)

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

---

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
│  │    Outcome             WARN  │  │   ├─────────────────────────────────┤   │
│  │    1 sponsor extension       │  │   │ "Mild"        45      MILD   ▼  │   │
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

Shows transformed data before export.

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

**Features**:

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
    Validation,
    Preview,
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

| Shortcut    | Action                       |
| ----------- | ---------------------------- |
| `↑` `↓`     | Navigate variable list       |
| `Enter`     | Accept suggestion            |
| `Backspace` | Clear mapping                |
| `Tab`       | Next item needing review     |
| `Shift+Tab` | Previous item needing review |
| `1` `2` `3` | Switch tabs                  |
| `/`         | Focus search                 |

---

## Part 8: File Structure

```
crates/sdtm-gui/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── app.rs                 # Main eframe::App implementation
    ├── theme.rs               # Colors, spacing, fonts
    ├── state.rs               # All state types
    ├── views/
    │   ├── mod.rs
    │   ├── home.rs            # Home screen (selection + overview)
    │   ├── domain_editor.rs   # Main editor (delegates to tabs)
    │   ├── mapping_tab.rs     # Mapping tab content
    │   ├── validation_tab.rs  # Validation tab content
    │   ├── preview_tab.rs     # Preview tab content
    │   └── export.rs          # Export screen
    ├── components/
    │   ├── mod.rs
    │   ├── domain_card.rs
    │   ├── variable_list.rs
    │   ├── suggestion_card.rs
    │   ├── data_table.rs
    │   └── progress_bar.rs
    └── dialogs/
        ├── mod.rs
        └── supp_dialog.rs
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
- [ ] Unmapped columns section

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

## Part 10: Dependencies

```toml
[dependencies]
eframe = "0.29"
egui = "0.29"
egui_extras = { version = "0.29", features = ["all_loaders"] }
rfd = "0.15"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
directories = "5.0"
anyhow = "1.0"
tracing = "0.1"

sdtm-model = { path = "../sdtm-model" }
sdtm-core = { path = "../sdtm-core" }
sdtm-map = { path = "../sdtm-map" }
sdtm-standards = { path = "../sdtm-standards" }
sdtm-ingest = { path = "../sdtm-ingest" }
sdtm-validate = { path = "../sdtm-validate" }
sdtm-report = { path = "../sdtm-report" }
```

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

The four-tab design (Mapping → SUPP → Validation → Preview) follows the natural
workflow:

1. **Mapping** — Map source columns to SDTM variables
2. **SUPP** — Decide what to do with unmapped columns
3. **Validation** — Validate all mapped values against CT
4. **Preview** — See the final transformed output
