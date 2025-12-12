# Core Refactoring Plan - Maintainability First

**Date:** December 12, 2025  
**Focus:** Clean, modular, maintainable codebase before adding features  
**Priority:** Remove complexity, improve organization, enhance testability

---

## Guiding Principles

1. **Single Responsibility** - Each module does ONE thing well
2. **Small Files** - Target: <300 lines per file
3. **Clear Boundaries** - Minimize inter-module dependencies
4. **Remove Overhead** - Delete unused code, simplify complex logic
5. **Test-Friendly** - Easy to unit test each component
6. **No Feature Creep** - Focus on refactoring existing functionality

---

## Phase 1: Define-XML Module Refactoring

### Current State
- **File:** `define_xml.py` (1,700 lines, 44 functions/classes)
- **Problems:**
  - Mixed responsibilities (metadata building, XML writing, utilities)
  - Complex interdependencies
  - Hard to test individual components
  - Difficult to modify without side effects

### Refactoring Strategy

Create `define_xml_module/` with clear separation:

```
define_xml_module/
├── __init__.py                 # Public API
├── constants.py                # Namespaces, OIDs, defaults (~100 lines)
├── models.py                   # Data classes (~150 lines)
├── standards.py                # Standards configuration (~100 lines)
├── metadata_builder.py         # Metadata construction (~300 lines)
├── dataset_builder.py          # ItemGroupDef building (~250 lines)
├── variable_builder.py         # ItemDef building (~300 lines)
├── codelist_builder.py         # CodeList building (~250 lines)
├── value_list_builder.py       # ValueList/WhereClause (~200 lines)
├── xml_writer.py               # XML tree generation (~250 lines)
└── utils.py                    # Helper functions (~100 lines)
```

### File Responsibility Breakdown

#### constants.py (~100 lines)
- All namespace declarations (ODM_NS, DEF_NS, etc.)
- OID constants (IG_STANDARD_OID, etc.)
- Default values (versions, page refs, etc.)
- Context values
- **No logic, just configuration**

#### models.py (~150 lines)
From define_xml.py:
- `DefineGenerationError` (line 81)
- `StandardDefinition` (line 86)
- `OriginDefinition` (line 99)
- `MethodDefinition` (line 110)
- `CommentDefinition` (line 121)
- `WhereClauseDefinition` (line 160)
- `ValueListItemDefinition` (line 180)
- `ValueListDefinition` (line 195)
- `StudyDataset` (line 212)
- **Pure data classes, no business logic**

#### standards.py (~100 lines)
From define_xml.py:
- `_default_standard_comments()` (line 128)
- `_get_default_standards()` (line 225)
- `_append_standards()` (line 704)
- **Standards configuration and management**

#### metadata_builder.py (~300 lines)
From define_xml.py:
- `build_define_tree()` (line 341) - orchestration
- `build_study_define_tree()` (line 365) - main builder
- Coordinate all builders to create complete tree
- **High-level orchestration only**

#### dataset_builder.py (~250 lines)
From define_xml.py:
- `_append_item_refs()` (line 724)
- `_get_key_sequence()` (line 752)
- `_get_variable_role()` (line 795)
- `_active_domain_variables()` (line 819)
- `_domain_description_alias()` (line 1334)
- **Build ItemGroupDef elements**

#### variable_builder.py (~300 lines)
From define_xml.py:
- `_append_item_defs()` (line 853)
- `_build_item_def_element()` (line 1354)
- `_get_datatype()` (line 908)
- `_get_origin()` (line 952)
- `_is_all_missing()` (line 988)
- `_item_oid()` (line 1280)
- **Build ItemDef elements**

#### codelist_builder.py (~250 lines)
From define_xml.py:
- `_append_code_lists()` (line 996)
- `_build_code_list_element()` (line 1005)
- `_collect_extended_codelist_values()` (line 1116)
- `_should_use_enumerated_item()` (line 1149)
- `_needs_meddra()` (line 1211)
- `_get_decode_value()` (line 1216)
- `_get_nci_code()` (line 1258)
- `_code_list_oid()` (line 1303)
- **Build CodeList elements**

#### value_list_builder.py (~200 lines)
From define_xml.py:
- `_build_supp_value_lists()` (line 1425)
- `_append_value_list_defs()` (line 1514)
- `_append_where_clause_defs()` (line 1550)
- `generate_vlm_for_findings_domain()` (line 1584)
- **Build ValueList and WhereClause elements**

#### xml_writer.py (~250 lines)
From define_xml.py:
- `write_define_file()` (line 270)
- `write_study_define_file()` (line 308)
- `_append_method_defs()` (line 1643)
- `_append_comment_defs()` (line 1679)
- XML formatting and file writing
- **File I/O and XML serialization**

#### utils.py (~100 lines)
From define_xml.py:
- `_tag()` (line 1314)
- `_attr()` (line 1318)
- `_safe_href()` (line 1322)
- **Helper functions only**

### Migration Steps

1. **Create package structure** (30 min)
   - Create `define_xml_module/` directory
   - Create all module files with docstrings
   - Create `__init__.py` with public API

2. **Move constants** (15 min)
   - Copy all constants to `constants.py`
   - Update imports in original file
   - Test no breaking changes

3. **Move models** (30 min)
   - Copy all dataclasses to `models.py`
   - Update imports
   - Test

4. **Move standards logic** (45 min)
   - Move standards functions to `standards.py`
   - Update imports
   - Test

5. **Move codelist builder** (1 hour)
   - Move codelist functions to `codelist_builder.py`
   - Update imports
   - Test

6. **Move variable builder** (1 hour)
   - Move variable functions to `variable_builder.py`
   - Update imports
   - Test

7. **Move dataset builder** (45 min)
   - Move dataset functions to `dataset_builder.py`
   - Update imports
   - Test

8. **Move value list builder** (45 min)
   - Move value list functions to `value_list_builder.py`
   - Update imports
   - Test

9. **Move utils** (15 min)
   - Move utility functions to `utils.py`
   - Update imports
   - Test

10. **Move xml writer** (45 min)
    - Move writing functions to `xml_writer.py`
    - Update imports
    - Test

11. **Move metadata builder** (45 min)
    - Move orchestration to `metadata_builder.py`
    - Update imports
    - Test

12. **Update public API** (30 min)
    - Define clean exports in `__init__.py`
    - Update external imports
    - Test all usage points

13. **Remove old file** (15 min)
    - Delete `define_xml.py`
    - Verify all tests pass

**Total Time Estimate:** ~8 hours

---

## Phase 2: Study Command Refactoring

### Current State
- **File:** `cli/commands/study.py` (1,980 lines)
- **Problems:**
  - Business logic mixed with CLI
  - Large helper functions embedded in file
  - Hard to reuse logic outside CLI
  - Difficult to test

### Refactoring Strategy

Extract services and simplify CLI:

```
services/
├── study_orchestration_service.py  # NEW (~400 lines)
├── domain_discovery_service.py     # NEW (~200 lines)
├── file_organization_service.py    # NEW (~150 lines)
└── progress_reporting_service.py   # NEW (~100 lines)

cli/commands/
└── study.py                         # SIMPLIFIED (~300 lines)
```

### File Responsibilities

#### study_orchestration_service.py (~400 lines)
**Purpose:** Orchestrate entire study processing workflow

From study.py:
- Study processing orchestration logic
- Domain processing coordination
- Trial design synthesis coordination
- File generation coordination
- Error handling and recovery
- **High-level study workflow**

#### domain_discovery_service.py (~200 lines)
**Purpose:** Discover and classify input files

From study.py:
- Find all data files in study folder
- Classify files by domain
- Detect domain variants
- Group related files
- Handle edge cases (split domains, etc.)
- **File discovery and classification**

#### file_organization_service.py (~150 lines)
**Purpose:** Organize output files and directories

From study.py:
- Create output directory structure
- Organize files by type (XPT, XML, SAS)
- Handle file naming conventions
- Manage file paths
- **Output file management**

#### progress_reporting_service.py (~100 lines)
**Purpose:** Report progress to user

From study.py:
- Progress bar management
- Status messages
- Summary reporting
- Error reporting
- **User feedback**

#### study.py (SIMPLIFIED ~300 lines)
**Purpose:** CLI command definition ONLY

Keep:
- `@click.command()` definition
- `@click.option()` definitions
- `study_command()` function - calls services
- Argument parsing
- CLI-specific error handling
- **CLI interface only, no business logic**

### Migration Steps

1. **Extract domain discovery** (2 hours)
   - Create `domain_discovery_service.py`
   - Move file finding logic
   - Move classification logic
   - Test independently

2. **Extract file organization** (1 hour)
   - Create `file_organization_service.py`
   - Move directory creation logic
   - Move file naming logic
   - Test independently

3. **Extract progress reporting** (1 hour)
   - Create `progress_reporting_service.py`
   - Move progress bar logic
   - Move logging logic
   - Test independently

4. **Extract study orchestration** (3 hours)
   - Create `study_orchestration_service.py`
   - Move workflow orchestration
   - Move error handling
   - Integrate other services
   - Test independently

5. **Simplify CLI command** (2 hours)
   - Update `study.py` to use services
   - Remove embedded business logic
   - Keep only CLI concerns
   - Test CLI still works

**Total Time Estimate:** ~9 hours

---

## Phase 3: Remove Complexity and Overhead

### Objectives
1. **Identify and remove dead code**
2. **Simplify overly complex functions**
3. **Consolidate duplicate code**
4. **Reduce cyclomatic complexity**
5. **Improve code readability**

### Analysis Tools

```bash
# Find unused imports
pip install vulture
vulture cdisc_transpiler/

# Find duplicate code
pip install pylint
pylint --disable=all --enable=duplicate-code cdisc_transpiler/

# Measure complexity
pip install radon
radon cc cdisc_transpiler/ -a -nb

# Find dead code
pip install dead
dead cdisc_transpiler/
```

### Target Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Max file size | 1,980 lines | 300 lines |
| Avg cyclomatic complexity | Unknown | <10 |
| Code duplication | Unknown | <3% |
| Unused imports | Unknown | 0 |

### Actions

1. **Remove unused imports** (2 hours)
   - Run vulture on all files
   - Remove identified unused imports
   - Test no breakage

2. **Remove dead code** (2 hours)
   - Identify unreachable code
   - Remove unused functions
   - Test no breakage

3. **Consolidate duplicates** (3 hours)
   - Find duplicate code blocks
   - Extract to shared utilities
   - Update all call sites
   - Test

4. **Simplify complex functions** (4 hours)
   - Identify high complexity functions
   - Break into smaller functions
   - Extract nested logic
   - Test

5. **Improve readability** (2 hours)
   - Add docstrings where missing
   - Improve variable names
   - Add type hints
   - Format with black

**Total Time Estimate:** ~13 hours

---

## Phase 4: Improve Module Organization

### Current Issues
- Some modules still too large
- Unclear module boundaries
- Circular dependencies
- Inconsistent organization

### Actions

1. **Review all modules** (2 hours)
   - List all module responsibilities
   - Identify violations of single responsibility
   - Identify circular dependencies

2. **Split large modules** (4 hours)
   - mapping.py (653 lines) - split if needed
   - metadata.py (593 lines) - split if needed
   - Keep modules focused

3. **Remove circular dependencies** (3 hours)
   - Identify circular imports
   - Refactor to remove circles
   - Use dependency injection

4. **Standardize structure** (2 hours)
   - Consistent __all__ exports
   - Consistent import patterns
   - Consistent module docstrings

**Total Time Estimate:** ~11 hours

---

## Phase 5: Testing Infrastructure

### Current State
- Limited test coverage
- Hard to test due to tight coupling
- No integration tests

### Goals
- Unit tests for all services
- Integration tests for workflows
- >80% code coverage

### Actions

1. **Add unit tests for define_xml_module** (8 hours)
   - Test each builder independently
   - Test utils, constants, models
   - Mock dependencies

2. **Add unit tests for services** (6 hours)
   - Test domain_service
   - Test file_generation_service
   - Test trial_design_service
   - Test new study services

3. **Add integration tests** (4 hours)
   - Test full workflows
   - Test with mock data
   - Test error scenarios

4. **Measure coverage** (1 hour)
   - Run coverage tools
   - Identify gaps
   - Add missing tests

**Total Time Estimate:** ~19 hours

---

## Phase 6: Documentation Consolidation

### Current State
- Multiple overlapping docs
- Outdated information
- Scattered documentation

### Actions

1. **Merge status documents** (2 hours)
   - Combine all refactoring docs
   - Keep relevant history
   - Update current state

2. **Update README** (1 hour)
   - Current architecture
   - How to use
   - How to contribute

3. **Add module documentation** (3 hours)
   - Docstrings for all modules
   - Usage examples
   - Architecture diagrams

4. **Create developer guide** (2 hours)
   - How to add new domains
   - How to extend validation
   - Testing guidelines

**Total Time Estimate:** ~8 hours

---

## Total Time Estimate

| Phase | Hours |
|-------|-------|
| Phase 1: Define-XML Refactoring | 8 |
| Phase 2: Study Command Refactoring | 9 |
| Phase 3: Remove Complexity | 13 |
| Phase 4: Module Organization | 11 |
| Phase 5: Testing Infrastructure | 19 |
| Phase 6: Documentation | 8 |
| **TOTAL** | **68 hours (~2 weeks)** |

---

## Success Criteria

### Code Quality
- [  ] No file >500 lines
- [  ] Avg file size <300 lines
- [  ] No function >50 lines
- [  ] Cyclomatic complexity <10
- [  ] Zero code duplication >5 lines
- [  ] Zero unused imports
- [  ] Zero dead code

### Architecture
- [  ] Clear module boundaries
- [  ] Single responsibility per module
- [  ] No circular dependencies
- [  ] Services independent of CLI
- [  ] Easy to mock for testing

### Testing
- [  ] >80% code coverage
- [  ] All services have unit tests
- [  ] Integration tests for workflows
- [  ] All tests pass
- [  ] Fast test execution (<30 sec)

### Documentation
- [  ] Single comprehensive guide
- [  ] All modules documented
- [  ] Architecture clearly explained
- [  ] Examples for common tasks
- [  ] Developer contribution guide

---

## Next Steps

1. **Start with Phase 1** - Define-XML refactoring
2. **Test after each migration step** - Ensure no breakage
3. **Commit frequently** - Small, focused commits
4. **Document as you go** - Update docs with changes
5. **Get feedback early** - Review after each phase

---

**REMEMBER: The goal is a clean, maintainable codebase. No new features until refactoring is complete.**
