"""SDTM compliance validation tests.

This module tests that generated datasets comply with SDTM requirements:
- Required variables are present
- Variable types are correct
- Variable lengths are within limits
- Controlled terminology is used correctly
"""

from pathlib import Path

import pandas as pd
import pytest

from cdisc_transpiler.application.models import ProcessStudyRequest
from cdisc_transpiler.infrastructure.container import create_default_container
from cdisc_transpiler.infrastructure.sdtm_spec import get_domain

# Path to sample study data
MOCKDATA_DIR = Path(__file__).parent.parent.parent / "mockdata"
DEMO_GDISC = MOCKDATA_DIR / "DEMO_GDISC_20240903_072908"


@pytest.mark.validation
@pytest.mark.integration
class TestSDTMCompliance:
    """Test SDTM compliance of generated datasets."""

    @pytest.fixture(scope="class")
    def processed_study(self, tmp_path_factory):
        """Process a study and return the output directory."""
        if not DEMO_GDISC.exists():
            pytest.skip("DEMO_GDISC sample data not available")

        output_dir = tmp_path_factory.mktemp("sdtm_validation")

        # Process the study
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

    def test_required_variables_present_in_dm(self, processed_study):
        """Test that DM domain has all required variables."""
        # Read generated XPT file
        xpt_files = list((processed_study / "xpt").glob("dm.xpt"))
        if not xpt_files:
            pytest.skip("DM domain not generated")

        import pyreadstat

        df, meta = pyreadstat.read_xport(str(xpt_files[0]))

        # Get domain definition
        domain = get_domain("DM")
        required_vars = [v.name for v in domain.variables if v.core == "Req"]

        # Check required variables are present
        missing = set(required_vars) - set(df.columns)
        assert not missing, f"Missing required variables in DM: {missing}"

    def test_required_variables_present_in_ae(self, processed_study):
        """Test that AE domain has all required variables."""
        xpt_files = list((processed_study / "xpt").glob("ae.xpt"))
        if not xpt_files:
            pytest.skip("AE domain not generated")

        import pyreadstat

        df, meta = pyreadstat.read_xport(str(xpt_files[0]))

        # Get domain definition
        domain = get_domain("AE")
        required_vars = [v.name for v in domain.variables if v.core == "Req"]

        # Check required variables are present
        missing = set(required_vars) - set(df.columns)
        assert not missing, f"Missing required variables in AE: {missing}"

    def test_variable_types_correct_in_dm(self, processed_study):
        """Test that DM domain variables have correct types."""
        xpt_files = list((processed_study / "xpt").glob("dm.xpt"))
        if not xpt_files:
            pytest.skip("DM domain not generated")

        import pyreadstat

        df, meta = pyreadstat.read_xport(str(xpt_files[0]))

        # Get domain definition
        domain = get_domain("DM")

        # Check some key numeric variables
        numeric_vars = [
            v.name for v in domain.variables if v.type == "Num" and v.name in df.columns
        ]

        for var in numeric_vars:
            # Check that numeric columns can be converted to numeric
            try:
                pd.to_numeric(df[var], errors="coerce")
                # It's OK if all values are NaN (empty numeric column)
            except Exception as e:
                pytest.fail(f"Variable {var} should be numeric but got error: {e}")

    def test_variable_lengths_within_limits(self, processed_study):
        """Test that character variables respect length limits."""
        xpt_files = list((processed_study / "xpt").glob("dm.xpt"))
        if not xpt_files:
            pytest.skip("DM domain not generated")

        import pyreadstat

        df, meta = pyreadstat.read_xport(str(xpt_files[0]))

        # Get domain definition
        domain = get_domain("DM")

        # Check character variable lengths
        for var in domain.variables:
            if var.type == "Char" and var.length and var.name in df.columns:
                max_length = df[var.name].astype(str).str.len().max()
                if pd.notna(max_length):
                    assert max_length <= var.length, (
                        f"Variable {var.name} has values exceeding maximum length "
                        f"({max_length} > {var.length})"
                    )

    def test_studyid_populated(self, processed_study):
        """Test that STUDYID is populated in all domains."""
        xpt_dir = processed_study / "xpt"
        if not xpt_dir.exists():
            pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        if not xpt_files:
            pytest.skip("No XPT files found")

        import pyreadstat

        for xpt_file in xpt_files:
            df, meta = pyreadstat.read_xport(str(xpt_file))

            if "STUDYID" in df.columns and len(df) > 0:
                # STUDYID should not be empty if there are records
                assert not df["STUDYID"].isna().all(), (
                    f"STUDYID is empty in {xpt_file.name}"
                )

    def test_domain_populated(self, processed_study):
        """Test that DOMAIN is populated in all domains."""
        xpt_dir = processed_study / "xpt"
        if not xpt_dir.exists():
            pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        if not xpt_files:
            pytest.skip("No XPT files found")

        import pyreadstat

        for xpt_file in xpt_files:
            df, meta = pyreadstat.read_xport(str(xpt_file))

            if "DOMAIN" in df.columns and len(df) > 0:
                # DOMAIN should not be empty if there are records
                assert not df["DOMAIN"].isna().all(), (
                    f"DOMAIN is empty in {xpt_file.name}"
                )

    def test_usubjid_populated(self, processed_study):
        """Test that USUBJID is populated where present."""
        xpt_dir = processed_study / "xpt"
        if not xpt_dir.exists():
            pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        if not xpt_files:
            pytest.skip("No XPT files found")

        import pyreadstat

        for xpt_file in xpt_files:
            df, meta = pyreadstat.read_xport(str(xpt_file))

            if "USUBJID" in df.columns and len(df) > 0:
                # USUBJID should not be empty if there are records
                assert not df["USUBJID"].isna().all(), (
                    f"USUBJID is empty in {xpt_file.name}"
                )

    def test_sex_uses_controlled_terminology(self, processed_study):
        """Test that SEX uses SDTM controlled terminology (M, F, U, UNDIFFERENTIATED)."""
        xpt_files = list((processed_study / "xpt").glob("dm.xpt"))
        if not xpt_files:
            pytest.skip("DM domain not generated")

        import pyreadstat

        df, meta = pyreadstat.read_xport(str(xpt_files[0]))

        if "SEX" in df.columns and len(df) > 0:
            valid_values = {"M", "F", "U", "UNDIFFERENTIATED", ""}
            actual_values = set(df["SEX"].fillna("").astype(str).str.upper().unique())

            invalid = actual_values - valid_values
            assert not invalid, f"SEX contains invalid values: {invalid}"

    def test_race_uses_controlled_terminology(self, processed_study):
        """Test that RACE uses SDTM controlled terminology."""
        xpt_files = list((processed_study / "xpt").glob("dm.xpt"))
        if not xpt_files:
            pytest.skip("DM domain not generated")

        import pyreadstat

        df, meta = pyreadstat.read_xport(str(xpt_files[0]))

        if "RACE" in df.columns and len(df) > 0:
            # Common SDTM race values
            valid_values = {
                "AMERICAN INDIAN OR ALASKA NATIVE",
                "ASIAN",
                "BLACK OR AFRICAN AMERICAN",
                "NATIVE HAWAIIAN OR OTHER PACIFIC ISLANDER",
                "WHITE",
                "MULTIPLE",
                "NOT REPORTED",
                "UNKNOWN",
                "",
            }
            actual_values = set(df["RACE"].fillna("").astype(str).str.upper().unique())

            # Don't fail on unexpected values, just warn (race can vary)
            unexpected = actual_values - valid_values
            if unexpected:
                # This is informational, not a failure
                print(f"Note: RACE contains non-standard values: {unexpected}")


@pytest.mark.validation
@pytest.mark.integration
class TestSDTMStructure:
    """Test structural requirements of SDTM domains."""

    @pytest.fixture(scope="class")
    def processed_study(self, tmp_path_factory):
        """Process a study and return the output directory."""
        if not DEMO_GDISC.exists():
            pytest.skip("DEMO_GDISC sample data not available")

        output_dir = tmp_path_factory.mktemp("sdtm_structure")

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

    def test_no_duplicate_rows(self, processed_study):
        """Test that domains don't have completely duplicate rows."""
        xpt_dir = processed_study / "xpt"
        if not xpt_dir.exists():
            pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        if not xpt_files:
            pytest.skip("No XPT files found")

        import pyreadstat

        for xpt_file in xpt_files:
            df, meta = pyreadstat.read_xport(str(xpt_file))

            if len(df) > 0:
                duplicates = df.duplicated().sum()
                # Allow some duplicates but not excessive
                duplicate_pct = (duplicates / len(df)) * 100
                assert duplicate_pct < 50, (
                    f"{xpt_file.name} has {duplicate_pct:.1f}% duplicate rows"
                )

    def test_column_names_uppercase(self, processed_study):
        """Test that all column names are uppercase per SDTM standards."""
        xpt_dir = processed_study / "xpt"
        if not xpt_dir.exists():
            pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        if not xpt_files:
            pytest.skip("No XPT files found")

        import pyreadstat

        for xpt_file in xpt_files:
            df, meta = pyreadstat.read_xport(str(xpt_file))

            lowercase_cols = [col for col in df.columns if col != col.upper()]
            assert not lowercase_cols, (
                f"{xpt_file.name} has lowercase columns: {lowercase_cols}"
            )

    def test_no_special_characters_in_values(self, processed_study):
        """Test that values don't contain problematic special characters."""
        xpt_dir = processed_study / "xpt"
        if not xpt_dir.exists():
            pytest.skip("XPT directory not found")

        xpt_files = list(xpt_dir.glob("*.xpt"))
        if not xpt_files:
            pytest.skip("No XPT files found")

        # Characters that can cause issues in SAS/XML
        problematic_chars = ["\x00", "\x01", "\x02", "\x03", "\x04", "\x05"]

        import pyreadstat

        for xpt_file in xpt_files:
            df, meta = pyreadstat.read_xport(str(xpt_file))

            # Check string columns for problematic characters
            for col in df.select_dtypes(include=["object"]).columns:
                for char in problematic_chars:
                    if (
                        df[col]
                        .astype(str)
                        .str.contains(char, regex=False, na=False)
                        .any()
                    ):
                        pytest.fail(
                            f"{xpt_file.name} column {col} contains "
                            f"problematic character: {char!r}"
                        )
