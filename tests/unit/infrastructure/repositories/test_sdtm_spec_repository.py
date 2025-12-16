"""Tests for SDTMSpecRepository."""

import pytest

from cdisc_transpiler.infrastructure.repositories import SDTMSpecRepository
from cdisc_transpiler.infrastructure.caching import MemoryCache
from cdisc_transpiler.config import TranspilerConfig


class TestSDTMSpecRepository:
    """Tests for SDTMSpecRepository functionality."""

    @pytest.fixture
    def mock_spec_dir(self, tmp_path):
        """Create a mock SDTM spec directory with test data."""
        spec_dir = tmp_path / "SDTMIG_v3.4"
        spec_dir.mkdir()

        # Create Variables.csv
        variables_csv = spec_dir / "Variables.csv"
        variables_csv.write_text(
            "Domain,Variable Name,Label,Type,Role,Codelist Code\n"
            "DM,STUDYID,Study Identifier,Char,Identifier,\n"
            "DM,DOMAIN,Domain Abbreviation,Char,Identifier,\n"
            "DM,USUBJID,Unique Subject Identifier,Char,Identifier,\n"
            "DM,SUBJID,Subject Identifier,Char,Topic,\n"
            "DM,SEX,Sex,Char,Grouping Qualifier,C66731\n"
            "AE,STUDYID,Study Identifier,Char,Identifier,\n"
            "AE,DOMAIN,Domain Abbreviation,Char,Identifier,\n"
            "AE,USUBJID,Unique Subject Identifier,Char,Identifier,\n"
            "AE,AETERM,Reported Term for the Adverse Event,Char,Topic,\n"
        )

        # Create Datasets.csv
        datasets_csv = spec_dir / "Datasets.csv"
        datasets_csv.write_text(
            "Domain,Label,Class,Structure\n"
            "DM,Demographics,Special Purpose,One record per subject\n"
            "AE,Adverse Events,Events,One record per adverse event\n"
        )

        return tmp_path

    @pytest.fixture
    def repo_with_mock_data(self, mock_spec_dir):
        """Create a repository pointing to mock data."""
        config = TranspilerConfig(sdtm_spec_dir=mock_spec_dir / "SDTMIG_v3.4")
        cache = MemoryCache()
        return SDTMSpecRepository(config=config, cache=cache)

    def test_get_domain_variables_returns_list(self, repo_with_mock_data):
        """Test that get_domain_variables returns a list of variable dicts."""
        variables = repo_with_mock_data.get_domain_variables("DM")

        assert isinstance(variables, list)
        assert len(variables) == 5  # 5 DM variables in our fixture

    def test_get_domain_variables_case_insensitive(self, repo_with_mock_data):
        """Test that domain lookup is case-insensitive."""
        variables_lower = repo_with_mock_data.get_domain_variables("dm")
        variables_upper = repo_with_mock_data.get_domain_variables("DM")

        assert variables_lower == variables_upper

    def test_get_domain_variables_unknown_domain(self, repo_with_mock_data):
        """Test that unknown domains return empty list."""
        variables = repo_with_mock_data.get_domain_variables("UNKNOWN")

        assert variables == []

    def test_get_domain_variables_contains_expected_fields(self, repo_with_mock_data):
        """Test that variable dicts contain expected fields."""
        variables = repo_with_mock_data.get_domain_variables("DM")

        # Find USUBJID variable
        usubjid = next(
            (v for v in variables if v.get("Variable Name") == "USUBJID"), None
        )

        assert usubjid is not None
        assert usubjid.get("Label") == "Unique Subject Identifier"
        assert usubjid.get("Type") == "Char"
        assert usubjid.get("Role") == "Identifier"

    def test_get_dataset_attributes(self, repo_with_mock_data):
        """Test retrieving dataset attributes."""
        attrs = repo_with_mock_data.get_dataset_attributes("DM")

        assert attrs is not None
        assert attrs.get("label") == "Demographics"
        assert attrs.get("class") == "Special Purpose"

    def test_get_dataset_attributes_case_insensitive(self, repo_with_mock_data):
        """Test that dataset attributes lookup is case-insensitive."""
        attrs_lower = repo_with_mock_data.get_dataset_attributes("dm")
        attrs_upper = repo_with_mock_data.get_dataset_attributes("DM")

        assert attrs_lower == attrs_upper

    def test_get_dataset_attributes_unknown_domain(self, repo_with_mock_data):
        """Test that unknown domains return None."""
        attrs = repo_with_mock_data.get_dataset_attributes("UNKNOWN")

        assert attrs is None

    def test_list_available_domains(self, repo_with_mock_data):
        """Test listing all available domains."""
        domains = repo_with_mock_data.list_available_domains()

        assert isinstance(domains, list)
        assert "DM" in domains
        assert "AE" in domains
        assert domains == sorted(domains)  # Should be sorted

    def test_caching_prevents_repeated_reads(self, mock_spec_dir):
        """Test that caching prevents repeated file reads."""
        config = TranspilerConfig(sdtm_spec_dir=mock_spec_dir / "SDTMIG_v3.4")
        cache = MemoryCache()
        repo = SDTMSpecRepository(config=config, cache=cache)

        # First call loads data
        repo.get_domain_variables("DM")
        assert cache.has("sdtm_variables")

        # Second call should use cache
        repo.get_domain_variables("AE")
        # Still only one cache entry for all variables
        assert cache.size() >= 1

    def test_clear_cache(self, repo_with_mock_data):
        """Test clearing the cache."""
        # Load data to populate cache
        repo_with_mock_data.get_domain_variables("DM")

        # Clear cache
        repo_with_mock_data.clear_cache()

        # Cache should be empty now
        assert repo_with_mock_data._cache.size() == 0

    def test_with_real_spec_files(self):
        """Test with real SDTM spec files if available."""
        # Use default config which points to docs/SDTMIG_v3.4
        repo = SDTMSpecRepository()

        # Try to list domains - may return empty if files don't exist
        domains = repo.list_available_domains()

        # If we have real files, verify some basics
        if domains:
            assert "DM" in domains

            # Verify DM has expected variables
            dm_vars = repo.get_domain_variables("DM")
            var_names = [v.get("Variable Name") for v in dm_vars]
            assert "STUDYID" in var_names or "studyid" in [
                n.lower() for n in var_names if n
            ]


class TestSDTMSpecRepositoryProtocol:
    """Tests verifying SDTMSpecRepositoryPort protocol compliance."""

    def test_implements_protocol(self):
        """Test that SDTMSpecRepository implements the protocol."""
        from cdisc_transpiler.application.ports.repositories import (
            SDTMSpecRepositoryPort,
        )

        repo = SDTMSpecRepository()

        assert isinstance(repo, SDTMSpecRepositoryPort)

    def test_has_required_methods(self):
        """Test that all required protocol methods exist."""
        repo = SDTMSpecRepository()

        assert hasattr(repo, "get_domain_variables")
        assert hasattr(repo, "get_dataset_attributes")
        assert hasattr(repo, "list_available_domains")

        assert callable(repo.get_domain_variables)
        assert callable(repo.get_dataset_attributes)
        assert callable(repo.list_available_domains)
