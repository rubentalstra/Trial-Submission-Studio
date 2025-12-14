# Study Command Flow Analysis (As-Is Documentation)

**Date:** 2025-12-14  
**Repository:** rubentalstra/cdisc-transpiler  
**Focus:** `cdisc_transpiler/cli/commands/study.py` and full execution path

---

## 1. Purpose & Entrypoints

### Primary Purpose
The `study` command is the main entrypoint for processing an entire clinical study folder containing SDTM (Study Data Tabulation Model) source data files. It orchestrates the generation of regulatory submission packages including:
- **XPT files**: SAS transport format (v5) for FDA submissions
- **Dataset-XML files**: CDISC Dataset-XML 1.0 format
- **Define-XML**: Metadata file documenting all datasets per CDISC Define-XML 2.1 standard
- **SAS programs**: Transformation programs for each domain

### How It's Invoked

#### CLI Entrypoint
```bash
# Primary entrypoint via console script (pyproject.toml)
cdisc-transpiler study <study_folder> [OPTIONS]

# Alternative: module execution
python -m cdisc_transpiler.cli study <study_folder> [OPTIONS]
```

#### Code Path
1. **Console script** → `cdisc_transpiler.cli:app` (pyproject.toml line 28)
2. **Click group** → `cli/__init__.py:app()` (line 14-17)
3. **Command registration** → `cli/__init__.py:21` registers `study.study_command`
4. **Command execution** → `cli/commands/study.py:study_command()` (line 113)

### Inputs, Arguments & Configuration

#### Required Arguments
- `study_folder`: Path to folder containing CSV files with SDTM source data

#### Options & Flags
| Option | Type | Default | Purpose |
|--------|------|---------|---------|
| `--output-dir` | Path | `<study_folder>/output` | Output directory for generated files |
| `--study-id` | String | Derived from folder name | Study identifier (e.g., DEMO_GDISC) |
| `--format` | Choice | `both` | Output format: xpt, xml, or both |
| `--define-xml/--no-define-xml` | Boolean | `True` | Generate Define-XML 2.1 metadata |
| `--sas/--no-sas` | Boolean | `True` | Generate SAS transformation programs |
| `--sdtm-version` | String | `3.4` | SDTM-IG version for Define-XML |
| `--define-context` | Choice | `Submission` | Define-XML context (Submission/Other) |
| `--streaming` | Boolean | `False` | Use streaming mode for large datasets |
| `--chunk-size` | Integer | `1000` | Chunk size for streaming mode |
| `--min-confidence` | Float | `0.5` | Minimum confidence for fuzzy column matching |
| `-v, --verbose` | Count | `0` | Verbosity level (-v, -vv) |

#### Environment Variables
None explicitly used. Configuration is CLI-driven.

#### External Data Sources
- **Study folder CSV files**: Source data files (e.g., DM.csv, AE.csv, LB.csv)
- **Metadata files**: Items.csv, CodeLists.csv (optional)
- **CDISC Controlled Terminology**: Loaded from `docs/Controlled_Terminology/` (hardcoded)
- **SDTM specifications**: Loaded from `docs/SDTMIG_v3.4/` JSON files (hardcoded)

---

## 2. High-Level Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        study_command()                               │
│                    (cli/commands/study.py:113)                       │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │ 1. Initialization       │
                    │  - Create logger        │
                    │  - Load metadata        │
                    │  - List domains         │
                    │  - Create output dirs   │
                    └────────────┬────────────┘
                                 │
                    ┌────────────▼────────────────────┐
                    │ 2. File Discovery               │
                    │  DomainDiscoveryService         │
                    │  - Scan CSV files               │
                    │  - Match to domains             │
                    │  - Group variants               │
                    └────────────┬────────────────────┘
                                 │
                    ┌────────────▼─────────────────────┐
                    │ 3. Domain Processing Loop        │
                    │  For each domain:                │
                    └────────────┬─────────────────────┘
                                 │
                    ┌────────────▼─────────────────────────┐
                    │ DomainProcessingCoordinator          │
                    │  - Load input files                  │
                    │  - Apply transformations (VS, LB)    │
                    │  - Build column mappings             │
                    │  - Merge variants                    │
                    │  - Generate SUPPQUAL                 │
                    │  - Write output files (XPT/XML/SAS)  │
                    └────────────┬─────────────────────────┘
                                 │
                    ┌────────────▼─────────────────────┐
                    │ 4. Domain Synthesis              │
                    │  DomainSynthesisCoordinator      │
                    │  - Generate missing observation  │
                    │    domains (AE, LB, VS, EX)      │
                    │  - Generate trial design         │
                    │    domains (TS, TA, TE, SE, DS)  │
                    └────────────┬─────────────────────┘
                                 │
                    ┌────────────▼─────────────────────┐
                    │ 5. RELREC Generation             │
                    │  StudyOrchestrationService       │
                    │  - Build relationships between   │
                    │    domains (AE→DS, EX→DS)        │
                    └────────────┬─────────────────────┘
                                 │
                    ┌────────────▼─────────────────────┐
                    │ 6. Define-XML Generation         │
                    │  write_study_define_file()       │
                    │  - Collect all StudyDataset      │
                    │  - Generate Define-XML 2.1       │
                    └────────────┬─────────────────────┘
                                 │
                    ┌────────────▼─────────────────────┐
                    │ 7. Summary & Completion          │
                    │  print_study_summary()           │
                    │  - Display results table         │
                    │  - Show errors/warnings          │
                    └──────────────────────────────────┘
```

---

## 3. Detailed Call Flow

### Phase 1: Initialization (Lines 165-248)

#### Step 1.1: Logger Creation
```
study.py:165  → create_logger(console, verbosity)
  └─> logging_config.py:589 → SDTMLogger.__init__()
      └─> Initializes Rich console, stats tracking
```

**Inputs:** `console` (Rich Console), `verbosity` (int)  
**Outputs:** `SDTMLogger` instance  
**Side Effects:** Sets global logger via `set_logger()`

#### Step 1.2: Domain Registry Loading
```
study.py:168  → list_domains()
  └─> domains_module/__init__.py:35 → list_domains()
      └─> loaders.py:23 → load_sdtm_domains_from_json()
          └─> Reads docs/SDTMIG_v3.4/*.json files
```

**Inputs:** None (reads from hardcoded path)  
**Outputs:** List of supported domain codes (e.g., ['DM', 'AE', 'LB', ...])  
**Side Effects:** 
- Reads ~40 JSON files from filesystem
- Populates internal domain registry (singleton pattern)

#### Step 1.3: Study Metadata Loading
```
study.py:183  → load_study_metadata(study_folder)
  └─> metadata_module/loaders.py:150 → load_study_metadata()
      ├─> _load_items_csv(study_folder / "Items.csv")
      │   └─> Returns ItemMetadata objects
      └─> _load_codelists_csv(study_folder / "CodeLists.csv")
          └─> Returns CodeList objects
```

**Inputs:** `study_folder` (Path)  
**Outputs:** `StudyMetadata` object with items and codelists  
**Side Effects:** Reads Items.csv and CodeLists.csv if present (gracefully handles missing)

#### Step 1.4: Output Directory Setup
```
study.py:192-208
  - Create output_dir
  - Create xpt_dir, xml_dir, sas_dir based on format flags
  - Ensure ACRF PDF placeholder (for Define-XML)
```

**Side Effects:** Creates directory structure on filesystem

### Phase 2: File Discovery (Lines 211-247)

```
study.py:215  → DomainDiscoveryService().discover_domain_files()
  └─> services/domain_discovery_service.py:44
      ├─> For each CSV file:
      │   ├─> _is_metadata_file() → Skip Items.csv, CodeLists.csv
      │   └─> _match_domain() → Match filename to domain code
      │       ├─> Exact match (e.g., "DM.csv" → DM)
      │       └─> Prefix match (e.g., "LBCC.csv" → LB variant)
      └─> Returns dict[domain_code, list[tuple[Path, variant_name]]]
```

**Inputs:** 
- `csv_files`: List of Path objects
- `supported_domains`: List of domain codes

**Outputs:** 
- Dictionary mapping domain → list of (file_path, variant_name)
- Example: `{"LB": [(Path("LBCC.csv"), "LBCC"), (Path("LBHM.csv"), "LBHM")]}`

**Matching Logic:**
1. Skip files with patterns: CODELISTS, ITEMS, README, METADATA
2. Split filename by underscore
3. Try exact match of each part against domain codes
4. Try prefix match (e.g., "LBCC" starts with "LB")

**Side Effects:** Logs verbose matching information

### Phase 3: Domain Processing Loop (Lines 335-488)

For each domain in `ordered_domains` (sorted with DM first):

#### Step 3.1: Coordinator Initialization
```
study.py:346  → DomainProcessingCoordinator()
  └─> services/domain_processing_coordinator.py:42
      └─> Initializes with StudyOrchestrationService
```

#### Step 3.2: Process and Merge Domain
```
study.py:347  → coordinator.process_and_merge_domain()
  └─> domain_processing_coordinator.py:55
      ├─> For each input file:
      │   └─> _process_single_file()
      │       ├─> load_input_dataset(file)
      │       │   └─> io_module/readers.py:40
      │       │       ├─> pd.read_csv() with normalization
      │       │       └─> Normalize headers (strip, uppercase)
      │       ├─> Check for VS/LB transformation needs
      │       ├─> Apply VS transformation (wide→long)
      │       │   └─> orchestration_service.reshape_vs_to_long()
      │       │       └─> study_orchestration_service.py:55
      │       │           ├─> Loads CDISC CT for VS test codes
      │       │           ├─> Finds ORRES_* columns
      │       │           ├─> Unpivots to long format
      │       │           └─> Returns reshaped DataFrame
      │       ├─> Apply LB transformation (wide→long)
      │       │   └─> orchestration_service.reshape_lb_to_long()
      │       │       └─> study_orchestration_service.py:201
      │       │           ├─> Loads CDISC CT for LB test codes
      │       │           ├─> Regex pattern matching for test columns
      │       │           ├─> Unpivots to long format
      │       │           └─> Returns reshaped DataFrame
      │       ├─> Build column mappings
      │       │   └─> create_mapper(domain_code, metadata, min_confidence)
      │       │       └─> mapping_module/engine.py:37
      │       │           └─> MappingEngine.__init__()
      │       │       └─> engine.suggest(frame)
      │       │           ├─> Fuzzy match columns to variables
      │       │           ├─> Use rapidfuzz.fuzz for similarity
      │       │           └─> Returns MappingSuggestions
      │       ├─> build_config(domain_code, mappings)
      │       │   └─> mapping_module/__init__.py
      │       │       └─> Creates MappingConfig object
      │       ├─> build_domain_dataframe(frame, config)
      │       │   └─> xpt_module/builder.py:42
      │       │       └─> DomainFrameBuilder.build()
      │       │           ├─> Apply column mappings
      │       │           ├─> Transform dates (ISO 8601)
      │       │           ├─> Apply codelists
      │       │           ├─> Calculate study days (*DY)
      │       │           ├─> Reorder columns per domain spec
      │       │           └─> Returns SDTM-compliant DataFrame
      │       └─> build_suppqual() for non-standard columns
      │           └─> submission_module/suppqual.py:115
      │               ├─> Identify unused source columns
      │               ├─> Build SUPPQUAL records
      │               └─> Returns SUPP DataFrame
      ├─> _merge_dataframes() if multiple variants
      │   └─> pd.concat() and re-sequence SEQ columns
      ├─> _deduplicate_lb_data() for LB domain
      │   └─> Drop duplicates on (USUBJID, LBTESTCD, LBDTC)
      └─> _generate_output_files()
          ├─> write_xpt_file() if xpt format
          │   └─> xpt_module/writer.py:40
          │       ├─> Convert DataFrame to SAS format
          │       ├─> Apply XPT constraints (200 char, 40 vars)
          │       └─> Write binary XPT file
          ├─> write_dataset_xml() if xml format
          │   └─> xml_module/dataset_module/writer.py:30
          │       ├─> Build Dataset-XML structure
          │       └─> Write XML file
          ├─> generate_sas_program() if sas flag
          │   └─> sas_module/generator.py:25
          │       ├─> Build SAS DATA step code
          │       └─> write_sas_file()
          └─> _generate_supplemental_files() if SUPP exists
              └─> Repeat XPT/XML/SAS for SUPP domain
```

**Key Data Transformations:**

1. **VS Transformation** (lines 286-319):
   - Input: Wide format with columns like `ORRES_HR`, `ORRES_SYSBP`
   - Output: Long format with VSTESTCD, VSTEST, VSORRES columns
   - Multiplier: 1 row → N rows (N = number of vital sign tests)

2. **LB Transformation** (lines 321-370):
   - Input: Wide format with columns like `WBC`, `HGB`, `PLAT`
   - Output: Long format with LBTESTCD, LBTEST, LBORRES columns
   - Multiplier: 1 row → N rows (N = number of lab tests)

3. **Column Mapping** (lines 387-410):
   - Uses fuzzy matching (rapidfuzz) to map source columns to SDTM variables
   - Confidence threshold filtering (default 0.5)
   - Handles synonyms and abbreviations

4. **Study Day Calculation** (lines 400-428):
   - Requires RFSTDTC (reference start date) from DM domain
   - Calculates *DY variables as (date - RFSTDTC) + 1
   - Special handling for dates before RFSTDTC (negative days)

**Side Effects:**
- Reads CSV files from filesystem
- Writes XPT/XML/SAS files to output directories
- Logs progress and statistics
- Accumulates results and errors lists

### Phase 4: Domain Synthesis (Lines 490-531)

#### Step 4.1: Synthesize Missing Observation Domains
```
study.py:491-508
For domain in [AE, LB, VS, EX] if not processed:
  └─> DomainSynthesisCoordinator().synthesize_empty_observation_domain()
      └─> services/domain_synthesis_coordinator.py:91
          ├─> _pick_subject() → Get reference subject ID
          ├─> _generate_empty_observation_rows()
          │   └─> Create minimal scaffold row
          ├─> build_domain_dataframe()
          └─> _generate_domain_files()
              └─> Write XPT/XML/SAS files
```

**Purpose:** FDA requires core domains even if no data exists

#### Step 4.2: Synthesize Trial Design Domains
```
study.py:512-531
For domain in [TS, TA, TE, SE, DS] if not processed:
  └─> DomainSynthesisCoordinator().synthesize_trial_design_domain()
      └─> services/domain_synthesis_coordinator.py:27
          ├─> _generate_trial_design_rows()
          │   ├─> TS: Study title and parameters
          │   ├─> TA: Arm and element definitions
          │   ├─> TE: Trial element timeline
          │   ├─> SE: Subject element records
          │   └─> DS: Disposition events
          └─> _generate_domain_files()
```

**Purpose:** Required for Pinnacle 21 validation and regulatory compliance

### Phase 5: RELREC Generation (Lines 533-568)

```
study.py:535-568
If RELREC not in processed_domains:
  └─> StudyOrchestrationService().synthesize_relrec()
      └─> study_orchestration_service.py:523
          ├─> build_relrec_records()
          │   └─> study_orchestration_service.py:408
          │       ├─> Find AE, DS, EX dataframes in results
          │       ├─> Create AE→DS relationships
          │       ├─> Create EX→DS relationships
          │       └─> Returns RELREC DataFrame
          └─> _generate_domain_files()
```

**Relationship Logic:**
- Links adverse events (AE) to disposition (DS) by subject
- Links exposure (EX) to disposition (DS) by subject
- Creates bidirectional references using RELID

### Phase 6: Define-XML Generation (Lines 570-587)

```
study.py:570-587
If generate_define and study_datasets:
  └─> write_study_define_file()
      └─> xml_module/define_module/__init__.py:30
          ├─> Build StudyMetadata object
          ├─> For each StudyDataset:
          │   ├─> Build dataset definition
          │   ├─> Build variable metadata
          │   ├─> Build value lists
          │   └─> Build codelists
          └─> Write Define-XML 2.1 file
```

**Inputs:** List of `StudyDataset` objects with:
- domain_code
- dataframe (for structure inference)
- config (MappingConfig)
- archive_location (relative path to XPT/XML file)
- is_split (for split datasets)

**Output:** `define.xml` file compliant with Define-XML 2.1 specification

### Phase 7: Summary Display (Lines 590-595)

```
study.py:590  → logger.log_final_stats()
study.py:593  → print_study_summary()
  └─> cli/helpers.py:116
      ├─> Build Rich table with domain results
      ├─> Show records, file indicators (XPT/XML/SAS)
      ├─> Display split datasets
      └─> Print total statistics
```

---

## 4. Data Flow & State Management

### Key Data Structures

#### 1. MappingConfig (mapping_module/models.py)
```python
@dataclass
class MappingConfig:
    domain: str
    mappings: list[ColumnMapping]
    study_id: str
    unmapped_columns: list[str]
```

**Flow:**
- Created per domain in `_build_mapped_config()`
- Passed to `build_domain_dataframe()`
- Used for column mapping, transformations, SAS generation

#### 2. StudyDataset (xml_module/define_module/models.py)
```python
@dataclass
class StudyDataset:
    domain_code: str
    dataframe: pd.DataFrame
    config: MappingConfig
    archive_location: Path
    is_split: bool = False
    split_suffix: str | None = None
```

**Flow:**
- Created after each domain is processed
- Accumulated in `study_datasets` list
- Consumed by `write_study_define_file()`

#### 3. Results Dictionary (per-domain)
```python
result = {
    "domain_code": str,
    "records": int,
    "domain_dataframe": pd.DataFrame,
    "config": MappingConfig,
    "xpt_path": Path | None,
    "xml_path": Path | None,
    "sas_path": Path | None,
    "split_datasets": list[tuple[str, pd.DataFrame, Path]],
    "supplementals": list[dict],  # SUPP domains
}
```

**Flow:**
- Returned by `process_and_merge_domain()`
- Appended to `results` list
- Used for RELREC generation and summary display

### State Mutation Points

#### Global State (via logger singleton)
```
cli/logging_config.py:564-586
- _logger: SDTMLogger | None (module-level)
- get_logger() / set_logger() pattern
```

**Mutated by:**
- `create_logger()` at study.py:165
- Each service via `from ..cli.logging_config import get_logger`

**Risk:** Multiple modules can modify logger state, potential for inconsistency

#### Study-Level State
```python
# study.py accumulates these:
processed_domains: set[str]          # Tracks which domains are done
results: list[dict]                   # All domain results
errors: list[tuple[str, str]]         # Error tracking
study_datasets: list[StudyDataset]    # For Define-XML
reference_starts: dict[str, str]      # USUBJID → RFSTDTC mapping
common_column_counts: dict[str, int]  # Column frequency heuristic
```

**Mutation flow:**
1. `processed_domains` updated after each domain
2. `reference_starts` populated after DM processing (line 400-428)
3. Used by later domains for study day calculations

### Caching & Performance

#### Domain Registry (Singleton)
```
domains_module/loaders.py:
- _DOMAIN_REGISTRY: dict[str, SDTMDomain] (module-level)
- Loaded once via load_sdtm_domains_from_json()
- Accessed via get_domain(code)
```

**Benefit:** Avoid re-parsing JSON files  
**Risk:** No cache invalidation mechanism

#### CDISC Controlled Terminology (Singleton)
```
terminology_module/registry.py:
- _TERMINOLOGY_CACHE: dict (module-level)
- Loaded lazily on first access
- Used for VS/LB test code normalization
```

**Benefit:** Avoid re-parsing CT files  
**Risk:** Memory consumption (~10MB for full CT)

### Data Flow Visualization

```
Input CSVs
    ↓
load_input_dataset() → normalize headers
    ↓
[Optional] reshape_vs_to_long() / reshape_lb_to_long()
    ↓
MappingEngine.suggest() → fuzzy match columns
    ↓
build_config() → create MappingConfig
    ↓
build_domain_dataframe() → apply transformations
    ↓                         ├─ date formatting
    │                         ├─ codelist mapping
    │                         ├─ study day calculation
    │                         └─ variable reordering
    ↓
[Optional] build_suppqual() → extract non-standard columns
    ↓
Write outputs
    ├─ write_xpt_file()
    ├─ write_dataset_xml()
    └─ generate_sas_program()
```

---

## 5. Duplication & Conflicts

### 5.1 File I/O Duplication

#### Problem: Multiple patterns for reading CSV files

**Location 1:** `io_module/readers.py:40`
```python
def load_input_dataset(path: Path) -> pd.DataFrame:
    frame = pd.read_csv(path, dtype=str, keep_default_na=False, na_values=[""])
    # Normalize headers
    frame.columns = [col.strip() for col in frame.columns]
    return frame
```

**Location 2:** `metadata_module/loaders.py:70`
```python
def _load_items_csv(path: Path) -> dict[str, ItemMetadata]:
    df = pd.read_csv(path, dtype=str)
    # Different normalization logic
    ...
```

**Location 3:** `metadata_module/csv_utils.py:10`
```python
def read_csv_safely(path: Path) -> pd.DataFrame | None:
    try:
        return pd.read_csv(path)
    except FileNotFoundError:
        return None
```

**Conflict:**
- Different `dtype` handling (str vs default)
- Different `na_values` settings
- Different error handling strategies
- Different header normalization

**Risk:** Inconsistent parsing behavior across modules

**Proposed fix:** Single source of truth for CSV reading
```python
# io_module/readers.py (enhanced)
def read_csv(
    path: Path,
    *,
    normalize_headers: bool = False,
    strict: bool = True,
    dtype: str | dict | None = str,
) -> pd.DataFrame:
    """Unified CSV reader with consistent behavior."""
```

### 5.2 Configuration Management Duplication

#### Problem: Configuration scattered across modules

**Location 1:** CLI arguments in `study.py:44-112`
- 13 different options with defaults

**Location 2:** Default values hardcoded in services
```python
# domain_processing_coordinator.py:65
min_confidence: float = 0.5  # Repeated default

# domain_synthesis_coordinator.py:159
return "SYNTH001", "2023-01-01"  # Hardcoded fallback

# study_orchestration_service.py:406
baseline_default = "2023-01-01"  # Repeated magic date
```

**Location 3:** Paths hardcoded in loaders
```python
# domains_module/loaders.py:25
domain_dir = Path("docs/SDTMIG_v3.4")  # Hardcoded path

# terminology_module/loader.py:18
ct_dir = Path("docs/Controlled_Terminology")  # Hardcoded path
```

**Conflict:**
- Magic numbers and strings repeated
- No central configuration file or dataclass
- Difficult to change defaults consistently

**Risk:** 
- Breaking changes require hunting through multiple files
- Testing with different configs is cumbersome

**Proposed fix:** Configuration dataclass
```python
# config.py (new)
@dataclass
class TranspilerConfig:
    # Paths
    sdtm_spec_dir: Path = Path("docs/SDTMIG_v3.4")
    ct_dir: Path = Path("docs/Controlled_Terminology")
    
    # Defaults
    min_confidence: float = 0.5
    default_study_date: str = "2023-01-01"
    default_subject_id: str = "SYNTH001"
    
    # XPT constraints
    max_xpt_label_length: int = 200
    max_xpt_variables: int = 40
```

### 5.3 Logging Duplication

#### Problem: Multiple logging patterns

**Pattern 1:** Structured logger (cli/logging_config.py)
```python
logger = get_logger()
logger.log_domain_start(domain_code, files)
logger.log_transformation(domain, "reshape", in_rows, out_rows)
```

**Pattern 2:** Direct Rich console (cli/helpers.py:26)
```python
console = Console()
console.print(f"[green]✓[/green] {message}")
```

**Pattern 3:** Logger + console mixing (services/domain_synthesis_coordinator.py:359)
```python
from ..cli.logging_config import get_logger
logger = get_logger()
logger.success(f"Generated {domain_code} XPT: {xpt_path}")
```

**Conflict:**
- Some modules import and create their own console
- Some use global logger
- Inconsistent message formatting

**Risk:**
- Messages may bypass verbosity controls
- Duplicate output in some scenarios

**Proposed fix:** Enforce single logging interface
- Remove all direct Console() instantiations
- All logging through get_logger()
- Services should not import Rich directly

### 5.4 File Writing Duplication

#### Problem: File generation logic repeated across services

**XPT Writing:**
- `domain_processing_coordinator.py:531-536`
- `domain_synthesis_coordinator.py:354-361`
- `study_orchestration_service.py:590-595`

**XML Writing:**
- `domain_processing_coordinator.py:547-557`
- `domain_synthesis_coordinator.py:365-371`
- `study_orchestration_service.py:598-603`

**Pattern repeated 3 times:**
```python
if xpt_dir and output_format in ("xpt", "both"):
    xpt_path = xpt_dir / f"{disk_name}.xpt"
    write_xpt_file(dataframe, domain_code, xpt_path)
    result["xpt_path"] = xpt_path
    result["xpt_filename"] = xpt_path.name
    logger.success(f"Generated XPT: {xpt_path}")
```

**Conflict:**
- Success logging format varies slightly
- Path construction duplicated
- Result dictionary updates duplicated

**Risk:**
- Bug fixes must be applied to 3+ locations
- Inconsistent behavior across domains

**Proposed fix:** Extract to FileGenerationService
```python
# services/file_generation_service.py (enhanced)
class FileGenerationService:
    def generate_all_outputs(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        config: MappingConfig,
        output_dirs: OutputDirs,
        options: OutputOptions,
    ) -> OutputResult:
        """Centralized file generation for all formats."""
```

### 5.5 Transformation Logic Duplication

#### Problem: VS and LB transformations are similar but separate

**VS Transformation** (study_orchestration_service.py:55-199)
- 145 lines of code
- Regex pattern matching for test columns
- Wide-to-long pivot logic
- CDISC CT lookup for test codes

**LB Transformation** (study_orchestration_service.py:201-406)
- 206 lines of code
- Nearly identical regex pattern matching
- Nearly identical pivot logic
- Nearly identical CT lookup

**Duplication percentage:** ~60% of logic is identical

**Conflict:**
- Bug fixes must be applied to both
- Pattern variations (e.g., `ORRES_*` vs `*ORRES`) scattered

**Risk:**
- Maintenance burden
- Behavior divergence over time

**Proposed fix:** Extract common transformation framework
```python
# transformations/wide_to_long.py (new)
class FindingsTransformer:
    def __init__(self, domain_code: str, ct_loader: CTLoader):
        self.domain_code = domain_code
        self.ct_loader = ct_loader
    
    def reshape(self, frame: pd.DataFrame) -> pd.DataFrame:
        """Generic wide-to-long for Findings domains."""
        # Common logic for VS, LB, QS, etc.
```

### 5.6 Domain Processor Duplication

#### Problem: Domain-specific processors have overlapping concerns

**Location:** `xpt_module/domain_processors/`
- `ae.py`: 120 lines (SEQ, date handling, AEDECOD derivation)
- `cm.py`: 95 lines (SEQ, date handling, CMDECOD derivation)
- `ex.py`: 110 lines (SEQ, date handling, EXDOSE derivation)
- `dm.py`: 180 lines (RFSTDTC derivation, age calculation)

**Common patterns:**
1. SEQ assignment: `groupby("USUBJID").cumcount() + 1`
2. Date handling: ISO 8601 formatting
3. *DECOD derivation from *TERM using codelists
4. Study day calculation from RFSTDTC

**Duplication percentage:** ~40% shared logic

**Risk:**
- SEQ logic differs slightly across domains
- Date handling inconsistencies

**Proposed fix:** Base processor with mixins
```python
# xpt_module/domain_processors/base.py
class SDTMProcessor(ABC):
    def assign_sequence(self, df: pd.DataFrame) -> pd.DataFrame:
        """Standard SEQ assignment."""
    
    def format_dates(self, df: pd.DataFrame) -> pd.DataFrame:
        """Standard date formatting."""
```

### 5.7 Conflicting Defaults & Magic Values

#### Examples of conflicts:

| Concept | Location 1 | Location 2 | Location 3 |
|---------|-----------|-----------|-----------|
| Default date | `2023-01-01` (study_orchestration_service.py:406) | `2023-01-01` (domain_synthesis_coordinator.py:159) | No default (io_module) |
| Default subject | `SYNTH001` (domain_synthesis_coordinator.py:159) | `"001"` (trial_design_service.py:45) | N/A |
| Min confidence | `0.5` (study.py:107) | `0.5` (domain_processing_coordinator.py:65) | `0.7` (engine.py:42) |
| Chunk size | `1000` (study.py:99) | Not used | N/A |
| Max XPT label | `200` chars (xpt_module/writer.py:85) | Not documented | N/A |

**Risk:** Changing a default in one place may not affect others

**Proposed fix:** Single constants module
```python
# constants.py (new, top-level)
class Defaults:
    DATE = "2023-01-01"
    SUBJECT_ID = "SYNTH001"
    MIN_CONFIDENCE = 0.5
    CHUNK_SIZE = 1000

class Constraints:
    XPT_MAX_LABEL_LENGTH = 200
    XPT_MAX_VARIABLES = 40
    QNAM_MAX_LENGTH = 8
```

---

## 6. Complexity Hotspots

### 6.1 Top Complex Functions

#### 1. `study_command()` (study.py:113-596)
- **Lines:** 483
- **Cyclomatic complexity:** ~40
- **Nested loops:** 3 levels
- **Responsibilities:**
  - Argument parsing
  - Initialization
  - File discovery orchestration
  - Domain processing loop
  - Synthesis orchestration
  - Define-XML generation
  - Error handling
  - Summary display

**Why complex:**
- God function antipattern
- Mixes orchestration with business logic
- Hard to test individual pieces
- 10+ local variables tracked

**Proposed refactor:** Extract to workflow orchestrator
```python
class StudyWorkflow:
    def __init__(self, config: StudyConfig):
        self.config = config
        self.context = WorkflowContext()
    
    def execute(self) -> StudyResult:
        self._initialize()
        self._discover_files()
        self._process_domains()
        self._synthesize_missing()
        self._generate_define_xml()
        return self._build_result()
```

#### 2. `reshape_lb_to_long()` (study_orchestration_service.py:201-406)
- **Lines:** 206
- **Complexity:** High
- **Issues:**
  - 15 regex patterns for column matching
  - Nested helper functions
  - Complex control flow for missing data
  - 3 levels of nested loops

**Why complex:**
- Handles multiple input formats (wide variations)
- Special cases for different column naming conventions
- Inline CT lookup logic

**Proposed refactor:** Pattern-based parser
```python
class LabTestParser:
    def __init__(self, ct_loader: CTLoader):
        self.patterns = self._build_patterns()
        self.ct_loader = ct_loader
    
    def parse_columns(self, columns: list[str]) -> dict[str, TestDefinition]:
        """Parse test definitions from column names."""
```

#### 3. `process_and_merge_domain()` (domain_processing_coordinator.py:55-621)
- **Lines:** 566
- **Complexity:** Very High
- **Responsibilities:**
  - File loading loop
  - Transformation dispatch
  - Mapping configuration
  - DataFrame building
  - Merging and deduplication
  - SUPPQUAL generation
  - File output generation

**Why complex:**
- Too many responsibilities
- Long parameter list (14 params)
- Mixes coordination with implementation details

**Proposed refactor:** Pipeline architecture
```python
class DomainProcessingPipeline:
    def __init__(self, domain_code: str):
        self.stages = [
            LoadStage(),
            TransformStage(),
            MapStage(),
            BuildStage(),
            MergeStage(),
            OutputStage(),
        ]
    
    def process(self, inputs: list[Path]) -> ProcessingResult:
        context = PipelineContext()
        for stage in self.stages:
            context = stage.execute(context)
        return context.result
```

#### 4. `build_domain_dataframe()` (xpt_module/builder.py:42-350)
- **Lines:** 308
- **Complexity:** Very High
- **Issues:**
  - Domain processor dispatch
  - Date transformation logic
  - Codelist application
  - Study day calculation
  - Validation logic

**Why complex:**
- Central chokepoint for all transformations
- Domain-specific logic interleaved with generic logic
- Hard to extend for new domains

**Proposed refactor:** Transformation chain
```python
class TransformationChain:
    def __init__(self, domain: SDTMDomain):
        self.transformers = [
            MappingTransformer(),
            DateTransformer(),
            CodelistTransformer(),
            StudyDayTransformer(),
            SequenceTransformer(),
            ValidationTransformer(),
        ]
    
    def apply(self, frame: pd.DataFrame, config: MappingConfig) -> pd.DataFrame:
        for transformer in self.transformers:
            frame = transformer.transform(frame, config)
        return frame
```

### 6.2 Module Coupling Analysis

```
High coupling (>5 imports):
- study.py: imports from 10 different modules
- domain_processing_coordinator.py: imports from 9 modules
- builder.py: imports from 7 modules

Tight coupling chains:
study.py 
  → domain_processing_coordinator.py
    → study_orchestration_service.py
      → terminology_module
      → domains_module

Circular import risks:
- cli.logging_config ← services.* (get_logger)
- domains_module ← xpt_module.builder ← domains_module
```

**Proposed fix:** Dependency injection
```python
# Instead of:
from ..cli.logging_config import get_logger
logger = get_logger()

# Use:
class DomainProcessor:
    def __init__(self, logger: Logger):
        self.logger = logger
```

### 6.3 Test Coverage Gaps

Based on directory structure review (no `tests/` directory found in repository root), test infrastructure appears to be absent.

**Critical gaps identified:**
1. No unit tests for transformations
2. No integration tests for workflow
3. No validation tests for SDTM compliance
4. No regression tests for file output

*Note: This assessment is based on directory structure inspection. Verify with `find . -name "*test*.py"` or `pytest --collect-only` before implementation.*

**Proposed test structure:**
```
tests/
├── unit/
│   ├── test_transformations.py
│   ├── test_mapping_engine.py
│   └── test_file_writers.py
├── integration/
│   ├── test_study_workflow.py
│   └── test_domain_processing.py
├── fixtures/
│   └── sample_study/
│       ├── DM.csv
│       ├── AE.csv
│       └── Items.csv
└── validation/
    └── test_sdtm_compliance.py
```

---

## Summary of Key Risks

### High Priority
1. **No tests** - Cannot safely refactor without test coverage
2. **God function antipattern** - `study_command()` is too complex
3. **Duplicated file I/O** - Inconsistent CSV reading across modules
4. **Magic constants** - Defaults scattered throughout codebase

### Medium Priority
5. **Transformation duplication** - VS and LB transformations are ~60% identical
6. **Global logger state** - Singleton pattern with mutation across modules
7. **Configuration management** - No central config, hardcoded paths

### Low Priority
8. **Module coupling** - High import counts, potential circular dependencies
9. **Domain processor duplication** - Similar logic across AE/CM/EX processors

---

## Appendix: Module Responsibility Matrix

| Module | Primary Responsibility | Dependencies | Lines |
|--------|----------------------|--------------|-------|
| cli/commands/study.py | Main workflow orchestration | 10 modules | 596 |
| services/domain_processing_coordinator.py | Domain file processing | 9 modules | 622 |
| services/study_orchestration_service.py | Domain transformations | 6 modules | 619 |
| xpt_module/builder.py | DataFrame construction | 7 modules | 350 |
| services/domain_synthesis_coordinator.py | Missing domain generation | 4 modules | 389 |
| cli/logging_config.py | Structured logging | 2 modules | 602 |
| mapping_module/engine.py | Column fuzzy matching | 3 modules | 280 |
| io_module/readers.py | CSV file reading | 1 module | 70 |
| metadata_module/loaders.py | Metadata file loading | 2 modules | 250 |

---

**Document Version:** 1.0  
**Last Updated:** 2025-12-14  
**Next Steps:** See companion "To-Be" architecture document
