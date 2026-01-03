# Exporting Data

After mapping and validation, export your data to CDISC-compliant formats.

<!-- TODO: Add screenshot of export dialog -->
<!-- ![Export Dialog](../images/screenshots/export-dialog.png) -->

## Export Formats

Trial Submission Studio supports multiple output formats:

| Format          | Version | Description                  | Use Case            |
|-----------------|---------|------------------------------|---------------------|
| **XPT**         | V5      | SAS Transport (FDA standard) | FDA submissions     |
| **XPT**         | V8      | Extended SAS Transport       | Longer names/labels |
| **Dataset-XML** | 1.0     | CDISC XML format             | Data exchange       |
| **Define-XML**  | 2.1     | Metadata documentation       | Submission package  |

## XPT Export

### XPT Version 5 (Default)

The FDA standard format with these constraints:

- Variable names: 8 characters max
- Labels: 40 characters max
- Compatible with SAS V5 Transport

### XPT Version 8

Extended format supporting:

- Variable names: 32 characters
- Labels: 256 characters
- Note: Not all systems support V8

### Export Steps

1. Click **Export** in the toolbar
2. Select **XPT V5** or **XPT V8**
3. Choose output location
4. Click **Save**

### XPT Options

| Option                    | Description                         |
|---------------------------|-------------------------------------|
| **Include all variables** | Export mapped and derived variables |
| **Sort by keys**          | Order rows by key variables         |
| **Compress**              | Reduce file size                    |

## Dataset-XML Export

CDISC ODM-based XML format for data exchange.

### Features

- Human-readable format
- Full Unicode support
- Metadata included
- Schema validation

### Export Steps

1. Click **Export**
2. Select **Dataset-XML**
3. Configure options
4. Click **Save**

## Define-XML Export

Generate submission metadata documentation.

### Define-XML 2.1

- Dataset definitions
- Variable metadata
- Controlled terminology
- Computational methods
- Value-level metadata

### Export Steps

1. Click **Export**
2. Select **Define-XML**
3. Review metadata
4. Click **Save**

## Batch Export

Export multiple domains at once:

1. **File → Batch Export**
2. Select domains to export
3. Choose format(s)
4. Set output directory
5. Click **Export All**

## Export Validation

Before export completes, the system verifies:

- All required variables are present
- Data types are correct
- Lengths don't exceed limits
- Controlled terms are valid

## Output Files

### File Naming

Default naming convention:

- `{domain}.xpt` - e.g., `dm.xpt`, `ae.xpt`
- `{domain}.xml` - for Dataset-XML
- `define.xml` - for Define-XML

### Checksums

Each export generates:

- SHA256 checksum file (`.sha256`)
- Useful for submission verification

## Quality Checks

### Post-Export Verification

1. Open the exported file in a viewer
2. Verify row counts match
3. Check variable order
4. Review sample values

### External Validation

Consider validating with:

- Pinnacle 21 Community
- SAS (if available)
- Other CDISC validators

## Best Practices

1. **Validate before export** - Fix all errors first
2. **Use XPT V5 for FDA** - Standard format
3. **Generate checksums** - For integrity verification
4. **Test with validators** - Confirm compliance
5. **Keep source files** - Maintain audit trail

## Troubleshooting

### Export Fails

| Issue             | Solution                 |
|-------------------|--------------------------|
| Validation errors | Fix errors before export |
| Disk full         | Free up space            |
| Permission denied | Check write permissions  |
| File in use       | Close file in other apps |

### Output Issues

| Issue            | Solution            |
|------------------|---------------------|
| Truncated values | Check length limits |
| Missing data     | Verify mappings     |
| Wrong encoding   | Ensure UTF-8 source |

## Next Steps

- [Common Workflows](workflows.md) - End-to-end examples
- [XPT Format](../output-formats/xpt-format.md) - XPT specification
- [Define-XML](../output-formats/define-xml.md) - Define-XML guide
