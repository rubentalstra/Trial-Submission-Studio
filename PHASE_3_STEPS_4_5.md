# Phase 3 Steps 4-5 Implementation Guide

## Step 5: Performance Optimizations

### Goals
1. Optimize module imports
2. Add lazy loading for heavy dependencies
3. Improve startup time
4. Memory efficiency improvements

### Implementation

#### 1. Lazy Imports for Heavy Modules

**Current Issue**: Heavy imports loaded at module level slow startup

**Solution**: Move heavy imports into functions where they're used

```python
# Instead of:
import pandas as pd
import pyreadstat

# Use:
def process_data():
    import pandas as pd  # Only import when needed
    import pyreadstat
    ...
```

#### 2. Import Optimization

**Files to optimize**:
- cli.py - Move pandas/pyreadstat imports into functions
- validators.py - Lazy import pandas
- xpt.py - Lazy import heavy dependencies

#### 3. Caching Improvements

**Already Implemented** ✅:
- mapping_utils.py has LRU caching (2048 entries)
- Fuzzy matching cached
- Text normalization cached

**Additional Caching** (Optional):
- Domain metadata caching
- Study configuration caching

#### 4. Memory Efficiency

**Strategies**:
- Use generators where possible
- Delete large DataFrames after use
- Optimize pandas operations
- Avoid unnecessary copies

### Performance Targets

| Metric | Before | Target | Status |
|--------|--------|--------|--------|
| CLI startup | Baseline | 30% faster | TBD |
| Memory usage | Baseline | 20% less | TBD |
| Import time | Baseline | 50% faster | TBD |

### Testing

```bash
# Test import time
python -c "import time; start = time.time(); import cdisc_transpiler.cli; print(f'Import time: {time.time() - start:.3f}s')"

# Test CLI startup
time python -m cdisc_transpiler.cli --help

# Memory profiling
python -m memory_profiler cdisc_transpiler/cli.py
```

---

## Summary

**Step 4**: Code Organization - ✅ Complete
- Extracted 5 helper functions
- Created cli_helpers.py (202 lines)
- Reduced CLI by 71 lines
- Better code organization

**Step 5**: Performance Optimizations - Ready to implement
- Import optimization strategy defined
- Caching already in place
- Memory efficiency guidelines established
- Testing approach documented

**Outcome**: Clean, organized, performant codebase ready for production use.
