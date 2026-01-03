# SDTM Validation Rules

Trial Submission Studio validates data against SDTM implementation guide rules.

## Validation Categories

### Structural Validation

Checks data structure and format.

| Rule ID | Description               | Severity |
|---------|---------------------------|----------|
| SD0001  | Required variable missing | Error    |
| SD0002  | Invalid variable name     | Error    |
| SD0003  | Variable length exceeded  | Error    |
| SD0004  | Invalid data type         | Error    |
| SD0005  | Duplicate records         | Warning  |
| SD0006  | Invalid domain code       | Error    |

### Content Validation

Checks data values and relationships.

| Rule ID | Description                         | Severity |
|---------|-------------------------------------|----------|
| CT0001  | Value not in controlled terminology | Error    |
| CT0002  | Invalid date format                 | Error    |
| CT0003  | Date out of valid range             | Warning  |
| CT0004  | Numeric value out of range          | Warning  |
| CT0005  | Missing required value              | Error    |

### Cross-Record Validation

Checks relationships between records.

| Rule ID | Description                       | Severity |
|---------|-----------------------------------|----------|
| XR0001  | USUBJID not in DM                 | Error    |
| XR0002  | Duplicate key values              | Error    |
| XR0003  | Missing parent record             | Warning  |
| XR0004  | Inconsistent dates across domains | Warning  |

## Common Validation Rules

### Identifier Rules

#### STUDYID

- Must be present in all records
- Must be consistent across domains
- Cannot be null or empty

#### USUBJID

- Must be present in all records
- Must exist in DM domain
- Must be unique per subject

#### DOMAIN

- Must match the domain abbreviation
- Must be uppercase
- Must be 2 characters

### Date/Time Rules

#### --DTC Variables

- Must follow ISO 8601 format
- Supported formats:
    - `YYYY-MM-DDTHH:MM:SS`
    - `YYYY-MM-DD`
    - `YYYY-MM`
    - `YYYY`

#### Date Ranges

- End date cannot precede start date
- Study dates should be within study period

### Controlled Terminology Rules

#### SEX

Valid values:

- `M` (Male)
- `F` (Female)
- `U` (Unknown)
- `UNDIFFERENTIATED`

#### AESEV

Valid values:

- `MILD`
- `MODERATE`
- `SEVERE`

#### AESER

Valid values:

- `Y` (Yes)
- `N` (No)

## Validation Report

### Error Summary

```
┌─────────────────────────────────────────────────────────────┐
│ Validation Summary                                          │
├─────────────────────────────────────────────────────────────┤
│ Errors:   5                                                 │
│ Warnings: 12                                                │
│ Info:     3                                                 │
├─────────────────────────────────────────────────────────────┤
│ Domain: DM                                                  │
│   - 2 Errors                                                │
│   - 3 Warnings                                              │
│                                                             │
│ Domain: AE                                                  │
│   - 3 Errors                                                │
│   - 9 Warnings                                              │
└─────────────────────────────────────────────────────────────┘
```

### Error Details

Each error includes:

- **Rule ID**: Unique identifier
- **Severity**: Error/Warning/Info
- **Description**: What's wrong
- **Location**: Affected rows/columns
- **Suggestion**: How to fix

## Fixing Validation Issues

### Mapping Issues

1. Verify correct source column is mapped
2. Check data type compatibility
3. Ensure all required variables are mapped

### Data Issues

1. Review affected rows
2. Correct values in source data
3. Re-import and re-validate

### Terminology Issues

1. Check expected values in codelist
2. Map source values to standard terms
3. Use value-level mapping if needed

## Custom Validation

### Severity Overrides

Some warnings can be suppressed if intentional:

1. Review the warning
2. Document the reason
3. Mark as reviewed (if applicable)

### Adding Context

For validation reports:

- Add comments explaining exceptions
- Document data collection differences
- Note protocol-specific variations

## Best Practices

1. **Validate incrementally**
    - After initial mapping
    - After each significant change
    - Before final export

2. **Address errors first**
    - Errors block export
    - Warnings should be reviewed
    - Info messages are FYI

3. **Document exceptions**
    - Why a warning is acceptable
    - Protocol-specific reasons
    - Historical data limitations

4. **Review validation reports**
    - Keep for audit trail
    - Share with data management
    - Include in submission package

## Next Steps

- [Controlled Terminology](../controlled-terminology.md) - Valid values
- [Exporting Data](../../user-guide/exporting-data.md) - Export after validation
