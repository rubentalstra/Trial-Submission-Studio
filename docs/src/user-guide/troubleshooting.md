# Troubleshooting

Common issues and their solutions when using Trial Submission Studio.

## Import Issues

### File Won't Import

| Symptom               | Cause                         | Solution                  |
|-----------------------|-------------------------------|---------------------------|
| "Invalid file format" | Not a CSV file                | Ensure file is CSV format |
| "Encoding error"      | Non-UTF8 encoding             | Re-save as UTF-8          |
| "No data found"       | Empty file or wrong delimiter | Check file contents       |
| "Parse error"         | Malformed CSV                 | Fix CSV structure         |

### Data Appears Garbled

**Cause**: Encoding mismatch

**Solution**:

1. Open the file in a text editor
2. Save with UTF-8 encoding
3. Re-import

### Missing Columns

**Cause**: Header row issues

**Solution**:

1. Verify first row contains headers
2. Check for BOM (byte order mark) issues
3. Remove hidden characters

---

## Mapping Issues

### No Suggested Mappings

**Cause**: Column names don't match SDTM variables

**Solution**:

1. Manually map columns
2. Consider renaming source columns
3. Create a mapping template for reuse

### Wrong Automatic Mappings

**Cause**: Fuzzy matching misidentified variables

**Solution**:

1. Review all automatic mappings
2. Manually correct incorrect mappings
3. Adjust match confidence threshold in settings

### Can't Map Required Variable

**Cause**: Source data missing required information

**Solution**:

1. Add the missing data to source file
2. Derive from other columns if possible
3. Consult with data manager

---

## Validation Issues

### Too Many Errors

**Cause**: Data quality issues or incorrect mappings

**Solution**:

1. Address errors in priority order
2. Fix mapping issues first
3. Clean source data if needed
4. Re-validate after each fix

### Controlled Terminology Errors

**Cause**: Values don't match CDISC CT

**Solution**:

1. Review expected values in the error message
2. Map source values to standard terms
3. Update source data if appropriate

### Date Format Errors

**Cause**: Non-ISO date formats

**Solution**:

1. Convert dates to ISO 8601 format (YYYY-MM-DD)
2. Or use partial dates where appropriate (YYYY-MM, YYYY)

---

## Export Issues

### Export Fails

| Error                     | Cause               | Solution                 |
|---------------------------|---------------------|--------------------------|
| "Validation errors exist" | Unresolved errors   | Fix all errors first     |
| "Permission denied"       | No write access     | Check folder permissions |
| "Disk full"               | Insufficient space  | Free up disk space       |
| "File in use"             | File open elsewhere | Close file in other apps |

### Truncated Data in XPT

**Cause**: Values exceed XPT limits

**Solution**:

1. XPT V5: Max 200 chars per variable
2. Check variable lengths before export
3. Consider using XPT V8 for longer values

### Missing Variables in Output

**Cause**: Variables not mapped or derived

**Solution**:

1. Verify all required mappings
2. Check if derived variables were created
3. Review export settings

---

## Performance Issues

### Slow Import

**Cause**: Large file size

**Solution**:

1. Allow time for large files
2. Consider splitting into smaller files
3. Close other applications
4. Increase available RAM

### Application Freezes

**Cause**: Processing large datasets

**Solution**:

1. Wait for operation to complete
2. Check progress indicator
3. If unresponsive after 5+ minutes, restart
4. Process smaller datasets

### High Memory Usage

**Cause**: Large dataset in memory

**Solution**:

1. Close unused files
2. Process one domain at a time
3. Restart application to free memory

---

## Application Issues

### Application Won't Start

**macOS**:

```bash
# If blocked by Gatekeeper
xattr -d com.apple.quarantine /Applications/Trial\ Submission\ Studio.app
```

**Linux**:

```bash
# Ensure executable permission
chmod +x trial-submission-studio
```

**Windows**:

- Run as administrator
- Check antivirus isn't blocking

### Crashes on Startup

**Solution**:

1. Delete configuration files:
    - macOS: `~/Library/Application Support/trial-submission-studio/`
    - Windows: `%APPDATA%\trial-submission-studio\`
    - Linux: `~/.config/trial-submission-studio/`
2. Reinstall the application

### Settings Not Saved

**Cause**: Permission issues

**Solution**:

1. Ensure write access to config directory
2. Run application with appropriate permissions

---

## Getting Help

### Collect Information

Before reporting an issue, gather:

1. Application version (Help → About)
2. Operating system and version
3. Steps to reproduce
4. Error messages (screenshots)
5. Sample data (anonymized)

### Report an Issue

1. Check [existing issues](https://github.com/rubentalstra/Trial-Submission-Studio/issues)
2. Create a [new issue](https://github.com/rubentalstra/Trial-Submission-Studio/issues/new)
3. Include collected information

### Community Support

- [GitHub Discussions](https://github.com/rubentalstra/Trial-Submission-Studio/discussions)

---

## Quick Reference

### Keyboard Shortcuts for Recovery

| Action           | Windows/Linux | macOS |
|------------------|---------------|-------|
| Force quit       | Alt+F4        | ⌘Q    |
| Cancel operation | Esc           | Esc   |
| Undo             | Ctrl+Z        | ⌘Z    |

### Log Files

Application logs are located at:

- **macOS**: `~/Library/Logs/trial-submission-studio/`
- **Windows**: `%LOCALAPPDATA%\trial-submission-studio\logs\`
- **Linux**: `~/.local/share/trial-submission-studio/logs/`

Include relevant log excerpts when reporting issues.
