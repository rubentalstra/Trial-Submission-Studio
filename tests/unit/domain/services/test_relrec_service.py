"""Tests for RELREC service.

These tests verify the RELREC service creates proper relationship records
linking observations across SDTM domains.
"""

import pytest
import pandas as pd

from cdisc_transpiler.domain.services import RelrecService, build_relrec


class TestRelrecService:
    """Tests for RelrecService."""

    def test_service_can_be_imported(self):
        """Test that RELREC service can be imported."""
        assert RelrecService is not None
        assert build_relrec is not None

    def test_service_instantiation(self):
        """Test that service can be instantiated."""
        service = RelrecService()
        assert service is not None

    def test_build_relrec_empty_inputs(self):
        """Test building RELREC with no domain data."""
        service = RelrecService()
        df, config = service.build_relrec({}, "TEST001")

        # Should return empty dataframe with proper structure
        assert isinstance(df, pd.DataFrame)
        assert df.empty
        assert list(df.columns) == [
            "STUDYID",
            "RDOMAIN",
            "USUBJID",
            "IDVAR",
            "IDVARVAL",
            "RELTYPE",
            "RELID",
        ]

        # Config should be properly structured
        assert config.domain == "RELREC"
        assert config.study_id == "TEST001"

    def test_build_relrec_ae_to_ds_linking(self):
        """Test linking AE records to DS records."""
        service = RelrecService()

        # Create sample domain dataframes
        ae_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001", "SUB002"],
                "AESEQ": [1, 1],
                "AETERM": ["Headache", "Nausea"],
            }
        )

        ds_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001", "SUB002"],
                "DSSEQ": [1, 1],
                "DSDECOD": ["COMPLETED", "WITHDRAWN"],
            }
        )

        domain_dataframes = {
            "AE": ae_df,
            "DS": ds_df,
        }

        df, config = service.build_relrec(domain_dataframes, "TEST001")

        # Should create 4 records: 2 AE records + 2 linked DS records
        assert len(df) == 4

        # Check AE records
        ae_records = df[df["RDOMAIN"] == "AE"]
        assert len(ae_records) == 2
        assert all(ae_records["IDVAR"] == "AESEQ")
        assert all(ae_records["STUDYID"] == "TEST001")

        # Check DS records
        ds_records = df[df["RDOMAIN"] == "DS"]
        assert len(ds_records) == 2
        assert all(ds_records["IDVAR"] == "DSSEQ")

        # Check that records are linked by RELID
        for usubjid in ["SUB001", "SUB002"]:
            usubjid_records = df[df["USUBJID"] == usubjid]
            # Should have 2 records (AE + DS)
            assert len(usubjid_records) == 2
            # Should have same RELID
            assert usubjid_records["RELID"].nunique() == 1

    def test_build_relrec_ex_to_ds_linking(self):
        """Test linking EX records to DS records."""
        service = RelrecService()

        # Create sample domain dataframes
        ex_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001", "SUB002"],
                "EXSEQ": [1, 1],
                "EXTRT": ["Drug A", "Drug B"],
            }
        )

        ds_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001", "SUB002"],
                "DSSEQ": [1, 1],
                "DSDECOD": ["COMPLETED", "COMPLETED"],
            }
        )

        domain_dataframes = {
            "EX": ex_df,
            "DS": ds_df,
        }

        df, config = service.build_relrec(domain_dataframes, "TEST001")

        # Should create 4 records: 2 EX records + 2 linked DS records
        assert len(df) == 4

        # Check EX records
        ex_records = df[df["RDOMAIN"] == "EX"]
        assert len(ex_records) == 2
        assert all(ex_records["IDVAR"] == "EXSEQ")

        # Check DS records
        ds_records = df[df["RDOMAIN"] == "DS"]
        assert len(ds_records) == 2
        assert all(ds_records["IDVAR"] == "DSSEQ")

    def test_build_relrec_both_ae_and_ex(self):
        """Test linking both AE and EX records to DS."""
        service = RelrecService()

        # Create sample domain dataframes
        ae_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001"],
                "AESEQ": [1],
                "AETERM": ["Headache"],
            }
        )

        ex_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001"],
                "EXSEQ": [1],
                "EXTRT": ["Drug A"],
            }
        )

        ds_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001"],
                "DSSEQ": [1],
                "DSDECOD": ["COMPLETED"],
            }
        )

        domain_dataframes = {
            "AE": ae_df,
            "EX": ex_df,
            "DS": ds_df,
        }

        df, config = service.build_relrec(domain_dataframes, "TEST001")

        # Should create 4 records: 1 AE + 1 EX + 2 DS (one for each link)
        assert len(df) == 4
        assert len(df[df["RDOMAIN"] == "AE"]) == 1
        assert len(df[df["RDOMAIN"] == "EX"]) == 1
        assert len(df[df["RDOMAIN"] == "DS"]) == 2

    def test_build_relrec_ds_only_fallback(self):
        """Test fallback to DS-only relationships when no AE/EX exist."""
        service = RelrecService()

        # Create only DS dataframe
        ds_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001", "SUB002"],
                "DSSEQ": [1, 1],
                "DSDECOD": ["COMPLETED", "WITHDRAWN"],
            }
        )

        domain_dataframes = {
            "DS": ds_df,
        }

        df, config = service.build_relrec(domain_dataframes, "TEST001")

        # Should create 2 DS-only records
        assert len(df) == 2
        assert all(df["RDOMAIN"] == "DS")
        assert all(df["IDVAR"] == "DSSEQ")

        # Check RELID format
        assert all(df["RELID"].str.startswith("DS_ONLY_"))

    def test_build_relrec_missing_usubjid(self):
        """Test that records without USUBJID are skipped."""
        service = RelrecService()

        # Create AE dataframe with missing USUBJID
        ae_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001", "", None, "SUB002"],
                "AESEQ": [1, 2, 3, 4],
                "AETERM": ["Event1", "Event2", "Event3", "Event4"],
            }
        )

        ds_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001", "SUB002"],
                "DSSEQ": [1, 1],
                "DSDECOD": ["COMPLETED", "COMPLETED"],
            }
        )

        domain_dataframes = {
            "AE": ae_df,
            "DS": ds_df,
        }

        df, config = service.build_relrec(domain_dataframes, "TEST001")

        # Should only create records for SUB001 and SUB002
        ae_records = df[df["RDOMAIN"] == "AE"]
        assert len(ae_records) == 2
        assert set(ae_records["USUBJID"]) == {"SUB001", "SUB002"}

    def test_build_relrec_missing_seq_columns(self):
        """Test behavior when sequence columns are missing."""
        service = RelrecService()

        # Create AE dataframe without AESEQ
        ae_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001"],
                "AETERM": ["Headache"],
            }
        )

        ds_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001"],
                "DSSEQ": [1],
                "DSDECOD": ["COMPLETED"],
            }
        )

        domain_dataframes = {
            "AE": ae_df,
            "DS": ds_df,
        }

        df, config = service.build_relrec(domain_dataframes, "TEST001")

        # Should still create records, using fallback index for missing seq
        ae_records = df[df["RDOMAIN"] == "AE"]
        assert len(ae_records) == 1
        # IDVARVAL should use fallback index (1)
        assert ae_records.iloc[0]["IDVARVAL"] == "1"

    def test_build_relrec_min_ds_seq(self):
        """Test that minimum DS sequence is used for linking."""
        service = RelrecService()

        # Create DS dataframe with multiple sequences per subject
        ds_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001", "SUB001", "SUB001"],
                "DSSEQ": [1, 2, 3],
                "DSDECOD": ["RANDOMIZED", "ONGOING", "COMPLETED"],
            }
        )

        ae_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001"],
                "AESEQ": [1],
                "AETERM": ["Headache"],
            }
        )

        domain_dataframes = {
            "AE": ae_df,
            "DS": ds_df,
        }

        df, config = service.build_relrec(domain_dataframes, "TEST001")

        # Check DS record uses minimum sequence (1)
        ds_record = df[df["RDOMAIN"] == "DS"].iloc[0]
        assert ds_record["IDVARVAL"] == "1"

    def test_convenience_function(self):
        """Test the convenience function build_relrec."""
        ds_df = pd.DataFrame(
            {
                "USUBJID": ["SUB001"],
                "DSSEQ": [1],
                "DSDECOD": ["COMPLETED"],
            }
        )

        domain_dataframes = {
            "DS": ds_df,
        }

        df, config = build_relrec(domain_dataframes, "TEST001")

        # Should work the same as service method
        assert len(df) == 1
        assert df.iloc[0]["RDOMAIN"] == "DS"
        assert config.domain == "RELREC"


class TestRelrecServiceHelpers:
    """Tests for RELREC service helper methods."""

    def test_get_domain_df_found(self):
        """Test getting domain dataframe when it exists."""
        service = RelrecService()

        ae_df = pd.DataFrame({"USUBJID": ["SUB001"]})
        domain_dataframes = {"AE": ae_df, "DS": pd.DataFrame()}

        result = service._get_domain_df(domain_dataframes, "AE")
        assert result is not None
        assert len(result) == 1

    def test_get_domain_df_not_found(self):
        """Test getting domain dataframe when it doesn't exist."""
        service = RelrecService()

        domain_dataframes = {"AE": pd.DataFrame({"USUBJID": ["SUB001"]})}

        result = service._get_domain_df(domain_dataframes, "DS")
        assert result is None

    def test_get_domain_df_empty(self):
        """Test getting domain dataframe when it's empty."""
        service = RelrecService()

        domain_dataframes = {"AE": pd.DataFrame()}

        result = service._get_domain_df(domain_dataframes, "AE")
        assert result is None

    def test_build_seq_map(self):
        """Test building sequence map."""
        service = RelrecService()

        df = pd.DataFrame(
            {
                "USUBJID": ["SUB001", "SUB001", "SUB002"],
                "DSSEQ": [1, 2, 1],
            }
        )

        seq_map = service._build_seq_map(df, "DSSEQ")

        assert len(seq_map) == 2
        assert seq_map["SUB001"] == 1  # Minimum
        assert seq_map["SUB002"] == 1

    def test_build_seq_map_missing_column(self):
        """Test building sequence map with missing column."""
        service = RelrecService()

        df = pd.DataFrame(
            {
                "USUBJID": ["SUB001"],
            }
        )

        seq_map = service._build_seq_map(df, "DSSEQ")

        assert len(seq_map) == 0

    def test_stringify_integer(self):
        """Test stringifying integer values."""
        service = RelrecService()

        assert service._stringify(42, 999) == "42"
        assert service._stringify(42.0, 999) == "42"

    def test_stringify_float(self):
        """Test stringifying float values."""
        service = RelrecService()

        assert service._stringify(42.5, 999) == "42.5"

    def test_stringify_none(self):
        """Test stringifying None values."""
        service = RelrecService()

        assert service._stringify(None, 999) == "999"

    def test_stringify_pandas_series(self):
        """Test stringifying pandas Series."""
        service = RelrecService()

        series = pd.Series([42])
        assert service._stringify(series, 999) == "42"

        empty_series = pd.Series([])
        assert service._stringify(empty_series, 999) == "999"

    def test_stringify_invalid(self):
        """Test stringifying invalid values."""
        service = RelrecService()

        assert service._stringify("not a number", 999) == "not a number"
