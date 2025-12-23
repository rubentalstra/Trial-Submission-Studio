"""Tests for study processing use case.

These tests verify the study processing orchestration logic using mocked
dependencies, ensuring testability without filesystem access.

CLEAN2-D2: Tests are now enabled since StudyProcessingUseCase accepts
injected dependencies and no longer imports from legacy modules.
"""

# pyright: reportPrivateUsage=false

from pathlib import Path
from unittest.mock import Mock

import pandas as pd

from cdisc_transpiler.application.models import (
    DomainProcessingResult,
    ProcessStudyRequest,
    ProcessStudyResponse,
)


class TestStudyProcessingUseCase:
    """Tests for StudyProcessingUseCase."""

    def test_use_case_can_be_imported(self):
        """Test that use case can be imported without issues."""
        from cdisc_transpiler.application.study_processing_use_case import (
            StudyProcessingUseCase,
        )

        assert StudyProcessingUseCase is not None
        assert hasattr(StudyProcessingUseCase, "execute")

    def test_use_case_accepts_injected_dependencies(self):
        """Test that use case accepts all dependencies via constructor."""
        from cdisc_transpiler.application.study_processing_use_case import (
            StudyProcessingUseCase,
        )
        from cdisc_transpiler.infrastructure.logging.null_logger import NullLogger

        # Create mock dependencies
        mock_logger = NullLogger()
        mock_repo = Mock()
        mock_domain_use_case = Mock()
        mock_discovery = Mock()
        mock_domain_frame_builder = Mock()
        mock_relrec_service = Mock()
        mock_relsub_service = Mock()
        mock_relspec_service = Mock()
        mock_file_gen = Mock()
        mock_output_preparer = Mock()
        mock_domain_definition_repository = Mock()

        # This should work without errors
        use_case = StudyProcessingUseCase(
            logger=mock_logger,
            study_data_repository=mock_repo,
            domain_processing_use_case=mock_domain_use_case,
            domain_discovery_service=mock_discovery,
            domain_frame_builder=mock_domain_frame_builder,
            relrec_service=mock_relrec_service,
            relsub_service=mock_relsub_service,
            relspec_service=mock_relspec_service,
            file_generator=mock_file_gen,
            output_preparer=mock_output_preparer,
            domain_definition_repository=mock_domain_definition_repository,
        )

        assert use_case is not None
        assert use_case.logger == mock_logger
        assert use_case._study_data_repository == mock_repo
        assert use_case._domain_processing_use_case == mock_domain_use_case
        assert use_case._domain_discovery_service == mock_discovery
        assert use_case._domain_frame_builder == mock_domain_frame_builder
        assert use_case._relrec_service == mock_relrec_service
        assert use_case._relsub_service == mock_relsub_service
        assert use_case._relspec_service == mock_relspec_service
        assert use_case._file_generator == mock_file_gen
        assert use_case._output_preparer == mock_output_preparer
        assert (
            use_case._domain_definition_repository == mock_domain_definition_repository
        )

    def test_no_legacy_imports(self):
        """Test that use case does not import legacy modules at module level."""
        use_case_path = (
            Path(__file__).parent.parent.parent.parent
            / "cdisc_transpiler"
            / "application"
            / "study_processing_use_case.py"
        )
        with open(use_case_path, encoding="utf-8") as f:
            source = f.read()

        # Check for module-level imports (outside of functions/methods)
        # The file should not have any "from ..legacy" imports at module level
        lines = source.split("\n")

        # Find the first class definition to identify module-level code
        class_start = None
        for i, line in enumerate(lines):
            if line.startswith("class "):
                class_start = i
                break

        # Check only module-level imports (before class definition)
        if class_start is not None:
            module_level_code = "\n".join(lines[:class_start])
            assert "from ..legacy" not in module_level_code, (
                "Found module-level legacy imports in StudyProcessingUseCase"
            )

    def test_container_wires_all_dependencies(self):
        """Test that DependencyContainer wires all dependencies."""
        from cdisc_transpiler.infrastructure.container import DependencyContainer

        container = DependencyContainer(use_null_logger=True)
        use_case = container.create_study_processing_use_case()

        # Verify all dependencies are wired
        assert use_case._study_data_repository is not None
        assert use_case._domain_processing_use_case is not None
        assert use_case._domain_discovery_service is not None
        assert use_case._domain_frame_builder is not None
        assert use_case._relrec_service is not None
        assert use_case._relsub_service is not None
        assert use_case._relspec_service is not None
        assert use_case._file_generator is not None
        assert use_case._output_preparer is not None
        assert use_case._domain_definition_repository is not None


class TestProcessStudyRequest:
    """Tests for ProcessStudyRequest DTO."""

    def test_create_with_defaults(self):
        """Test creating request with default values."""
        request = ProcessStudyRequest(
            study_folder=Path("/study"),
            study_id="TEST001",
            output_dir=Path("/output"),
        )

        assert request.study_folder == Path("/study")
        assert request.study_id == "TEST001"
        assert request.output_dir == Path("/output")
        assert request.output_formats == {"xpt", "xml"}
        assert request.generate_define_xml is True
        assert request.generate_sas is True
        assert request.min_confidence == 0.5

    def test_create_with_custom_values(self):
        """Test creating request with custom values."""
        request = ProcessStudyRequest(
            study_folder=Path("/study"),
            study_id="CUSTOM001",
            output_dir=Path("/out"),
            output_formats={"xpt"},
            generate_define_xml=False,
            generate_sas=False,
            min_confidence=0.7,
            verbose=2,
        )

        assert request.output_formats == {"xpt"}
        assert request.generate_define_xml is False
        assert request.generate_sas is False
        assert request.min_confidence == 0.7
        assert request.verbose == 2


class TestDomainProcessingResult:
    """Tests for DomainProcessingResult DTO."""

    def test_create_successful_result(self):
        """Test creating a successful domain processing result."""
        df = pd.DataFrame({"STUDYID": ["TEST001"], "USUBJID": ["TEST001-001"]})

        result = DomainProcessingResult(
            domain_code="DM",
            success=True,
            records=1,
            domain_dataframe=df,
        )

        assert result.domain_code == "DM"
        assert result.success is True
        assert result.records == 1
        assert result.domain_dataframe is not None
        assert len(result.domain_dataframe) == 1

    def test_create_failed_result(self):
        """Test creating a failed domain processing result."""
        result = DomainProcessingResult(
            domain_code="AE",
            success=False,
            error="File not found",
        )

        assert result.domain_code == "AE"
        assert result.success is False
        assert result.error == "File not found"
        assert result.domain_dataframe is None

    def test_create_with_supplementals(self):
        """Test creating result with supplemental domains."""
        main_df = pd.DataFrame({"STUDYID": ["TEST001"]})
        supp_df = pd.DataFrame({"STUDYID": ["TEST001"]})

        supp_result = DomainProcessingResult(
            domain_code="SUPPAE",
            success=True,
            records=1,
            domain_dataframe=supp_df,
        )

        result = DomainProcessingResult(
            domain_code="AE",
            success=True,
            records=1,
            domain_dataframe=main_df,
            supplementals=[supp_result],
        )

        assert len(result.supplementals) == 1
        assert result.supplementals[0].domain_code == "SUPPAE"

    def test_synthesized_domain(self):
        """Test creating a synthesized domain result."""
        result = DomainProcessingResult(
            domain_code="VS",
            success=True,
            records=0,
            synthesized=True,
            synthesis_reason="No source files found",
        )

        assert result.synthesized is True
        assert result.synthesis_reason == "No source files found"


class TestProcessStudyResponse:
    """Tests for ProcessStudyResponse DTO."""

    def test_create_successful_response(self):
        """Test creating a successful study processing response."""
        result1 = DomainProcessingResult(domain_code="DM", success=True, records=100)
        result2 = DomainProcessingResult(domain_code="AE", success=True, records=50)

        response = ProcessStudyResponse(
            success=True,
            study_id="TEST001",
            processed_domains={"DM", "AE"},
            domain_results=[result1, result2],
            total_records=150,
        )

        assert response.success is True
        assert response.study_id == "TEST001"
        assert len(response.processed_domains) == 2
        assert response.total_records == 150
        assert response.has_errors is False

    def test_response_with_errors(self):
        """Test creating response with errors."""
        result1 = DomainProcessingResult(domain_code="DM", success=True, records=100)

        response = ProcessStudyResponse(
            success=False,
            study_id="TEST001",
            processed_domains={"DM"},
            domain_results=[result1],
            errors=[("AE", "File not found"), ("LB", "Parse error")],
        )

        assert response.success is False
        assert response.has_errors is True
        assert len(response.errors) == 2
        assert len(response.failed_domains) == 2
        assert "AE" in response.failed_domains
        assert "LB" in response.failed_domains

    def test_response_with_define_xml_error(self):
        """Test creating response with Define-XML error."""
        response = ProcessStudyResponse(
            success=True,
            study_id="TEST001",
            define_xml_error="Invalid schema",
        )

        assert response.has_errors is True
        assert response.define_xml_error == "Invalid schema"

    def test_successful_domains_property(self):
        """Test successful_domains property."""
        result1 = DomainProcessingResult(domain_code="DM", success=True)
        result2 = DomainProcessingResult(domain_code="AE", success=False)
        result3 = DomainProcessingResult(domain_code="LB", success=True)

        response = ProcessStudyResponse(
            domain_results=[result1, result2, result3],
        )

        successful = response.successful_domains
        assert len(successful) == 2
        assert "DM" in successful
        assert "LB" in successful
        assert "AE" not in successful
