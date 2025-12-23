"""Unit tests for constants module.

Tests validate that constants are defined correctly and
remain consistent with SDTM specifications.
"""

import re

from cdisc_transpiler.constants import (
    Constraints,
    Defaults,
    LogLevels,
    MetadataFiles,
    Patterns,
    SDTMVersions,
)


class TestDefaults:
    """Test suite for Defaults class."""

    def test_date_format(self):
        """Test that default date is in ISO 8601 format."""
        pattern = re.compile(r"^\d{4}-\d{2}-\d{2}$")
        assert pattern.match(Defaults.DATE), "DATE must be ISO 8601 (YYYY-MM-DD)"

    def test_subject_id_format(self):
        """Test that default subject ID matches expected pattern."""
        assert len(Defaults.SUBJECT_ID) > 0, "SUBJECT_ID must not be empty"
        assert Defaults.SUBJECT_ID.isalnum(), "SUBJECT_ID should be alphanumeric"

    def test_min_confidence_range(self):
        """Test that minimum confidence is in valid range."""
        assert 0.0 <= Defaults.MIN_CONFIDENCE <= 1.0, "MIN_CONFIDENCE must be 0.0-1.0"

    def test_chunk_size_positive(self):
        """Test that chunk size is positive."""
        assert Defaults.CHUNK_SIZE > 0, "CHUNK_SIZE must be positive"

    def test_output_format_values(self):
        """Test that output format is valid."""
        valid_formats = {"xpt", "xml", "both"}
        assert Defaults.OUTPUT_FORMAT in valid_formats, (
            f"OUTPUT_FORMAT must be one of {valid_formats}"
        )

    def test_boolean_flags(self):
        """Test that boolean flags are actually booleans."""
        assert isinstance(Defaults.GENERATE_DEFINE, bool)
        assert isinstance(Defaults.GENERATE_SAS, bool)


class TestConstraints:
    """Test suite for Constraints class."""

    def test_xpt_label_length_constraint(self):
        """Test XPT label length matches SAS specification."""
        # SAS Transport V5 specification: max 200 characters
        assert Constraints.XPT_MAX_LABEL_LENGTH == 200
        assert Constraints.XPT_MAX_LABEL_LENGTH > 0

    def test_xpt_variables_constraint(self):
        """Test XPT variables limit matches SAS specification."""
        # SAS Transport V5 specification: max 40 variables per dataset
        assert Constraints.XPT_MAX_VARIABLES == 40
        assert Constraints.XPT_MAX_VARIABLES > 0

    def test_xpt_name_length_constraint(self):
        """Test XPT variable name length matches SAS specification."""
        # SAS specification: max 8 characters for variable names
        assert Constraints.XPT_MAX_NAME_LENGTH == 8
        assert Constraints.XPT_MAX_NAME_LENGTH > 0

    def test_qnam_length_constraint(self):
        """Test QNAM length matches SDTM specification."""
        # SDTMIG v3.4: QNAM in SUPPQUAL is max 8 characters
        assert Constraints.QNAM_MAX_LENGTH == 8
        assert Constraints.QNAM_MAX_LENGTH > 0

    def test_studyid_length_constraint(self):
        """Test STUDYID length matches SDTM specification."""
        # SDTMIG v3.4: STUDYID is max 20 characters
        assert Constraints.STUDYID_MAX_LENGTH == 20
        assert Constraints.STUDYID_MAX_LENGTH > 0

    def test_domain_length_constraint(self):
        """Test DOMAIN code length matches SDTM specification."""
        # SDTMIG v3.4: DOMAIN is max 2 characters
        assert Constraints.DOMAIN_MAX_LENGTH == 2
        assert Constraints.DOMAIN_MAX_LENGTH > 0

    def test_define_xml_version(self):
        """Test Define-XML version string format."""
        pattern = re.compile(r"^\d+\.\d+\.\d+$")
        assert pattern.match(Constraints.DEFINE_XML_VERSION), (
            "Version must be X.Y.Z format"
        )

    def test_dataset_xml_version(self):
        """Test Dataset-XML version string format."""
        pattern = re.compile(r"^\d+\.\d+\.\d+$")
        assert pattern.match(Constraints.DATASET_XML_VERSION), (
            "Version must be X.Y.Z format"
        )


class TestPatterns:
    """Test suite for regex patterns."""

    def test_sdtm_variable_name_pattern(self):
        """Test SDTM variable name pattern against valid/invalid names."""
        pattern = re.compile(Patterns.SDTM_VARIABLE_NAME)

        # Valid names
        valid_names = [
            "USUBJID",
            "AESEQ",
            "DOMAIN",
            "STUDYID",
            "AETERM",
            "AESTDTC",
            "VS_TEST",  # With underscore
            "A",  # Single character
            "A1234567",  # 8 characters (max)
        ]
        for name in valid_names:
            assert pattern.match(name), f"{name} should be valid SDTM variable name"

        # Invalid names
        invalid_names = [
            "usubjid",  # Lowercase
            "1AESEQ",  # Starts with number
            "TOOLONGNAME",  # > 8 characters
            "TEST-VAR",  # Invalid character (dash)
            "TEST VAR",  # Space
            "",  # Empty
        ]
        for name in invalid_names:
            assert not pattern.match(name), (
                f"{name} should be invalid SDTM variable name"
            )

    def test_iso_date_full_pattern(self):
        """Test ISO 8601 full date pattern."""
        pattern = re.compile(Patterns.ISO_DATE_FULL)

        # Valid dates
        assert pattern.match("2023-01-01")
        assert pattern.match("2023-12-31")
        assert pattern.match("1990-06-15")

        # Invalid dates
        assert not pattern.match("2023-1-1")  # No padding
        assert not pattern.match("23-01-01")  # 2-digit year
        assert not pattern.match("2023/01/01")  # Wrong delimiter
        assert not pattern.match("2023-01")  # Partial

    def test_iso_date_partial_patterns(self):
        """Test ISO 8601 partial date patterns."""
        month_pattern = re.compile(Patterns.ISO_DATE_PARTIAL_MONTH)
        year_pattern = re.compile(Patterns.ISO_DATE_PARTIAL_YEAR)

        # Valid partial dates
        assert month_pattern.match("2023-01")
        assert month_pattern.match("2023-12")
        assert year_pattern.match("2023")
        assert year_pattern.match("1990")

        # Invalid partial dates
        assert not month_pattern.match("2023-1")  # No padding
        assert not year_pattern.match("23")  # 2-digit year

    def test_testcd_pattern(self):
        """Test TESTCD pattern for Findings domains."""
        pattern = re.compile(Patterns.TESTCD_PATTERN)

        # Valid test codes
        assert pattern.match("HR")
        assert pattern.match("SYSBP")
        assert pattern.match("DIABP")
        assert pattern.match("TEMP")
        assert pattern.match("A")
        assert pattern.match("A1234567")  # 8 chars

        # Invalid test codes
        assert not pattern.match("hr")  # Lowercase
        assert not pattern.match("1HR")  # Starts with number
        assert not pattern.match("TOOLONGNAME")  # > 8 chars

    def test_usubjid_pattern(self):
        """Test USUBJID pattern."""
        pattern = re.compile(Patterns.USUBJID_PATTERN)

        # Valid subject IDs
        assert pattern.match("001")
        assert pattern.match("ABC-001")
        assert pattern.match("STUDY_001")
        assert pattern.match("S001-P001")

        # Invalid subject IDs (with special characters)
        assert not pattern.match("001@ABC")
        assert not pattern.match("001 ABC")

    def test_code_row_pattern(self):
        """Test code row detection pattern."""
        pattern = re.compile(Patterns.CODE_ROW_PATTERN)

        # Valid code row values
        assert pattern.match("USUBJID")
        assert pattern.match("TestCode")
        assert pattern.match("Test_Code")
        assert pattern.match("A")

        # Invalid (not code rows)
        assert not pattern.match("usubjid")  # Lowercase start
        assert not pattern.match("Test Code")  # Space
        assert not pattern.match("1TEST")  # Starts with number


class TestMetadataFiles:
    """Test suite for MetadataFiles class."""

    def test_file_names_defined(self):
        """Test that standard file names are defined."""
        assert MetadataFiles.ITEMS == "Items.csv"
        assert MetadataFiles.CODELISTS == "CodeLists.csv"
        assert MetadataFiles.README == "README.txt"

    def test_skip_patterns_defined(self):
        """Test that skip patterns are defined."""
        assert isinstance(MetadataFiles.SKIP_PATTERNS, list)
        assert len(MetadataFiles.SKIP_PATTERNS) > 0

        # Check some expected patterns
        assert "CODELISTS" in MetadataFiles.SKIP_PATTERNS
        assert "ITEMS" in MetadataFiles.SKIP_PATTERNS

    def test_skip_patterns_uppercase(self):
        """Test that skip patterns are uppercase for case-insensitive matching."""
        for pattern in MetadataFiles.SKIP_PATTERNS:
            assert pattern.isupper(), f"Skip pattern '{pattern}' should be uppercase"


class TestSDTMVersions:
    """Test suite for SDTMVersions class."""

    def test_default_version_defined(self):
        """Test that default version is defined."""
        assert SDTMVersions.DEFAULT_VERSION == "3.4"

    def test_supported_versions_list(self):
        """Test that supported versions list is defined."""
        assert isinstance(SDTMVersions.SUPPORTED_VERSIONS, list)
        assert len(SDTMVersions.SUPPORTED_VERSIONS) > 0

    def test_default_version_in_supported(self):
        """Test that default version is in supported versions."""
        assert SDTMVersions.DEFAULT_VERSION in SDTMVersions.SUPPORTED_VERSIONS

    def test_context_values_defined(self):
        """Test that Define-XML context values are defined."""
        assert SDTMVersions.DEFINE_CONTEXT_SUBMISSION == "Submission"
        assert SDTMVersions.DEFINE_CONTEXT_OTHER == "Other"


class TestLogLevels:
    """Test suite for LogLevels class."""

    def test_log_levels_defined(self):
        """Test that log levels are defined."""
        assert LogLevels.NORMAL == 0
        assert LogLevels.VERBOSE == 1
        assert LogLevels.DEBUG == 2

    def test_log_levels_ordered(self):
        """Test that log levels are in increasing order."""
        assert LogLevels.NORMAL < LogLevels.VERBOSE < LogLevels.DEBUG

    def test_log_levels_non_negative(self):
        """Test that log levels are non-negative."""
        assert LogLevels.NORMAL >= 0
        assert LogLevels.VERBOSE >= 0
        assert LogLevels.DEBUG >= 0
