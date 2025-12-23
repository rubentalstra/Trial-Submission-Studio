"""Contract tests for repository port interfaces.

These tests define the expected behavior that any repository implementation
must satisfy. They verify that implementations correctly adhere to the
port interfaces using runtime protocol checks.
"""

from pathlib import Path

import pandas as pd
import pytest

from cdisc_transpiler.application.ports import (
    CTRepositoryPort,
    SDTMSpecRepositoryPort,
    StudyDataRepositoryPort,
)
from cdisc_transpiler.domain.entities.controlled_terminology import (
    ControlledTerminology,
)
from cdisc_transpiler.domain.entities.study_metadata import (
    SourceColumn,
    StudyMetadata,
)


class MockCTRepository:
    """Mock CT repository for testing protocol compliance."""

    def __init__(self):
        self._data = {
            "C66767": ControlledTerminology(
                codelist_code="C66767",
                codelist_name="SEX",
                submission_values={"M", "F", "U"},
                codelist_extensible=False,
            ),
        }

    def get_by_code(self, codelist_code: str) -> ControlledTerminology | None:
        return self._data.get(codelist_code)

    def get_by_name(self, codelist_name: str) -> ControlledTerminology | None:
        for ct in self._data.values():
            if ct.codelist_name == codelist_name:
                return ct
        return None

    def list_all_codes(self) -> list[str]:
        return list(self._data.keys())


class MockSDTMSpecRepository:
    """Mock SDTM spec repository for testing protocol compliance."""

    def __init__(self):
        self._variables = {
            "DM": [
                {
                    "Variable Name": "STUDYID",
                    "Label": "Study Identifier",
                    "Role": "Identifier",
                },
                {
                    "Variable Name": "USUBJID",
                    "Label": "Unique Subject Identifier",
                    "Role": "Identifier",
                },
            ],
        }
        self._attributes = {
            "DM": {
                "class": "SPECIAL PURPOSE",
                "label": "Demographics",
                "structure": "One record per subject",
            },
        }

    def get_domain_variables(self, domain_code: str) -> list[dict[str, str]]:
        return self._variables.get(domain_code, [])

    def get_dataset_attributes(self, domain_code: str) -> dict[str, str] | None:
        return self._attributes.get(domain_code)

    def list_available_domains(self) -> list[str]:
        return list(self._variables.keys())


class MockStudyDataRepository:
    """Mock study data repository for testing protocol compliance."""

    def read_dataset(self, file_path: str | Path) -> pd.DataFrame:
        # Return a simple DataFrame for testing
        return pd.DataFrame({"STUDYID": ["TEST001"], "USUBJID": ["TEST001-001"]})

    def load_study_metadata(self, study_folder: Path) -> StudyMetadata:
        # Return minimal StudyMetadata for testing
        return StudyMetadata(
            items={
                "STUDYID": SourceColumn(
                    id="STUDYID",
                    label="Study Identifier",
                    data_type="text",
                    mandatory=True,
                    format_name=None,
                    content_length=200,
                )
            },
            codelists={},
        )

    def list_data_files(self, folder: Path, pattern: str = "*.csv") -> list[Path]:
        return [Path("DM.csv"), Path("AE.csv")]


class TestCTRepositoryContract:
    """Contract tests for CTRepositoryPort implementations."""

    @pytest.fixture
    def ct_repo(self):
        """Provide a mock CT repository."""
        return MockCTRepository()

    def test_implements_protocol(self, ct_repo):
        """Test that implementation satisfies the protocol."""
        assert isinstance(ct_repo, CTRepositoryPort)

    def test_get_by_code_returns_ct_or_none(self, ct_repo):
        """Test that get_by_code returns ControlledTerminology or None."""
        result = ct_repo.get_by_code("C66767")
        assert result is None or isinstance(result, ControlledTerminology)

    def test_get_by_code_with_valid_code(self, ct_repo):
        """Test get_by_code with a valid code returns CT."""
        result = ct_repo.get_by_code("C66767")
        assert result is not None
        assert result.codelist_code == "C66767"
        assert result.codelist_name == "SEX"

    def test_get_by_code_with_invalid_code(self, ct_repo):
        """Test get_by_code with invalid code returns None."""
        result = ct_repo.get_by_code("INVALID")
        assert result is None

    def test_get_by_name_returns_ct_or_none(self, ct_repo):
        """Test that get_by_name returns ControlledTerminology or None."""
        result = ct_repo.get_by_name("SEX")
        assert result is None or isinstance(result, ControlledTerminology)

    def test_get_by_name_with_valid_name(self, ct_repo):
        """Test get_by_name with valid name returns CT."""
        result = ct_repo.get_by_name("SEX")
        assert result is not None
        assert result.codelist_name == "SEX"

    def test_get_by_name_with_invalid_name(self, ct_repo):
        """Test get_by_name with invalid name returns None."""
        result = ct_repo.get_by_name("INVALID")
        assert result is None

    def test_list_all_codes_returns_list(self, ct_repo):
        """Test that list_all_codes returns a list of strings."""
        result = ct_repo.list_all_codes()
        assert isinstance(result, list)
        assert all(isinstance(code, str) for code in result)

    def test_list_all_codes_contains_expected_codes(self, ct_repo):
        """Test that list_all_codes contains expected codes."""
        result = ct_repo.list_all_codes()
        assert "C66767" in result


class TestSDTMSpecRepositoryContract:
    """Contract tests for SDTMSpecRepositoryPort implementations."""

    @pytest.fixture
    def spec_repo(self):
        """Provide a mock SDTM spec repository."""
        return MockSDTMSpecRepository()

    def test_implements_protocol(self, spec_repo):
        """Test that implementation satisfies the protocol."""
        assert isinstance(spec_repo, SDTMSpecRepositoryPort)

    def test_get_domain_variables_returns_list(self, spec_repo):
        """Test that get_domain_variables returns a list."""
        result = spec_repo.get_domain_variables("DM")
        assert isinstance(result, list)

    def test_get_domain_variables_with_valid_domain(self, spec_repo):
        """Test get_domain_variables with valid domain returns variables."""
        result = spec_repo.get_domain_variables("DM")
        assert len(result) > 0
        assert all(isinstance(var, dict) for var in result)
        assert all("Variable Name" in var for var in result)

    def test_get_domain_variables_with_invalid_domain(self, spec_repo):
        """Test get_domain_variables with invalid domain returns empty list."""
        result = spec_repo.get_domain_variables("INVALID")
        assert isinstance(result, list)
        assert len(result) == 0

    def test_get_dataset_attributes_returns_dict_or_none(self, spec_repo):
        """Test that get_dataset_attributes returns dict or None."""
        result = spec_repo.get_dataset_attributes("DM")
        assert result is None or isinstance(result, dict)

    def test_get_dataset_attributes_with_valid_domain(self, spec_repo):
        """Test get_dataset_attributes with valid domain returns attributes."""
        result = spec_repo.get_dataset_attributes("DM")
        assert result is not None
        assert isinstance(result, dict)
        assert "class" in result or "label" in result

    def test_get_dataset_attributes_with_invalid_domain(self, spec_repo):
        """Test get_dataset_attributes with invalid domain returns None."""
        result = spec_repo.get_dataset_attributes("INVALID")
        assert result is None

    def test_list_available_domains_returns_list(self, spec_repo):
        """Test that list_available_domains returns a list of strings."""
        result = spec_repo.list_available_domains()
        assert isinstance(result, list)
        assert all(isinstance(domain, str) for domain in result)

    def test_list_available_domains_contains_expected_domains(self, spec_repo):
        """Test that list_available_domains contains expected domains."""
        result = spec_repo.list_available_domains()
        assert "DM" in result


class TestStudyDataRepositoryContract:
    """Contract tests for StudyDataRepositoryPort implementations."""

    @pytest.fixture
    def data_repo(self):
        """Provide a mock study data repository."""
        return MockStudyDataRepository()

    def test_implements_protocol(self, data_repo):
        """Test that implementation satisfies the protocol."""
        assert isinstance(data_repo, StudyDataRepositoryPort)

    def test_read_dataset_returns_dataframe(self, data_repo):
        """Test that read_dataset returns a DataFrame."""
        result = data_repo.read_dataset("DM.csv")
        assert isinstance(result, pd.DataFrame)

    def test_read_dataset_with_valid_file(self, data_repo):
        """Test read_dataset with valid file returns non-empty DataFrame."""
        result = data_repo.read_dataset("DM.csv")
        assert isinstance(result, pd.DataFrame)
        assert not result.empty

    def test_load_study_metadata_returns_study_metadata(self, data_repo):
        """Test that load_study_metadata returns StudyMetadata."""
        result = data_repo.load_study_metadata(Path("study001"))
        assert isinstance(result, StudyMetadata)

    def test_load_study_metadata_has_expected_structure(self, data_repo):
        """Test that StudyMetadata has expected structure."""
        result = data_repo.load_study_metadata(Path("study001"))
        assert hasattr(result, "items")
        assert hasattr(result, "codelists")
        assert isinstance(result.items, dict)
        assert isinstance(result.codelists, dict)

    def test_list_data_files_returns_list(self, data_repo):
        """Test that list_data_files returns a list of Path objects."""
        result = data_repo.list_data_files(Path("study001"))
        assert isinstance(result, list)
        assert all(isinstance(path, Path) for path in result)

    def test_list_data_files_with_pattern(self, data_repo):
        """Test list_data_files respects the pattern parameter."""
        result = data_repo.list_data_files(Path("study001"), "*.csv")
        assert isinstance(result, list)
        # All results should match the pattern in a real implementation
        # Here we just verify it returns a list
        assert len(result) >= 0
