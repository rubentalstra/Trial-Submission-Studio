# Migration Guide: xpt.py → xpt_module

This guide helps you migrate from the deprecated `xpt.py` module to the new modular `xpt_module` architecture.

## Quick Migration

### Basic Usage

**Old way** (deprecated):
```python
from cdisc_transpiler.xpt import build_domain_dataframe, write_xpt_file

df = build_domain_dataframe(source_df, config)
write_xpt_file(df, "DM", "output/dm.xpt")
```

**New way** (recommended):
```python
from cdisc_transpiler.xpt_module import build_domain_dataframe, write_xpt_file

df = build_domain_dataframe(source_df, config)
write_xpt_file(df, "DM", "output/dm.xpt")
```

### Advanced Usage

The new modular structure also provides direct access to transformers and validators:

```python
from cdisc_transpiler.xpt_module import (
    build_domain_dataframe,
    write_xpt_file,
)
from cdisc_transpiler.xpt_module.transformers import (
    DateTransformer,
    CodelistTransformer,
    NumericTransformer,
    TextTransformer,
)
from cdisc_transpiler.xpt_module.validators import XPTValidator

# Use transformers independently
DateTransformer.normalize_dates(df, domain.variables)
CodelistTransformer.apply_codelist_validations(df, domain.variables)

# Use validators independently
XPTValidator.enforce_lengths(df, domain.variables)
XPTValidator.reorder_columns(df, domain.variables)
```

## What Changed?

### Module Structure

**Old structure**:
- Single 3,124-line `xpt.py` file
- All logic mixed together
- Hard to test and maintain

**New structure**:
```
xpt_module/
├── __init__.py              # Public API
├── writer.py                # XPT file writing (95 lines)
├── builder.py               # DataFrame construction (263 lines)
├── validators.py            # Validation logic (180 lines)
└── transformers/
    ├── __init__.py
    ├── date.py              # Date/time transformations (220 lines)
    ├── codelist.py          # Controlled terminology (240 lines)
    ├── numeric.py           # Numeric transformations (90 lines)
    └── text.py              # Text transformations (70 lines)
```

### Benefits

1. **Better Organization**: Each module has a single responsibility
2. **Easier Testing**: Can test transformers independently
3. **Reusability**: Use transformers in other contexts
4. **Maintainability**: Find and fix issues faster
5. **Clarity**: Clear separation of concerns

## Migration Steps

### Step 1: Update Imports

**In your code files**:
```python
# Replace this:
from cdisc_transpiler.xpt import build_domain_dataframe, write_xpt_file

# With this:
from cdisc_transpiler.xpt_module import build_domain_dataframe, write_xpt_file
```

### Step 2: Test Your Code

The API is identical, so your code should work without changes:

```python
# This still works the same way
df = build_domain_dataframe(
    source_frame,
    config,
    reference_starts=ref_starts,
    lenient=False,
    metadata=study_metadata,
)

write_xpt_file(df, domain_code, output_path)
```

### Step 3: Suppress Warnings (Optional)

If you're still using the old imports temporarily:

```python
import warnings
warnings.filterwarnings('ignore', category=DeprecationWarning, module='cdisc_transpiler.xpt')
```

## Advanced Patterns

### Using Transformers Directly

You can now use transformers independently for custom processing:

```python
from cdisc_transpiler.xpt_module.transformers import DateTransformer
import pandas as pd

# Transform dates in any DataFrame
df = pd.DataFrame({
    'AESTDTC': ['2024-01-15', '2024-02-20'],
    'AEENDTC': ['2024-01-20', '2024-02-25'],
})

# Use date transformer
DateTransformer.ensure_date_pair_order(df, 'AESTDTC', 'AEENDTC')
print(df)
```

### Using Validators Directly

```python
from cdisc_transpiler.xpt_module.validators import XPTValidator

# Validate required values
missing = XPTValidator.validate_required_values(df, domain.variables)
if missing:
    print(f"Missing required values in: {missing}")

# Enforce lengths
XPTValidator.enforce_lengths(df, domain.variables)
```

### Custom Processing Pipeline

Build custom pipelines using modular components:

```python
from cdisc_transpiler.xpt_module.transformers import (
    DateTransformer,
    NumericTransformer,
    TextTransformer,
)

def custom_domain_processor(df, domain):
    """Custom processing pipeline using modular transformers."""
    # Normalize dates
    DateTransformer.normalize_dates(df, domain.variables)
    
    # Clean up visits
    TextTransformer.normalize_visit(df)
    
    # Force numeric types
    if 'VISITNUM' in df.columns:
        df['VISITNUM'] = NumericTransformer.force_numeric(df['VISITNUM'])
    
    return df
```

## Compatibility

### Backward Compatibility

The old `xpt.py` module is maintained for backward compatibility:

- ✅ All existing code continues to work
- ✅ No breaking changes
- ⚠️ Deprecation warnings are shown
- 📋 Plan to remove in future major version

### Forward Compatibility

New code should use `xpt_module`:

- ✅ Cleaner API
- ✅ Better organization
- ✅ More flexibility
- ✅ Easier to test and maintain

## Common Issues

### Issue: Deprecation Warnings

**Problem**: You see warnings like:
```
DeprecationWarning: xpt.build_domain_dataframe is deprecated.
Use cdisc_transpiler.xpt_module.build_domain_dataframe instead.
```

**Solution**: Update your imports as shown above.

### Issue: Import Errors

**Problem**: `ImportError: cannot import name 'DateTransformer'`

**Solution**: Make sure you're importing from the right place:
```python
from cdisc_transpiler.xpt_module.transformers import DateTransformer
```

### Issue: Different Behavior

**Problem**: Output differs from old implementation

**Solution**: This shouldn't happen - the API is identical. Please report as a bug.

## FAQ

### Q: Do I need to migrate immediately?

**A**: No. The old `xpt.py` is maintained for backward compatibility. However, new code should use `xpt_module`.

### Q: Will my XPT files be different?

**A**: No. The output is byte-identical. Only the internal structure changed.

### Q: Can I use both old and new APIs?

**A**: Yes, but not recommended. Choose one approach for consistency.

### Q: What if I only need part of the functionality?

**A**: The new modular structure lets you import only what you need:
```python
from cdisc_transpiler.xpt_module.transformers import DateTransformer
# Use only DateTransformer, no other dependencies
```

### Q: When will the old xpt.py be removed?

**A**: Not before a major version bump. You'll have plenty of warning.

## Examples

### Example 1: Simple Domain Processing

```python
from cdisc_transpiler.xpt_module import build_domain_dataframe, write_xpt_file
from cdisc_transpiler.mapping import MappingConfig
import pandas as pd

# Source data
source_df = pd.DataFrame({
    'subject_id': ['001', '002', '003'],
    'birth_date': ['1980-01-15', '1975-06-20', '1990-03-10'],
})

# Mapping configuration
config = MappingConfig(
    domain='DM',
    study_id='STUDY001',
    mappings=[...],
)

# Build domain DataFrame
dm_df = build_domain_dataframe(source_df, config)

# Write XPT file
write_xpt_file(dm_df, 'DM', 'output/dm.xpt')
```

### Example 2: Custom Validation

```python
from cdisc_transpiler.xpt_module.validators import XPTValidator
from cdisc_transpiler.domains import get_domain

# Get domain definition
domain = get_domain('AE')

# Validate your DataFrame
missing_vars = XPTValidator.validate_required_values(df, domain.variables)
if missing_vars:
    print(f"Warning: Missing required values in {missing_vars}")

# Enforce constraints
XPTValidator.enforce_lengths(df, domain.variables)
XPTValidator.enforce_required_values(df, domain.variables, lenient=False)
```

### Example 3: Date Processing

```python
from cdisc_transpiler.xpt_module.transformers import DateTransformer
import pandas as pd

df = pd.DataFrame({
    'USUBJID': ['SUBJ-001', 'SUBJ-002'],
    'AESTDTC': ['2024-01-15', '2024-02-20'],
    'RFSTDTC': ['2024-01-01', '2024-02-01'],
})

reference_starts = {
    'SUBJ-001': '2024-01-01',
    'SUBJ-002': '2024-02-01',
}

# Calculate study days
from cdisc_transpiler.domains import get_domain
domain = get_domain('AE')

DateTransformer.calculate_dy(df, domain.variables, reference_starts)
print(df[['USUBJID', 'AESTDTC', 'AESTDY']])
```

## Support

For questions or issues:
1. Check this migration guide
2. Review `PHASE_4_XPT_REFACTORING.md` for technical details
3. Check the module docstrings
4. Report issues on GitHub

## Summary

- ✅ Update imports from `xpt` to `xpt_module`
- ✅ Test your code (should work unchanged)
- ✅ Enjoy better code organization
- ✅ Use transformers independently when needed
- ✅ Old code continues to work (with warnings)

The migration is straightforward: just update your imports!
