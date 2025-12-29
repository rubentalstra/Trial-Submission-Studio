---
name: validate-sdtm
description: Run SDTM validation checks and analyze conformance issues. Use when validating SDTM outputs, debugging validation errors, or checking domain conformance.
---

# SDTM Validation Skill

## Purpose

This skill helps validate SDTM datasets against conformance rules and analyze validation issues.

## When to Use

- Validating converted SDTM datasets
- Debugging validation errors or warnings
- Checking controlled terminology conformance
- Analyzing why validation is blocking XPT output

## Validation Commands

```bash
# Run full pipeline with validation
cargo run -- -s path/to/study -o output/

# Run validation tests
cargo test --package sdtm-validate

# Run specific domain validation tests
cargo test --package sdtm-validate test_name
```

## Understanding Validation Issues

### Severity Levels

1. **Error** - Blocks XPT output (gating)
2. **Warning** - Reported but doesn't block output
3. **Info** - Informational only

### Common Validation Issue Codes

- `CT_INVALID_VALUE` - Value not in controlled terminology codelist
- `REQUIRED_VARIABLE_MISSING` - Required SDTM variable missing
- `INVALID_DATA_TYPE` - Variable has wrong data type
- `EXPECTED_VARIABLE_MISSING` - Expected variable missing (warning)

## Key Files

```
crates/sdtm-validate/src/
├── lib.rs                    # Main validation entry point
├── ct_validator.rs           # CT conformance checks
├── required_variables.rs     # Required variable checks
└── validators/               # Domain-specific validators
```

## Validation Workflow

1. **Read validation output** - Check console for ValidationIssue messages
2. **Identify severity** - Focus on Errors first (block output)
3. **Locate source** - Find which domain/variable has the issue
4. **Check standards** - Look up requirement in `standards/sdtmig/v3_4/`
5. **Fix root cause** - Update domain processor or mapping logic

## Debugging Tips

- Check if CT value needs normalization in domain processor
- Verify variable is in domain's expected_vars set
- Review SDTM spec in `standards/sdtmig/v3_4/chapters/` for requirement
- Look for case-sensitivity issues (use CaseInsensitiveSet)

## Example Analysis

When you see:
```
Error [CT_INVALID_VALUE] in DM.SEX: value "M" not in codelist C66731
```

Steps:
1. Check `standards/ct/*/CDISC_CT.csv` for codelist C66731
2. Find valid submission values (likely "M", "F", "U")
3. Verify value is normalized in `crates/sdtm-core/src/domain_processors/dm.rs`
4. Check if case-insensitive matching is needed

## Related Documentation

- See `docs/NAMING_CONVENTIONS.md` for terminology mapping
- See `standards/sdtmig/v3_4/chapters/` for SDTM requirements
- See validation crate at `crates/sdtm-validate/`
