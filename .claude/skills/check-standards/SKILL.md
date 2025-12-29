---
name: check-standards
description: Look up SDTM and controlled terminology standards from offline CSV files. Use when checking domain specifications, variable definitions, CT codelists, or verifying SDTM requirements.
---

# SDTM Standards Lookup Skill

## Purpose

This skill helps navigate and query the offline CDISC standards committed in the `standards/` directory.

## When to Use

- Looking up SDTM domain specifications
- Finding variable definitions and their CT codelists
- Verifying controlled terminology values
- Checking SDTM Implementation Guide requirements
- Understanding core designation (Required/Expected/Permissible)

## Standards Directory Structure

```
standards/
├── ct/                          # Controlled Terminology by version
│   ├── 2023-09-29/
│   │   └── CDISC_CT.csv        # All CT terms and codelists
│   └── 2024-06-28/
│       └── CDISC_CT.csv
├── sdtmig/v3_4/
│   ├── Datasets.csv             # Domain metadata (code, class, label)
│   ├── Variables.csv            # Variable definitions with CT codes
│   └── chapters/                # SDTMIG v3.4 documentation
│       ├── Chapter1.md
│       ├── Chapter2.md          # General Observations Classes
│       ├── Chapter3.md          # Interventions
│       ├── Chapter4.md          # Events
│       ├── Chapter5.md          # Findings
│       ├── Chapter6.md          # Special Purpose
│       └── Chapter7.md          # Relationships
└── sdtm/                        # SDTM model specs
```

## Common Lookup Tasks

### 1. Find Domain Specification

```bash
# Search for domain in Datasets.csv
grep -i "^DM," standards/sdtmig/v3_4/Datasets.csv
```

Or use Read tool on `standards/sdtmig/v3_4/Datasets.csv`

### 2. Find Variable Definition

```bash
# Search Variables.csv for specific variable
grep "USUBJID" standards/sdtmig/v3_4/Variables.csv
```

Variables.csv columns:
- `Variable`: Variable name (e.g., USUBJID, STUDYID)
- `Label`: Human-readable label
- `Data Type`: text, integer, float, datetime
- `Role`: Identifier, Topic, Qualifier, Timing
- `Core`: Req (Required), Exp (Expected), Perm (Permissible)
- `Codelist Code`: CT codelist code (e.g., C66731 for SEX)

### 3. Look Up Controlled Terminology

```bash
# Find all terms in a codelist (e.g., C66731 for SEX)
grep "C66731" standards/ct/2024-06-28/CDISC_CT.csv
```

CDISC_CT.csv columns:
- `Code`: Internal code (e.g., C66731)
- `Codelist Name`: Human name (e.g., "Sex")
- `Term Code`: Term's internal code
- `Submission Value`: Value to use in SDTM (e.g., "M", "F", "U")
- `Synonyms`: Alternative terms (pipe-separated)
- `Definition`: Term definition

### 4. Check SDTMIG Chapter Documentation

Before implementing domain-specific rules, read the relevant chapter:

- **DM (Demographics)**: Chapter 2 (General Observations)
- **EX (Exposure)**: Chapter 3 (Interventions)
- **AE (Adverse Events)**: Chapter 4 (Events)
- **VS (Vital Signs)**: Chapter 5 (Findings)
- **SUPPQUAL**: Chapter 6 (Special Purpose)

Use Read tool on `standards/sdtmig/v3_4/chapters/Chapter*.md`

## Verification Workflow

When implementing SDTM rules:

1. **Check Variables.csv** - Verify variable is defined for the domain
2. **Check Core designation** - Determine if Req/Exp/Perm
3. **Check Codelist Code** - If present, variable uses CT
4. **Check CDISC_CT.csv** - Find valid submission values for codelist
5. **Read chapter docs** - Verify implementation requirements

## Example Queries

### Find all variables for DM domain
```bash
grep "^DM," standards/sdtmig/v3_4/Variables.csv
```

### Find valid values for ETHNIC variable (codelist C66790)
```bash
grep "C66790" standards/ct/2024-06-28/CDISC_CT.csv | grep -i "submission"
```

### Check if variable requires CT normalization
1. Look up variable in Variables.csv
2. If "Codelist Code" column has value → uses CT
3. Find codelist in CDISC_CT.csv
4. Implement normalization in domain processor

## Key Principles

- **Offline-first**: All standards are committed; never fetch external data
- **Version-specific**: Use correct CT version (check `standards/ct/`)
- **Case-insensitive**: SDTM variable matching is case-insensitive
- **Authority**: Chapter documentation is source of truth for requirements

## Related Files

- `crates/sdtm-standards/src/` - Code that loads these CSVs
- `crates/sdtm-model/src/` - Types representing these standards
- `docs/NAMING_CONVENTIONS.md` - Terminology mapping conventions
