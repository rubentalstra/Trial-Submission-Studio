"""Tests for synthesis service.

These tests verify the synthesis service creates proper SDTM domain structures
for both trial design and observation domains.

The SynthesisService is a pure domain service that returns only domain data
(DataFrames + configs). File generation is handled in the application layer.
"""

import pytest

from cdisc_transpiler.domain.services import SynthesisService, SynthesisResult


class TestSynthesisService:
    """Tests for SynthesisService."""

    def test_service_can_be_imported(self):
        """Test that synthesis service can be imported."""
        assert SynthesisService is not None
        assert SynthesisResult is not None

    def test_service_instantiation(self):
        """Test that service can be instantiated without dependencies."""
        service = SynthesisService()
        assert service is not None

    def test_service_is_pure_domain_service(self):
        """Test that service has no infrastructure dependencies."""
        # SynthesisService should be instantiable with no arguments
        # This verifies it's a pure domain service
        service = SynthesisService()
        assert service is not None

        # Verify it doesn't have file_generator or logger attributes
        assert not hasattr(service, "_file_generator")
        assert not hasattr(service, "_logger")


class TestSynthesizeTrialDesign:
    """Tests for trial design domain synthesis."""

    @pytest.fixture
    def service(self):
        """Create synthesis service instance."""
        return SynthesisService()

    def test_synthesize_ts_domain(self, service):
        """Test TS (Trial Summary) domain synthesis."""
        result = service.synthesize_trial_design(
            domain_code="TS",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "TS"
        assert result.domain_dataframe is not None
        assert len(result.domain_dataframe) >= 1

        # Check required columns
        df = result.domain_dataframe
        assert "STUDYID" in df.columns
        assert "DOMAIN" in df.columns
        assert "TSPARMCD" in df.columns
        assert "TSPARM" in df.columns
        assert "TSVAL" in df.columns

        # Check values
        assert df["STUDYID"].iloc[0] == "TEST001"
        assert df["DOMAIN"].iloc[0] == "TS"
        # TSPARMCD values are populated by the domain builder's default SDTM population

    def test_synthesize_ta_domain(self, service):
        """Test TA (Trial Arms) domain synthesis."""
        result = service.synthesize_trial_design(
            domain_code="TA",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "TA"
        assert result.domain_dataframe is not None
        assert len(result.domain_dataframe) >= 2  # SCREENING + TREATMENT

        df = result.domain_dataframe
        assert "ARMCD" in df.columns
        assert "ARM" in df.columns
        assert "ETCD" in df.columns
        assert "ELEMENT" in df.columns

    def test_synthesize_te_domain(self, service):
        """Test TE (Trial Elements) domain synthesis."""
        result = service.synthesize_trial_design(
            domain_code="TE",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "TE"
        assert result.domain_dataframe is not None

        df = result.domain_dataframe
        assert "ETCD" in df.columns
        assert "ELEMENT" in df.columns
        # Note: TEDUR may be dropped by xpt_module builder when all values are empty/optional

    def test_synthesize_se_domain(self, service):
        """Test SE (Subject Elements) domain synthesis."""
        result = service.synthesize_trial_design(
            domain_code="SE",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "SE"
        assert result.domain_dataframe is not None

        df = result.domain_dataframe
        assert "USUBJID" in df.columns
        assert "ETCD" in df.columns
        assert "ELEMENT" in df.columns

    def test_synthesize_ds_domain(self, service):
        """Test DS (Disposition) domain synthesis."""
        result = service.synthesize_trial_design(
            domain_code="DS",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "DS"
        assert result.domain_dataframe is not None

        df = result.domain_dataframe
        assert "DSTERM" in df.columns
        assert "DSDECOD" in df.columns

    def test_synthesize_with_reference_starts(self, service):
        """Test synthesis uses reference start dates when provided."""
        result = service.synthesize_trial_design(
            domain_code="TS",
            study_id="TEST001",
            reference_starts={"SUBJ001": "2024-01-15", "SUBJ002": "2024-02-01"},
        )

        assert result.success
        assert result.domain_dataframe is not None

    def test_config_is_returned(self, service):
        """Test that mapping config is returned with synthesis."""
        result = service.synthesize_trial_design(
            domain_code="TS",
            study_id="TEST001",
        )

        assert result.success
        assert result.config is not None
        assert result.config.study_id == "TEST001"
        assert result.config.domain == "TS"


class TestSynthesizeObservation:
    """Tests for observation domain synthesis."""

    @pytest.fixture
    def service(self):
        """Create synthesis service instance."""
        return SynthesisService()

    def test_synthesize_ae_domain(self, service):
        """Test AE (Adverse Events) domain synthesis."""
        result = service.synthesize_observation(
            domain_code="AE",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "AE"
        assert result.domain_dataframe is not None
        assert len(result.domain_dataframe) >= 1

        df = result.domain_dataframe
        assert "STUDYID" in df.columns
        assert "DOMAIN" in df.columns
        assert "USUBJID" in df.columns
        assert "AETERM" in df.columns
        assert "AEDECOD" in df.columns

        # Check default values
        assert df["STUDYID"].iloc[0] == "TEST001"
        assert df["DOMAIN"].iloc[0] == "AE"

    def test_synthesize_lb_domain(self, service):
        """Test LB (Laboratory) domain synthesis."""
        result = service.synthesize_observation(
            domain_code="LB",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "LB"
        assert result.domain_dataframe is not None

        df = result.domain_dataframe
        assert "LBTESTCD" in df.columns
        assert "LBTEST" in df.columns

    def test_synthesize_vs_domain(self, service):
        """Test VS (Vital Signs) domain synthesis."""
        result = service.synthesize_observation(
            domain_code="VS",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "VS"
        assert result.domain_dataframe is not None

        df = result.domain_dataframe
        assert "VSTESTCD" in df.columns
        assert "VSTEST" in df.columns

    def test_synthesize_ex_domain(self, service):
        """Test EX (Exposure) domain synthesis."""
        result = service.synthesize_observation(
            domain_code="EX",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "EX"
        assert result.domain_dataframe is not None

        df = result.domain_dataframe
        assert "EXTRT" in df.columns
        assert "EXDOSE" in df.columns

    def test_synthesize_with_reference_starts(self, service):
        """Test synthesis uses reference start dates when provided."""
        result = service.synthesize_observation(
            domain_code="AE",
            study_id="TEST001",
            reference_starts={"SUBJ001": "2024-01-15"},
        )

        assert result.success
        assert result.domain_dataframe is not None

        # Should use the first subject from reference_starts
        df = result.domain_dataframe
        assert df["USUBJID"].iloc[0] == "SUBJ001"

    def test_config_is_returned(self, service):
        """Test that mapping config is returned with synthesis."""
        result = service.synthesize_observation(
            domain_code="AE",
            study_id="TEST001",
        )

        assert result.success
        assert result.config is not None
        assert result.config.study_id == "TEST001"
        assert result.config.domain == "AE"


class TestSynthesisResult:
    """Tests for SynthesisResult data class."""

    def test_result_creation(self):
        """Test SynthesisResult can be created."""
        result = SynthesisResult(domain_code="TS")

        assert result.domain_code == "TS"
        assert result.records == 0
        assert result.success is True
        assert result.error is None

    def test_result_to_dict(self):
        """Test SynthesisResult to_dict conversion."""
        result = SynthesisResult(
            domain_code="TS",
            records=1,
        )

        result_dict = result.to_dict()

        assert result_dict["domain_code"] == "TS"
        assert result_dict["records"] == 1

    def test_result_with_error(self):
        """Test SynthesisResult with error."""
        result = SynthesisResult(
            domain_code="TS",
            success=False,
            error="Domain not found",
        )

        assert result.success is False
        assert result.error == "Domain not found"

    def test_result_is_pure_domain_object(self):
        """Test SynthesisResult has no file path attributes.

        File paths are application/infrastructure concerns, not domain concerns.
        """
        result = SynthesisResult(domain_code="TS")

        # SynthesisResult should not have file path attributes
        assert not hasattr(result, "xpt_path")
        assert not hasattr(result, "xml_path")
        assert not hasattr(result, "sas_path")
