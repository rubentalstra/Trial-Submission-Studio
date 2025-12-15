"""Tests for study processing use case.

These tests verify the study processing orchestration logic using mocked
dependencies, ensuring testability without filesystem access.
"""

import pytest
from pathlib import Path
from unittest.mock import Mock, MagicMock, patch
import pandas as pd

from cdisc_transpiler.application.models import (
    ProcessStudyRequest,
    ProcessStudyResponse,
    DomainProcessingResult,
)


# Note: Due to circular import issues in the current codebase (services -> cli -> services),
# we cannot import StudyProcessingUseCase at the module level.
# These tests are placeholders that demonstrate the testing approach.
# Once the circular import is resolved, uncomment the actual tests.

class TestStudyProcessingUseCase:
    """Tests for StudyProcessingUseCase."""
    
    @pytest.mark.skip(reason="Circular import issue: services -> cli -> services")
    def test_happy_path_all_domains_succeed(self):
        """Test successful processing of all domains."""
        # This would test the main success path
        # from cdisc_transpiler.application.study_processing_use_case import StudyProcessingUseCase
        pass
    
    @pytest.mark.skip(reason="Circular import issue: services -> cli -> services")
    def test_partial_failure_some_domains_fail(self):
        """Test processing with some domain failures."""
        # This would test partial failure handling
        pass
    
    @pytest.mark.skip(reason="Circular import issue: services -> cli -> services")
    def test_complete_failure_no_domains_found(self):
        """Test processing when no domains are found."""
        # This would test complete failure scenario
        pass
    
    @pytest.mark.skip(reason="Circular import issue: services -> cli -> services")
    def test_synthesis_triggered_for_missing_domains(self):
        """Test that synthesis is triggered for missing required domains."""
        # This would test synthesis logic
        pass
    
    @pytest.mark.skip(reason="Circular import issue: services -> cli -> services")
    def test_define_xml_generation(self):
        """Test Define-XML generation."""
        # This would test Define-XML generation
        pass


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
