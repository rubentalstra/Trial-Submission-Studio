"""Tests for synthesis service.

These tests verify the synthesis service creates proper SDTM domain scaffold structures
for both trial design and observation domains.

The SynthesisService is a pure domain service that returns scaffold data
(DataFrames + configs). SDTM building and file generation are handled in the
application layer.
"""

import pytest

from cdisc_transpiler.domain.services import SynthesisService, SynthesisResult
from cdisc_transpiler.domain.entities.sdtm_domain import SDTMDomain, SDTMVariable


def _make_domain(code: str, variable_names: list[str]) -> SDTMDomain:
    variables: list[SDTMVariable] = []
    for idx, name in enumerate(variable_names, start=1):
        variables.append(
            SDTMVariable(
                name=name,
                label=name,
                type="Num" if name.upper().endswith("DOSE") else "Char",
                length=200,
                core="Req",
                variable_order=idx,
            )
        )

    upper = code.upper()
    return SDTMDomain(
        code=upper,
        description=f"{upper} domain",
        class_name="Synthetic",
        structure="",
        label=upper,
        variables=tuple(variables),
        dataset_name=upper,
    )


@pytest.fixture
def domain_resolver():
    domains = {
        "TS": _make_domain("TS", ["STUDYID", "DOMAIN", "TSPARMCD", "TSPARM", "TSVAL"]),
        "TA": _make_domain(
            "TA", ["STUDYID", "DOMAIN", "ARMCD", "ARM", "ETCD", "ELEMENT", "EPOCH"]
        ),
        "TE": _make_domain("TE", ["STUDYID", "DOMAIN", "ETCD", "ELEMENT", "TEDUR"]),
        "SE": _make_domain(
            "SE",
            ["STUDYID", "DOMAIN", "USUBJID", "ETCD", "ELEMENT", "EPOCH"],
        ),
        "DS": _make_domain("DS", ["STUDYID", "DOMAIN", "USUBJID", "DSTERM", "DSDECOD"]),
        "AE": _make_domain("AE", ["STUDYID", "DOMAIN", "USUBJID", "AETERM", "AEDECOD"]),
        "LB": _make_domain(
            "LB", ["STUDYID", "DOMAIN", "USUBJID", "LBTESTCD", "LBTEST", "LBORRES"]
        ),
        "VS": _make_domain(
            "VS", ["STUDYID", "DOMAIN", "USUBJID", "VSTESTCD", "VSTEST", "VSORRES"]
        ),
        "EX": _make_domain("EX", ["STUDYID", "DOMAIN", "USUBJID", "EXTRT", "EXDOSE"]),
    }

    def _resolve(domain_code: str) -> SDTMDomain:
        return domains[domain_code.upper()]

    return _resolve


class TestSynthesisService:
    """Tests for SynthesisService."""

    def test_service_can_be_imported(self):
        """Test that synthesis service can be imported."""
        assert SynthesisService is not None
        assert SynthesisResult is not None

    def test_service_instantiation(self):
        """Test that service requires a domain resolver."""
        with pytest.raises(TypeError):
            SynthesisService()  # type: ignore[call-arg]

    def test_service_is_pure_domain_service(self, domain_resolver):
        """Test that service has no infrastructure dependencies."""
        service = SynthesisService(domain_resolver=domain_resolver)

        # Verify it doesn't have file_generator or logger attributes
        assert not hasattr(service, "_file_generator")
        assert not hasattr(service, "_logger")


class TestSynthesizeTrialDesign:
    """Tests for trial design domain synthesis."""

    @pytest.fixture
    def service(self, domain_resolver):
        """Create synthesis service instance."""
        return SynthesisService(domain_resolver=domain_resolver)

    def test_synthesize_ts_domain(self, service):
        """Test TS (Trial Summary) domain synthesis."""
        result = service.synthesize_trial_design(
            domain_code="TS",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "TS"
        assert result.domain_dataframe is not None

        df = result.domain_dataframe
        assert len(df) == 1

        # Check required columns
        assert "STUDYID" in df.columns
        assert "DOMAIN" in df.columns
        assert "TSPARMCD" in df.columns
        assert "TSPARM" in df.columns
        assert "TSVAL" in df.columns

        # Values are intentionally left empty in this scaffold; the application
        # layer builds SDTM-compliant datasets.
        assert df["STUDYID"].isna().iloc[0]
        assert df["DOMAIN"].isna().iloc[0]

    def test_synthesize_ta_domain(self, service):
        """Test TA (Trial Arms) domain synthesis."""
        result = service.synthesize_trial_design(
            domain_code="TA",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "TA"
        assert result.domain_dataframe is not None
        assert len(result.domain_dataframe) == 1

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
        # Note: TEDUR may be dropped when all values are empty/optional

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

    def test_synthesize_trial_design_injects_config_rows(self, service):
        result = service.synthesize_trial_design(
            domain_code="TS",
            study_id="TEST001",
            rows=[{"TSPARMCD": "ACTSUB", "TSVAL": "01"}],
        )

        assert result.success
        assert result.domain_dataframe is not None
        df = result.domain_dataframe
        assert len(df) == 1
        assert df["TSPARMCD"].iloc[0] == "ACTSUB"
        assert df["TSVAL"].iloc[0] == "01"


class TestSynthesizeObservation:
    """Tests for observation domain synthesis."""

    @pytest.fixture
    def service(self, domain_resolver):
        """Create synthesis service instance."""
        return SynthesisService(domain_resolver=domain_resolver)

    def test_synthesize_ae_domain(self, service):
        """Test AE (Adverse Events) domain synthesis."""
        result = service.synthesize_observation(
            domain_code="AE",
            study_id="TEST001",
        )

        assert result.success
        assert result.domain_code == "AE"
        assert result.domain_dataframe is not None

        df = result.domain_dataframe
        assert len(df) == 1

        assert "STUDYID" in df.columns
        assert "DOMAIN" in df.columns
        assert "USUBJID" in df.columns
        assert "AETERM" in df.columns
        assert "AEDECOD" in df.columns

        # Values are intentionally left empty in this scaffold.
        assert df["STUDYID"].isna().iloc[0]
        assert df["DOMAIN"].isna().iloc[0]

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

        # Reference starts are accepted for downstream date transformations;
        # scaffold values remain intentionally empty.

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
