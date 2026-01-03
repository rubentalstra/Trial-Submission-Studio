# Quick Start Guide

Get up and running with Trial Submission Studio in 5 minutes.

## Overview

This guide walks you through the basic workflow:

1. **Import** your source CSV data
2. **Map** columns to SDTM variables
3. **Validate** against CDISC standards
4. **Export** to XPT format

---

## Step 1: Launch the Application

After [installing](installation.md) Trial Submission Studio, launch the
application:

- **macOS**: Open from Applications folder
- **Windows**: Run `trial-submission-studio.exe`
- **Linux**: Run `./trial-submission-studio`

You'll see the main interface with the data import panel.

---

## Step 2: Import Your Data

1. Click **Import CSV** or drag and drop your CSV file
2. Trial Submission Studio will automatically:
   - Detect column types
   - Identify potential SDTM domains
   - Parse date formats

> [!TIP]
> Your data should have column headers in the first row.

---

## Step 3: Select SDTM Domain

1. Choose the target SDTM domain (e.g., DM, AE, VS)
2. The application will display the required and optional variables for that
   domain

---

## Step 4: Map Columns

1. Review the suggested column mappings
2. For each source column, select the corresponding SDTM variable
3. Use the fuzzy matching suggestions to speed up mapping

The mapping interface shows:

- **Source Column**: Your CSV column name
- **Target Variable**: The SDTM variable
- **Match Score**: Confidence of the suggested mapping

---

## Step 5: Validate

1. Click **Validate** to check your data against CDISC rules
2. Review any validation messages:
   - **Errors**: Must be fixed before export
   - **Warnings**: Should be reviewed
   - **Info**: Informational messages

---

## Step 6: Export

1. Click **Export**
2. Choose your output format:
   - **XPT V5** (FDA standard)
   - **XPT V8** (extended names/labels)
   - **Dataset-XML** (CDISC data exchange)
3. Select the output location
4. Click **Save**

---

## Next Steps

Now that you've completed the basic workflow:

- [Interface Overview](../user-guide/ui-overview.md) - Learn about all features
- [Column Mapping](../user-guide/column-mapping.md) - Advanced mapping
  techniques
- [Validation](../user-guide/validation.md) - Understanding validation rules
- [SDTM Standards](../cdisc-standards/sdtm/introduction.md) - SDTM reference
  guide
