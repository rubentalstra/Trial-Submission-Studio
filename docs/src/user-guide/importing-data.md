# Importing Data

Trial Submission Studio accepts CSV files as input and automatically detects schema information.

<!-- TODO: Add screenshot of import dialog -->
<!-- ![Import Dialog](../images/screenshots/import-dialog.png) -->

## Supported Input Format

Currently, Trial Submission Studio supports:

- **CSV files** (`.csv`)
- UTF-8 or ASCII encoding
- Comma-separated values
- Headers in first row

## Import Methods

### Drag and Drop

Simply drag a CSV file from your file manager and drop it onto the application window.

### File Menu

1. Click **File → Import CSV**
2. Navigate to your file
3. Click **Open**

### Toolbar Button

Click the **Import** button in the toolbar.

## Automatic Detection

When you import a file, Trial Submission Studio automatically:

### Column Type Detection

Analyzes sample values to determine:

- **Numeric** - Integer or floating-point numbers
- **Date/Time** - Various date formats
- **Text** - Character strings

### Domain Suggestion

Based on column names, suggests likely SDTM domains:

- `USUBJID`, `AGE`, `SEX` → Demographics (DM)
- `AETERM`, `AESTDTC` → Adverse Events (AE)
- `VSTESTCD`, `VSSTRESN` → Vital Signs (VS)

### Date Format Detection

Automatically recognizes common date formats:

- ISO 8601: `2024-01-15`
- US format: `01/15/2024`
- EU format: `15-01-2024`
- With time: `2024-01-15T09:30:00`

## Data Preview

After import, you'll see:

### Data Grid

- First 100 rows displayed
- Scroll to view more data
- Column headers with detected types

### Summary Panel

- Total row count
- Total column count
- File size
- Encoding detected

### Column Information

- Column name
- Detected type
- Sample values
- Null count

## Handling Issues

### Encoding Problems

If you see garbled characters:

1. Ensure your file is UTF-8 encoded
2. Re-save from your source application with UTF-8 encoding

### Missing Headers

If your CSV lacks headers:

1. Add a header row to your file
2. Re-import

### Large Files

For files with millions of rows:

- Import may take longer
- A progress indicator will show status
- Consider splitting into smaller files if needed

## Best Practices

1. **Clean your data** before import
    - Remove trailing whitespace
    - Standardize date formats
    - Check for encoding issues

2. **Use descriptive column names**
    - Helps with automatic mapping suggestions
    - Use SDTM-like naming when possible

3. **Include all required data**
    - USUBJID for subject identification
    - Domain-specific required variables

## Next Steps

- [Column Mapping](column-mapping.md) - Map imported columns to SDTM variables
- [Validation](validation.md) - Validate your data
