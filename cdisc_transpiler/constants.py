"""Constants module for magic values and defaults.

This module centralizes all magic values that were previously scattered
throughout the codebase. Constants are organized by category and include
SDTM references where applicable.
"""

from __future__ import annotations


class Defaults:
    """Default values used throughout the application.
    
    These values can be overridden via configuration (see config.py).
    """
    
    # Date and subject defaults for synthesis
    DATE = "2023-01-01"  # ISO 8601 format
    SUBJECT_ID = "SYNTH001"
    
    # Trial design defaults for synthesis
    ELEMENT_DURATION = "P1D"  # ISO 8601 duration for trial elements (1 day)
    
    # Processing defaults
    MIN_CONFIDENCE = 0.5  # Minimum confidence for fuzzy matching
    CHUNK_SIZE = 1000  # Chunk size for streaming processing
    
    # Output format defaults
    OUTPUT_FORMAT = "both"  # "xpt", "xml", or "both"
    GENERATE_DEFINE = True
    GENERATE_SAS = True


class Constraints:
    """System constraints from SDTM and SAS specifications.
    
    These are hard limits that cannot be exceeded without violating
    SDTM or SAS format specifications.
    """
    
    # XPT (SAS Transport V5) constraints
    # Reference: SAS Transport File Format specification
    XPT_MAX_LABEL_LENGTH = 200  # Maximum label length in XPT files
    XPT_MAX_VARIABLES = 40  # Maximum variables per XPT file (SAS constraint)
    XPT_MAX_NAME_LENGTH = 8  # Maximum variable name length
    
    # SDTM constraints
    # Reference: SDTMIG v3.4
    QNAM_MAX_LENGTH = 8  # Maximum length for QNAM in SUPPQUAL
    STUDYID_MAX_LENGTH = 20  # Maximum length for STUDYID
    DOMAIN_MAX_LENGTH = 2  # Maximum length for DOMAIN code
    
    # Define-XML constraints
    # Reference: Define-XML 2.1 specification
    DEFINE_XML_VERSION = "2.1.0"
    DATASET_XML_VERSION = "1.0.0"


class Patterns:
    """Regular expression patterns used for validation and parsing.
    
    These patterns are used throughout the codebase for consistent
    pattern matching behavior.
    """
    
    # SDTM variable name pattern (alphanumeric + underscore, max 8 chars)
    SDTM_VARIABLE_NAME = r"^[A-Z][A-Z0-9_]{0,7}$"
    
    # ISO 8601 date patterns
    ISO_DATE_FULL = r"^\d{4}-\d{2}-\d{2}$"  # YYYY-MM-DD
    ISO_DATE_PARTIAL_MONTH = r"^\d{4}-\d{2}$"  # YYYY-MM
    ISO_DATE_PARTIAL_YEAR = r"^\d{4}$"  # YYYY
    
    # Test code patterns for Findings domains
    TESTCD_PATTERN = r"^[A-Z][A-Z0-9]{0,7}$"
    
    # Subject identifier pattern (alphanumeric + dash/underscore)
    USUBJID_PATTERN = r"^[A-Za-z0-9_-]+$"
    
    # Code row detection (for intelligent header detection)
    CODE_ROW_PATTERN = r"^[A-Z][A-Za-z0-9_]*$"


class MetadataFiles:
    """Standard metadata file names used in study folders.
    
    These files are optional but provide additional context
    for data processing when present.
    """
    
    ITEMS = "Items.csv"  # Column definitions
    CODELISTS = "CodeLists.csv"  # Controlled terminology
    README = "README.txt"  # Study documentation
    
    # Files to skip during domain discovery
    SKIP_PATTERNS = [
        "CODELISTS",
        "CODELIST",
        "ITEMS",
        "README",
        "METADATA",
    ]


class SDTMVersions:
    """SDTM version information.
    
    The transpiler is designed for SDTMIG v3.4 but may work
    with other versions.
    """
    
    DEFAULT_VERSION = "3.4"
    SUPPORTED_VERSIONS = ["3.4"]
    
    # Define-XML context
    DEFINE_CONTEXT_SUBMISSION = "Submission"
    DEFINE_CONTEXT_OTHER = "Other"


class LogLevels:
    """Logging verbosity levels.
    
    These map to CLI -v flags:
    - 0: Normal (essential progress only)
    - 1: Verbose (-v)
    - 2: Debug (-vv)
    """
    
    NORMAL = 0
    VERBOSE = 1
    DEBUG = 2
