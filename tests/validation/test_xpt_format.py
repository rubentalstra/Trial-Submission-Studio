"""XPT format validation tests.

This module tests that generated XPT files:
- Can be read by standard tools (pyreadstat/SAS)
- Have correct structure and metadata
- Comply with SAS Transport File (XPORT) format specifications
"""

from pathlib import Path

import pyreadstat
import pytest

from cdisc_transpiler.application.models import ProcessStudyRequest
from cdisc_transpiler.infrastructure import create_default_container

# Path to sample study data
MOCKDATA_DIR = Path(__file__).parent.parent.parent / "mockdata"
DEMO_GDISC = MOCKDATA_DIR / "DEMO_GDISC_20240903_072908"


@pytest.mark.validation
@pytest.mark.integration
class TestXPTFormatReadability:
    """Test that XPT files can be read by standard tools."""

    @pytest.fixture(scope="class")
    def processed_study(self, tmp_path_factory):
        """Process a study and return the output directory."""
        # if not DEMO_GDISC.exists():
        #     pytest.skip("DEMO_GDISC sample data not available")

        output_dir = tmp_path_factory.mktemp("xpt_validation")

        container = create_default_container(verbose=0)
        use_case = container.create_study_processing_use_case()

        request = ProcessStudyRequest(
            study_folder=DEMO_GDISC,
            study_id="DEMO_GDISC",
            output_dir=output_dir,
            output_formats=["xpt"],
            generate_define_xml=False,
            generate_sas=False,
            sdtm_version="3.2",
        )

        response = use_case.execute(request)
        assert response.success, "Study processing should succeed"

        return output_dir

    def test_xpt_files_exist(self, processed_study):
        """Test that XPT files were created."""
        xpt_dir = processed_study / "xpt"
        assert xpt_dir.exists(), "XPT directory should exist"

        xpt_files = list(xpt_dir.glob("*.xpt"))
        assert len(xpt_files) > 0, "Should have at least one XPT file"

    def test_xpt_files_readable_by_pyreadstat(self, processed_study):
        """Test that all XPT files can be read by pyreadstat."""
        xpt_dir = processed_study / "xpt"
        # if not xpt_dir.exists():
        #     pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        # if not xpt_files:
        #     pytest.skip("No XPT files found")

        for xpt_file in xpt_files:
            try:
                df, meta = pyreadstat.read_xport(str(xpt_file))
                assert df is not None, f"Failed to read {xpt_file.name}"
                assert len(df.columns) > 0, f"{xpt_file.name} has no columns"
            except Exception as e:
                pytest.fail(f"Failed to read {xpt_file.name}: {e}")

    def test_xpt_file_sizes_reasonable(self, processed_study):
        """Test that XPT files have reasonable sizes (not too small or suspiciously large)."""
        xpt_dir = processed_study / "xpt"
        # if not xpt_dir.exists():
        #     pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        # if not xpt_files:
        #     pytest.skip("No XPT files found")

        for xpt_file in xpt_files:
            size = xpt_file.stat().st_size

            # Minimum size (header alone is ~1KB)
            assert size > 1000, f"{xpt_file.name} is too small ({size} bytes)"

            # Maximum size (100MB is very large for SDTM)
            assert size < 100 * 1024 * 1024, (
                f"{xpt_file.name} is suspiciously large ({size / 1024 / 1024:.1f} MB)"
            )


@pytest.mark.validation
@pytest.mark.integration
class TestXPTMetadata:
    """Test XPT file metadata and structure."""

    @pytest.fixture(scope="class")
    def processed_study(self, tmp_path_factory):
        """Process a study and return the output directory."""
        # if not DEMO_GDISC.exists():
        #     pytest.skip("DEMO_GDISC sample data not available")

        output_dir = tmp_path_factory.mktemp("xpt_metadata")

        container = create_default_container(verbose=0)
        use_case = container.create_study_processing_use_case()

        request = ProcessStudyRequest(
            study_folder=DEMO_GDISC,
            study_id="DEMO_GDISC",
            output_dir=output_dir,
            output_formats=["xpt"],
            generate_define_xml=False,
            generate_sas=False,
            sdtm_version="3.2",
        )

        response = use_case.execute(request)
        assert response.success, "Study processing should succeed"

        return output_dir

    def test_xpt_has_column_labels(self, processed_study):
        """Test that XPT files have column labels (metadata)."""
        xpt_dir = processed_study / "xpt"
        # if not xpt_dir.exists():
        #     pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        # if not xpt_files:
        #     pytest.skip("No XPT files found")

        for xpt_file in xpt_files:
            df, meta = pyreadstat.read_xport(str(xpt_file))

            # Check that column labels exist
            assert hasattr(meta, "column_labels"), (
                f"{xpt_file.name} metadata missing column_labels"
            )

            # At least some columns should have labels
            labels = [label for label in meta.column_labels if label]
            assert len(labels) > 0, f"{xpt_file.name} has no column labels"

    def test_xpt_column_names_valid(self, processed_study):
        """Test that XPT column names are valid SAS names."""
        xpt_dir = processed_study / "xpt"
        # if not xpt_dir.exists():
        #     pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        # if not xpt_files:
        #     pytest.skip("No XPT files found")

        for xpt_file in xpt_files:
            df, meta = pyreadstat.read_xport(str(xpt_file))

            for col in df.columns:
                # SAS variable names: 1-32 chars, start with letter/underscore
                assert len(col) <= 32, (
                    f"{xpt_file.name} column {col} exceeds 32 characters"
                )
                assert col[0].isalpha() or col[0] == "_", (
                    f"{xpt_file.name} column {col} doesn't start with letter/underscore"
                )
                # Only alphanumeric and underscore allowed
                assert col.replace("_", "").isalnum(), (
                    f"{xpt_file.name} column {col} contains invalid characters"
                )

    def test_xpt_has_table_name(self, processed_study):
        """Test that XPT files have table/member names set."""
        xpt_dir = processed_study / "xpt"
        # if not xpt_dir.exists():
        #     pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        # if not xpt_files:
        #     pytest.skip("No XPT files found")

        for xpt_file in xpt_files:
            df, meta = pyreadstat.read_xport(str(xpt_file))

            # Check that table name is set
            assert hasattr(meta, "table_name"), (
                f"{xpt_file.name} metadata missing table_name"
            )

            # Table name should not be empty
            if meta.table_name:
                assert len(meta.table_name) > 0, f"{xpt_file.name} has empty table_name"


@pytest.mark.validation
@pytest.mark.integration
class TestXPTDataIntegrity:
    """Test data integrity in XPT files."""

    @pytest.fixture(scope="class")
    def processed_study(self, tmp_path_factory):
        """Process a study and return the output directory."""
        # if not DEMO_GDISC.exists():
        #     pytest.skip("DEMO_GDISC sample data not available")

        output_dir = tmp_path_factory.mktemp("xpt_integrity")

        container = create_default_container(verbose=0)
        use_case = container.create_study_processing_use_case()

        request = ProcessStudyRequest(
            study_folder=DEMO_GDISC,
            study_id="DEMO_GDISC",
            output_dir=output_dir,
            output_formats=["xpt"],
            generate_define_xml=False,
            generate_sas=False,
            sdtm_version="3.2",
        )

        response = use_case.execute(request)
        assert response.success, "Study processing should succeed"

        return output_dir

    def test_xpt_data_not_empty(self, processed_study):
        """Test that XPT files contain data (not just headers)."""
        xpt_dir = processed_study / "xpt"
        # if not xpt_dir.exists():
        #     pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        # if not xpt_files:
        #     pytest.skip("No XPT files found")

        non_empty_count = 0
        for xpt_file in xpt_files:
            df, meta = pyreadstat.read_xport(str(xpt_file))

            if len(df) > 0:
                non_empty_count += 1

        # At least some files should have data
        assert non_empty_count > 0, "All XPT files are empty"

    def test_xpt_no_invalid_dates(self, processed_study):
        """Test that XPT files don't have obviously invalid dates."""
        xpt_dir = processed_study / "xpt"
        # if not xpt_dir.exists():
        #     pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        # if not xpt_files:
        #     pytest.skip("No XPT files found")

        for xpt_file in xpt_files:
            df, meta = pyreadstat.read_xport(str(xpt_file))

            # Check DTC columns (datetime) for valid formats
            dtc_cols = [col for col in df.columns if col.endswith("DTC")]

            for col in dtc_cols:
                if col in df.columns and len(df) > 0:
                    # Check for obviously invalid dates (year > 2100 or < 1900)
                    # This is a basic sanity check
                    values = df[col].dropna().astype(str)
                    if len(values) > 0:
                        for val in values:
                            if len(val) >= 4:
                                year_str = val[:4]
                                if year_str.isdigit():
                                    year = int(year_str)
                                    assert 1900 <= year <= 2100, (
                                        f"{xpt_file.name} {col} has invalid year: {year}"
                                    )

    def test_xpt_string_encoding(self, processed_study):
        """Test that XPT files handle string encoding correctly."""
        xpt_dir = processed_study / "xpt"
        # if not xpt_dir.exists():
        #     pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        # if not xpt_files:
        #     pytest.skip("No XPT files found")

        for xpt_file in xpt_files:
            try:
                # Try to read with default encoding
                df, meta = pyreadstat.read_xport(str(xpt_file))

                # Check that string columns don't have encoding errors
                for col in df.select_dtypes(include=["object"]).columns:
                    # Try to convert to string without errors
                    _ = df[col].astype(str)

            except UnicodeDecodeError as e:
                pytest.fail(f"{xpt_file.name} has encoding issues: {e}")
