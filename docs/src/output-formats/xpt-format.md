# XPT (SAS Transport) Format

XPT is the FDA-standard format for regulatory data submissions.

## Overview

The SAS Transport Format (XPT) is:

- Required by FDA for electronic submissions
- A platform-independent binary format
- Compatible with SAS and other tools
- The de facto standard for clinical data exchange

## XPT Versions

Trial Submission Studio supports two XPT versions:

### XPT Version 5 (FDA Standard)

| Characteristic        | Limit          |
|-----------------------|----------------|
| Variable name length  | 8 characters   |
| Variable label length | 40 characters  |
| Record length         | 8,192 bytes    |
| Numeric precision     | 8 bytes (IEEE) |

**Use for**: FDA submissions, regulatory requirements

### XPT Version 8 (Extended)

| Characteristic        | Limit          |
|-----------------------|----------------|
| Variable name length  | 32 characters  |
| Variable label length | 256 characters |
| Record length         | 131,072 bytes  |
| Numeric precision     | 8 bytes (IEEE) |

**Use for**: Internal use, longer names needed

## File Structure

### Header Records

XPT files contain metadata headers:

- Library header (first record)
- Member header (dataset info)
- Namestr records (variable definitions)

### Data Records

- Fixed-width records
- Packed binary format
- IEEE floating-point numbers

## Creating XPT Files

### Export Steps

1. Complete data mapping
2. Run validation
3. Click **Export → XPT**
4. Select version (V5 or V8)
5. Choose output location
6. Click **Save**

### Export Options

| Option           | Description                    |
|------------------|--------------------------------|
| Version          | V5 (default) or V8             |
| Sort by keys     | Order records by key variables |
| Include metadata | Dataset label, variable labels |

## XPT Constraints

### Variable Names

**V5 Requirements**:

- Maximum 8 characters
- Start with letter or underscore
- Alphanumeric and underscore only
- Uppercase recommended

**V8 Requirements**:

- Maximum 32 characters
- Same character restrictions

### Variable Labels

**V5**: 40 characters max
**V8**: 256 characters max

### Data Values

**Character variables**:

- V5: Max 200 bytes per value
- Trailing spaces trimmed
- Missing = blank

**Numeric variables**:

- 8-byte IEEE format
- 28 SAS missing value codes supported (.A through .Z, ._)
- Precision: ~15 significant digits

## Numeric Precision

### IEEE to SAS Conversion

Trial Submission Studio handles:

- IEEE 754 double precision
- SAS missing value encoding
- Proper byte ordering

### Missing Values

SAS/XPT supports 28 missing value codes:

| Code        | Meaning             |
|-------------|---------------------|
| `.`         | Standard missing    |
| `.A` - `.Z` | Special missing A-Z |
| `._`        | Underscore missing  |

## Validation Before Export

### Automatic Checks

- Variable name lengths
- Label lengths
- Data type compatibility
- Value length limits

### Common Issues

| Issue           | Solution          |
|-----------------|-------------------|
| Name too long   | Use V8 or rename  |
| Label truncated | Shorten label     |
| Value too long  | Truncate or split |

## Post-Export Verification

### Recommended Steps

1. **Check file size** - Matches expected data volume
2. **Open in viewer** - Verify structure
3. **Validate with external tools** - Pinnacle 21, SAS
4. **Compare row counts** - Match source data

### External Validation

Consider validating with:

- Pinnacle 21 Community (free)
- SAS Universal Viewer
- Other XPT readers

## FDA Submission Requirements

### Required Format

- XPT Version 5 for FDA submissions
- Define-XML 2.1 for metadata
- Appropriate file naming (lowercase domain codes)

### File Naming Convention

- `dm.xpt` - Demographics
- `ae.xpt` - Adverse Events
- `vs.xpt` - Vital Signs
- (lowercase domain abbreviation)

### Dataset Limits

| Constraint            | Limit                  |
|-----------------------|------------------------|
| File size             | 5 GB (practical limit) |
| Variables per dataset | No formal limit        |
| Records per dataset   | No formal limit        |

## Technical Details

### Byte Order

- XPT uses big-endian byte order
- Trial Submission Studio handles conversion automatically

### Character Encoding

- ASCII-compatible
- Extended ASCII for special characters
- UTF-8 source data converted appropriately

### Record Blocking

- 80-byte logical records
- Blocked for efficiency
- Headers use fixed-format records

## Next Steps

- [Dataset-XML](dataset-xml.md) - Alternative export format
- [Define-XML](define-xml.md) - Metadata documentation
- [Exporting Data](../user-guide/exporting-data.md) - Export guide
