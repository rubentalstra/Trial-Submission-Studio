from typing import ClassVar


class Defaults:
    DATE = "2023-01-01"
    SUBJECT_ID = "SYNTH001"
    ELEMENT_DURATION = "P1D"
    MIN_CONFIDENCE = 0.5
    CHUNK_SIZE = 1000
    OUTPUT_FORMAT = "both"
    GENERATE_DEFINE = True
    GENERATE_SAS = True
    OPERATIONAL_COLUMN_MIN_COUNT = 3
    OPERATIONAL_COLUMN_COMMON_FRACTION = 0.5


class Constraints:
    XPT_MAX_LABEL_LENGTH = 200
    XPT_MAX_VARIABLES = 40
    XPT_MAX_NAME_LENGTH = 8
    QNAM_MAX_LENGTH = 8
    STUDYID_MAX_LENGTH = 20
    DOMAIN_MAX_LENGTH = 2
    DEFINE_XML_VERSION = "2.1.0"
    DATASET_XML_VERSION = "1.0.0"


class Patterns:
    SDTM_VARIABLE_NAME = "^[A-Z][A-Z0-9_]{0,7}$"
    ISO_DATE_FULL = "^\\d{4}-\\d{2}-\\d{2}$"
    ISO_DATE_PARTIAL_MONTH = "^\\d{4}-\\d{2}$"
    ISO_DATE_PARTIAL_YEAR = "^\\d{4}$"
    TESTCD_PATTERN = "^[A-Z][A-Z0-9]{0,7}$"
    USUBJID_PATTERN = "^[A-Za-z0-9_-]+$"
    CODE_ROW_PATTERN = "^[A-Z][A-Za-z0-9_]*$"


class MetadataFiles:
    ITEMS = "Items.csv"
    CODELISTS = "CodeLists.csv"
    README = "README.txt"
    SKIP_PATTERNS: ClassVar[tuple[str, ...]] = (
        "CODELISTS",
        "CODELIST",
        "ITEMS",
        "README",
        "METADATA",
    )


class SDTMVersions:
    DEFAULT_VERSION = "3.4"
    SUPPORTED_VERSIONS: ClassVar[tuple[str, ...]] = ("3.4",)
    DEFINE_CONTEXT_SUBMISSION = "Submission"
    DEFINE_CONTEXT_OTHER = "Other"


class LogLevels:
    NORMAL = 0
    VERBOSE = 1
    DEBUG = 2


class MissingValues:
    STRING_MARKERS: ClassVar[frozenset[str]] = frozenset(
        {"NAN", "<NA>", "NONE", "NULL"}
    )
