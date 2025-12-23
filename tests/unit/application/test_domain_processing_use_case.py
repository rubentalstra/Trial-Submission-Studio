"""Tests for domain processing use case.

These tests verify the domain processing orchestration logic using mocked
dependencies, ensuring testability without filesystem access.

CLEAN2-D1: These tests are now enabled since the circular import issue
has been resolved by implementing the real DomainProcessingUseCase.
"""

# pyright: reportPrivateUsage=false

import inspect
from pathlib import Path
from unittest.mock import Mock, patch
import pandas as pd

from cdisc_transpiler.application.models import (
    ProcessDomainRequest,
    ProcessDomainResponse,
)
from cdisc_transpiler.application.domain_processing_use_case import (
    DomainProcessingUseCase,
)
from cdisc_transpiler.infrastructure.logging import NullLogger


class TestDomainProcessingUseCase:
    """Tests for DomainProcessingUseCase."""

    def _create_mock_domain(self, domain_code: str = "DM"):
        """Create a mock SDTMDomain."""
        mock_domain = Mock()
        mock_domain.code = domain_code
        mock_domain.description = f"{domain_code} Domain"
        mock_domain.resolved_dataset_name.return_value = domain_code.lower()
        mock_domain.variables = []
        mock_domain.variable_names.return_value = []
        return mock_domain

    def _create_use_case(self):
        """Create use case with mocked dependencies."""
        logger = NullLogger()
        mock_repo = Mock()
        mock_generator = Mock()
        mock_mapping = Mock()
        mock_output_preparer = Mock()
        mock_domain_frame_builder = Mock()
        mock_suppqual_service = Mock()
        mock_terminology_service = Mock()
        mock_domain_definition_repository = Mock()
        mock_domain_definition_repository.get_domain.return_value = (
            self._create_mock_domain()
        )
        mock_xpt_writer = Mock()
        return DomainProcessingUseCase(
            logger=logger,
            study_data_repository=mock_repo,
            file_generator=mock_generator,
            mapping_service=mock_mapping,
            output_preparer=mock_output_preparer,
            domain_frame_builder=mock_domain_frame_builder,
            suppqual_service=mock_suppqual_service,
            terminology_service=mock_terminology_service,
            domain_definition_repository=mock_domain_definition_repository,
            xpt_writer=mock_xpt_writer,
        )

    def test_execute_returns_failed_response_on_no_data(self):
        """Test that processing returns failed response when no data can be processed."""
        use_case = self._create_use_case()

        request = ProcessDomainRequest(
            files_for_domain=[],  # No files provided
            domain_code="DM",
            study_id="TEST001",
        )

        response = use_case.execute(request)

        assert response.success is False
        assert response.domain_code == "DM"
        assert "No data could be processed" in response.error

    def test_execute_error_handling(self):
        """Test domain processing error handling."""
        use_case = self._create_use_case()

        # Make domain lookup raise an exception
        with patch.object(
            use_case._domain_definition_repository,  # type: ignore[union-attr]
            "get_domain",
            side_effect=ValueError("Domain not found"),
        ):
            request = ProcessDomainRequest(
                files_for_domain=[(Path("/data/DM.csv"), "DM")],
                domain_code="INVALID",
                study_id="TEST001",
            )

            response = use_case.execute(request)

        assert response.success is False
        assert "Domain not found" in response.error

    def test_use_case_can_be_imported_at_runtime(self):
        """Test that use case can be imported dynamically."""
        assert DomainProcessingUseCase is not None
        assert hasattr(DomainProcessingUseCase, "execute")

    def test_use_case_no_longer_imports_legacy(self):
        """Test that the use case no longer imports from legacy module.

        This validates CLEAN2-D1 acceptance criteria: no legacy imports.
        """
        from cdisc_transpiler.application import domain_processing_use_case as module

        source = inspect.getsource(module)

        # Check that there are no imports from legacy (excluding comments/docstrings)
        assert "from ..legacy" not in source
        assert "from cdisc_transpiler.legacy" not in source
        # Check that there's no actual usage (import statement)
        assert "import DomainProcessingCoordinator" not in source
        assert "from ..services import DomainProcessingCoordinator" not in source

    def test_use_case_accepts_injected_dependencies(self):
        """Test that use case accepts injected dependencies."""
        logger = NullLogger()
        mock_repo = Mock()
        mock_generator = Mock()
        mock_mapping = Mock()
        mock_output_preparer = Mock()
        mock_domain_frame_builder = Mock()
        mock_suppqual_service = Mock()
        mock_terminology_service = Mock()
        mock_domain_definition_repository = Mock()
        mock_xpt_writer = Mock()

        use_case = DomainProcessingUseCase(
            logger=logger,
            study_data_repository=mock_repo,
            file_generator=mock_generator,
            mapping_service=mock_mapping,
            output_preparer=mock_output_preparer,
            domain_frame_builder=mock_domain_frame_builder,
            suppqual_service=mock_suppqual_service,
            terminology_service=mock_terminology_service,
            domain_definition_repository=mock_domain_definition_repository,
            xpt_writer=mock_xpt_writer,
        )

        assert use_case.logger is logger
        assert use_case._study_data_repository is mock_repo
        assert use_case._file_generator is mock_generator
        assert use_case._terminology_service is mock_terminology_service

    def test_container_creates_use_case_with_dependencies(self):
        """Test that DependencyContainer properly wires dependencies."""
        from cdisc_transpiler.infrastructure.container import DependencyContainer

        container = DependencyContainer(use_null_logger=True)
        use_case = container.create_domain_processing_use_case()

        assert use_case is not None
        assert use_case.logger is not None
        assert use_case._study_data_repository is not None
        assert use_case._file_generator is not None


class TestProcessDomainRequest:
    """Tests for ProcessDomainRequest DTO."""

    def test_create_with_defaults(self):
        """Test creating request with default values."""
        request = ProcessDomainRequest(
            files_for_domain=[(Path("/data/DM.csv"), "DM")],
            domain_code="DM",
            study_id="TEST001",
        )

        assert request.files_for_domain == [(Path("/data/DM.csv"), "DM")]
        assert request.domain_code == "DM"
        assert request.study_id == "TEST001"
        assert request.output_formats == {"xpt", "xml"}
        assert request.min_confidence == 0.5
        assert request.streaming is False
        assert request.chunk_size == 1000
        assert request.generate_sas is True
        assert request.verbose == 0

    def test_create_with_custom_values(self):
        """Test creating request with custom values."""
        request = ProcessDomainRequest(
            files_for_domain=[(Path("/data/AE.csv"), "AE")],
            domain_code="AE",
            study_id="CUSTOM001",
            output_formats={"xpt"},
            output_dirs={"xpt": Path("/output/xpt")},
            min_confidence=0.7,
            streaming=True,
            chunk_size=500,
            generate_sas=False,
            verbose=2,
        )

        assert request.output_formats == {"xpt"}
        assert request.output_dirs == {"xpt": Path("/output/xpt")}
        assert request.min_confidence == 0.7
        assert request.streaming is True
        assert request.chunk_size == 500
        assert request.generate_sas is False
        assert request.verbose == 2

    def test_create_with_multiple_files(self):
        """Test creating request with multiple input files (variants)."""
        files = [
            (Path("/data/LB.csv"), "LB"),
            (Path("/data/LBCC.csv"), "LBCC"),
            (Path("/data/LBHM.csv"), "LBHM"),
        ]

        request = ProcessDomainRequest(
            files_for_domain=files,
            domain_code="LB",
            study_id="TEST001",
        )

        assert len(request.files_for_domain) == 3
        assert request.domain_code == "LB"

    def test_create_with_metadata(self):
        """Test creating request with study metadata."""
        request = ProcessDomainRequest(
            files_for_domain=[(Path("/data/DM.csv"), "DM")],
            domain_code="DM",
            study_id="TEST001",
            metadata={"items": {}, "codelists": {}},
            reference_starts={"SUBJ001": "2023-01-01"},
            common_column_counts={"studyid": 5, "usubjid": 5},
            total_input_files=10,
        )

        assert request.metadata is not None
        assert request.reference_starts == {"SUBJ001": "2023-01-01"}
        assert request.common_column_counts is not None
        assert request.total_input_files == 10


class TestProcessDomainResponse:
    """Tests for ProcessDomainResponse DTO."""

    def test_create_successful_response(self):
        """Test creating a successful domain processing response."""
        df = pd.DataFrame(
            {
                "STUDYID": ["TEST001", "TEST001"],
                "DOMAIN": ["DM", "DM"],
                "USUBJID": ["TEST001-001", "TEST001-002"],
            }
        )

        response = ProcessDomainResponse(
            success=True,
            domain_code="DM",
            records=2,
            domain_dataframe=df,
            xpt_path=Path("/output/xpt/dm.xpt"),
            xml_path=Path("/output/xml/dm.xml"),
        )

        assert response.success is True
        assert response.domain_code == "DM"
        assert response.records == 2
        assert response.domain_dataframe is not None
        assert len(response.domain_dataframe) == 2
        assert response.xpt_path == Path("/output/xpt/dm.xpt")
        assert response.xml_path == Path("/output/xml/dm.xml")
        assert response.error is None

    def test_create_failed_response(self):
        """Test creating a failed domain processing response."""
        response = ProcessDomainResponse(
            success=False,
            domain_code="AE",
            error="File not found: AE.csv",
        )

        assert response.success is False
        assert response.domain_code == "AE"
        assert response.error == "File not found: AE.csv"
        assert response.domain_dataframe is None
        assert response.records == 0

    def test_create_with_supplementals(self):
        """Test creating response with supplemental domains."""
        main_df = pd.DataFrame({"STUDYID": ["TEST001"], "DOMAIN": ["AE"]})
        supp_df = pd.DataFrame({"STUDYID": ["TEST001"], "RDOMAIN": ["AE"]})

        supp_response = ProcessDomainResponse(
            success=True,
            domain_code="SUPPAE",
            records=1,
            domain_dataframe=supp_df,
            xpt_path=Path("/output/xpt/suppae.xpt"),
        )

        response = ProcessDomainResponse(
            success=True,
            domain_code="AE",
            records=1,
            domain_dataframe=main_df,
            supplementals=[supp_response],
        )

        assert len(response.supplementals) == 1
        assert response.supplementals[0].domain_code == "SUPPAE"
        assert response.supplementals[0].records == 1

    def test_create_with_warnings(self):
        """Test creating response with warnings."""
        df = pd.DataFrame({"STUDYID": ["TEST001"]})

        response = ProcessDomainResponse(
            success=True,
            domain_code="VS",
            records=1,
            domain_dataframe=df,
            warnings=["Low confidence match for VSTESTCD", "Missing VSORRESU"],
        )

        assert response.success is True
        assert len(response.warnings) == 2
        assert "Low confidence match" in response.warnings[0]

    def test_to_dict_conversion(self):
        """Test conversion to dictionary for legacy compatibility."""
        df = pd.DataFrame({"STUDYID": ["TEST001"]})
        supp_df = pd.DataFrame({"STUDYID": ["TEST001"]})

        supp_response = ProcessDomainResponse(
            success=True,
            domain_code="SUPPAE",
            records=1,
            domain_dataframe=supp_df,
        )

        response = ProcessDomainResponse(
            success=True,
            domain_code="AE",
            records=1,
            domain_dataframe=df,
            xpt_path=Path("/output/xpt/ae.xpt"),
            supplementals=[supp_response],
        )

        result_dict = response.to_dict()

        assert result_dict["domain_code"] == "AE"
        assert result_dict["records"] == 1
        assert result_dict["xpt_path"] == Path("/output/xpt/ae.xpt")
        assert len(result_dict["supplementals"]) == 1
        assert result_dict["supplementals"][0]["domain_code"] == "SUPPAE"

    def test_to_dict_with_empty_collections(self):
        """Test to_dict with empty supplementals and splits."""
        df = pd.DataFrame({"STUDYID": ["TEST001"]})

        response = ProcessDomainResponse(
            success=True,
            domain_code="DM",
            records=1,
            domain_dataframe=df,
        )

        result_dict = response.to_dict()

        assert result_dict["supplementals"] == []
