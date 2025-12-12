# Phase 4: XPT Module Refactoring - Complete Guide

## Executive Summary

Phase 4 transforms the massive 3,124-line xpt.py monolithic file into a clean, modular architecture with focused responsibilities. This addresses one of the largest code quality issues in the codebase.

---

## Problem Statement

### Current State (xpt.py)
- **3,124 lines** in a single file
- **30+ methods** in _DomainFrameBuilder class
- **Mixed responsibilities**: Building, transforming, validating, normalizing, writing
- **High complexity**: Difficult to understand and maintain
- **Hard to test**: Monolithic structure prevents isolated testing
- **Code duplication**: Similar patterns repeated

### Issues
1. **Massive Class**: _DomainFrameBuilder has too many responsibilities
2. **Mixed Concerns**: Building, validation, transformation all mixed
3. **Hard to Navigate**: Finding specific logic requires scrolling through 3,000+ lines
4. **Difficult to Test**: Can't test transformations independently
5. **Code Duplication**: Similar date/codelist logic repeated

---

## Solution: Modular Architecture

### Target Structure
```
xpt_module/
├── __init__.py (50 lines)
│   └── Public API with clean exports
├── writer.py (95 lines)
│   └── XPT file writing with pyreadstat
├── builder.py (~500 lines)
│   └── DataFrame construction orchestration
├── transformers/
│   ├── __init__.py (30 lines)
│   ├── date.py (~300 lines)
│   │   └── Date/time/duration transformations
│   ├── codelist.py (~400 lines)
│   │   └── Controlled terminology application
│   ├── numeric.py (~150 lines)
│   │   └── Numeric transformations
│   └── text.py (~50 lines)
│       └── Text transformations
└── validators.py (~300 lines)
    └── XPT-specific validation

Total: ~1,845 lines across 9 focused modules
Average: 205 lines per module (vs 3,124 monolith)
```

---

## Implementation Steps

### ✅ Step 1: Package & Writer (COMPLETE)

**Created**:
- `xpt_module/__init__.py` (50 lines) - Public API
- `xpt_module/writer.py` (95 lines) - XPT writing

**Extracted From xpt.py**:
- `write_xpt_file()` function
- `XportGenerationError` exception
- Path validation logic
- Label generation logic

**Benefits**:
- Clean separation of writing from building
- 95 lines in focused module
- Easy to test writing independently
- Reusable across codebase

**Commit**: 695eb8c

---

### 📋 Step 2: Builder Module (NEXT)

**Goal**: Extract DataFrame construction orchestration

**Create**: `xpt_module/builder.py` (~500 lines)

**Extract These Methods**:
1. **Class Definition**:
   - `_DomainFrameBuilder` class
   - `__init__()` method
   
2. **Main Orchestration**:
   - `build()` - Main entry point
   - Coordinates all transformations
   
3. **Mapping Application**:
   - `_apply_mapping()` - Apply column mappings
   - `_apply_codelist_transformation()` - Apply CT to mapping
   
4. **Column Management**:
   - `_default_column()` - Create default columns
   - `_reorder_columns()` - Order columns properly
   - `_drop_empty_optional_columns()` - Remove empty columns
   
5. **Helper Methods**:
   - `_unquote_column()` - Handle quoted column names
   - Other utility functions

**Expected Lines**: ~500  
**Effort**: 2-3 hours  
**Dependencies**: None (independent module)

---

### 📋 Step 3: Date Transformers

**Goal**: Extract all date/time transformation logic

**Create**: `xpt_module/transformers/date.py` (~300 lines)

**Extract These Methods**:
1. **Date Normalization**:
   - `_normalize_dates()` - ISO 8601 normalization
   - `_coerce_iso8601()` - ISO coercion helper
   
2. **Duration Normalization**:
   - `_normalize_durations()` - ISO 8601 duration
   
3. **Study Day Calculations**:
   - `_calculate_dy()` - Calculate --DY variables
   - `_compute_dy()` - Individual day computation
   - `_compute_study_day()` - Study day helper
   
4. **Date Pair Validation**:
   - `_ensure_date_pair_order()` - Start ≤ End validation

**Create Class**: `DateTransformer`
```python
class DateTransformer:
    @staticmethod
    def normalize_iso8601(series: pd.Series) -> pd.Series:
        """Normalize dates to ISO 8601 format."""
        
    @staticmethod
    def calculate_study_days(...) -> pd.Series:
        """Calculate study days from reference start."""
        
    @staticmethod
    def validate_date_pairs(...) -> None:
        """Ensure start dates <= end dates."""
```

**Expected Lines**: ~300  
**Effort**: 2-3 hours  
**Dependencies**: builder.py (called during build)

---

### 📋 Step 4: Codelist Transformers

**Goal**: Extract controlled terminology transformation logic

**Create**: `xpt_module/transformers/codelist.py` (~400 lines)

**Extract These Methods**:
1. **CT Application**:
   - `_apply_codelist_validations()` - Apply CT to columns
   
2. **CT Validation**:
   - `_validate_controlled_terms()` - Validate against CT
   - `_validate_paired_terms()` - TESTCD/TEST validation
   
3. **MedDRA Handling**:
   - `_populate_meddra_defaults()` - MedDRA term population

**Create Class**: `CodelistTransformer`
```python
class CodelistTransformer:
    def __init__(self, metadata: StudyMetadata):
        self.metadata = metadata
        
    def apply_codelists(self, df: pd.DataFrame) -> None:
        """Apply controlled terminology to DataFrame."""
        
    def validate_codelists(self, df: pd.DataFrame) -> list[ValidationIssue]:
        """Validate codelist compliance."""
```

**Expected Lines**: ~400  
**Effort**: 3-4 hours  
**Dependencies**: builder.py, validators framework

---

### 📋 Step 5: Numeric & Text Transformers

**Goal**: Extract numeric and text transformation logic

**Create**: 
- `xpt_module/transformers/numeric.py` (~150 lines)
- `xpt_module/transformers/text.py` (~50 lines)

**Numeric Methods**:
1. `_populate_stresc_from_orres()` - STRESC computation
2. `_force_numeric()` - Type coercion
3. `_assign_sequence()` - Sequence assignment
4. `_fill_required_defaults()` - Default value filling

**Text Methods**:
1. `_replace_unknown()` - Handle unknown values
2. `_normalize_visit()` - Visit normalization

**Create Classes**:
```python
class NumericTransformer:
    @staticmethod
    def force_numeric(series: pd.Series) -> pd.Series:
        """Coerce series to numeric type."""
        
    @staticmethod
    def populate_stresc(df: pd.DataFrame) -> None:
        """Populate STRESC from ORRES."""

class TextTransformer:
    @staticmethod
    def replace_unknown(series: pd.Series, default: str) -> pd.Series:
        """Replace unknown values with default."""
        
    @staticmethod
    def normalize_visit(df: pd.DataFrame) -> None:
        """Normalize visit naming."""
```

**Expected Lines**: ~200 total  
**Effort**: 2 hours  
**Dependencies**: builder.py

---

### 📋 Step 6: Validators Module

**Goal**: Extract XPT-specific validation logic

**Create**: `xpt_module/validators.py` (~300 lines)

**Extract These Methods**:
1. **Required Value Validation**:
   - `_validate_required_values()` - Check required fields
   - `_enforce_required_values()` - Enforce requirements
   
2. **Length Validation**:
   - `_enforce_lengths()` - Enforce max lengths

3. **Post-Processing**:
   - `_post_process_domain()` - Domain-specific processing

**Create Class**:
```python
class XPTValidator:
    def __init__(self, domain: SDTMDomain):
        self.domain = domain
        
    def validate_required_values(self, df: pd.DataFrame) -> list[ValidationIssue]:
        """Validate required values present."""
        
    def validate_lengths(self, df: pd.DataFrame) -> list[ValidationIssue]:
        """Validate field lengths."""
```

**Expected Lines**: ~300  
**Effort**: 2-3 hours  
**Dependencies**: validators framework

---

### 📋 Step 7: Integration & Testing

**Goal**: Wire everything together and test

**Tasks**:
1. **Update builder.py** to use transformers
2. **Update __init__.py** to export all classes
3. **Update consumers** (cli/commands/study.py, etc.)
4. **Add backward compatibility** layer
5. **Comprehensive testing**:
   - Unit tests for each transformer
   - Integration tests for builder
   - End-to-end tests for complete workflow

**Create**: `xpt_module/compat.py` (backward compatibility)
```python
# Maintain backward compatibility with old xpt.py
from .builder import build_domain_dataframe
from .writer import write_xpt_file, XportGenerationError

__all__ = ["build_domain_dataframe", "write_xpt_file", "XportGenerationError"]
```

**Expected Effort**: 4-6 hours  
**Testing**: Critical - ensure no regressions

---

### 📋 Step 8: Deprecation & Cleanup

**Goal**: Deprecate old xpt.py and clean up

**Tasks**:
1. **Add deprecation warnings** to old xpt.py
2. **Update all imports** across codebase
3. **Remove xpt.py** once all consumers updated
4. **Update documentation**
5. **Performance benchmarking** (ensure no regression)

**Expected Effort**: 2-3 hours

---

## Benefits

### Code Quality
- **Modular Design**: Each module has single responsibility
- **Smaller Files**: Average 205 lines vs 3,124 monolith
- **Better Organization**: Easy to find specific logic
- **Reduced Complexity**: Each transformer is simple

### Maintainability
- **Easy to Navigate**: Know exactly where to find code
- **Simple to Modify**: Changes isolated to relevant module
- **Clear Dependencies**: Explicit imports show relationships
- **Better Documentation**: Focused docstrings per module

### Testability
- **Unit Testing**: Test each transformer independently
- **Mocking**: Easy to mock dependencies
- **Integration Testing**: Test builder orchestration
- **Performance Testing**: Benchmark individual transformers

### Performance
- **Optimizable**: Can optimize individual transformers
- **Cacheable**: Can add caching per transformer
- **Parallelizable**: Could parallelize transformations
- **Measurable**: Can profile each transformer separately

---

## Success Criteria

### Code Metrics
- [x] Average module size: <300 lines (Target: 205 avg)
- [ ] Total modules: 9 focused modules
- [ ] Total lines: ~1,845 lines (from 3,124)
- [ ] Test coverage: >80% per module

### Functionality
- [ ] All tests passing
- [ ] No regressions in output
- [ ] XPT files byte-identical to original
- [ ] Performance: Same or better than original

### Architecture
- [x] Clean separation of concerns
- [x] Reusable transformers
- [x] Testable components
- [x] Clear dependencies

---

## Timeline

### Completed
- **Step 1**: Package & Writer - ✅ DONE (1 hour)

### Remaining
- **Step 2**: Builder Module - 2-3 hours
- **Step 3**: Date Transformers - 2-3 hours
- **Step 4**: Codelist Transformers - 3-4 hours
- **Step 5**: Numeric/Text Transformers - 2 hours
- **Step 6**: Validators Module - 2-3 hours
- **Step 7**: Integration & Testing - 4-6 hours
- **Step 8**: Deprecation & Cleanup - 2-3 hours

**Total Remaining**: 17-24 hours over 1-2 weeks

---

## Risk Mitigation

### Risks
1. **Breaking Changes**: Old xpt.py consumers break
2. **Performance Regression**: New architecture slower
3. **Behavioral Changes**: Output differs from original
4. **Incomplete Extraction**: Missing edge cases

### Mitigation Strategies
1. **Backward Compatibility**: Maintain compat layer
2. **Benchmarking**: Profile before/after
3. **Testing**: Comprehensive test suite
4. **Incremental Approach**: Test each step

---

## Status

**Phase 4 Progress**: 1/8 steps complete (12.5%)  
**Lines Extracted**: 145/1,845 (8%)  
**Current Step**: Step 2 (Builder Module)  
**Next Milestone**: Builder extracted and tested

**Last Updated**: December 12, 2025
