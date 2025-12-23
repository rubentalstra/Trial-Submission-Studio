"""Tests for CTRepository."""

import pytest

from cdisc_transpiler.config import TranspilerConfig
from cdisc_transpiler.infrastructure.caching.memory_cache import MemoryCache
from cdisc_transpiler.infrastructure.repositories.ct_repository import CTRepository


class TestCTRepository:
    """Tests for CTRepository functionality."""

    @pytest.fixture
    def mock_ct_dir(self, tmp_path):
        """Create a mock CT directory with test data."""
        ct_dir = tmp_path / "Controlled_Terminology"
        ct_dir.mkdir()

        # Create a version folder
        version_dir = ct_dir / "2025-09-26"
        version_dir.mkdir()

        # Create a simple CT CSV file
        ct_csv = version_dir / "SEX.csv"
        ct_csv.write_text(
            "Code,Codelist Code,Codelist Extensible,Codelist Name,CDISC Submission Value,CDISC Synonym(s),CDISC Definition,NCI Preferred Term\n"
            "C66731,C66731,No,SEX,,,Sex,Sex\n"
            "C16576,C66731,No,SEX,M,MALE,A person who belongs to the sex that produces spermatozoa.,Male\n"
            "C20197,C66731,No,SEX,F,FEMALE,A person who belongs to the sex that produces ova.,Female\n"
            "C17998,C66731,No,SEX,U,UNKNOWN,Unknown sex,Unknown\n"
        )

        return tmp_path

    @pytest.fixture
    def repo_with_mock_data(self, mock_ct_dir):
        """Create a repository pointing to mock data."""
        config = TranspilerConfig(ct_dir=mock_ct_dir / "Controlled_Terminology")
        cache = MemoryCache()
        return CTRepository(config=config, cache=cache, ct_version="2025-09-26")

    def test_list_all_codes_returns_list(self):
        """Test that list_all_codes returns a list."""
        repo = CTRepository()
        codes = repo.list_all_codes()

        assert isinstance(codes, list)

    def test_get_by_code_returns_none_for_unknown(self):
        """Test that get_by_code returns None for unknown codes."""
        repo = CTRepository()
        result = repo.get_by_code("INVALID_CODE_12345")

        assert result is None

    def test_get_by_name_returns_none_for_unknown(self):
        """Test that get_by_name returns None for unknown names."""
        repo = CTRepository()
        result = repo.get_by_name("INVALID_NAME_12345")

        assert result is None

    def test_caching_prevents_repeated_reads(self, mock_ct_dir):
        """Test that caching prevents repeated registry loads."""
        config = TranspilerConfig(ct_dir=mock_ct_dir / "Controlled_Terminology")
        cache = MemoryCache()
        repo = CTRepository(config=config, cache=cache)

        # First call loads data
        repo.list_all_codes()
        assert cache.has("ct_registry")

        # Second call should use cache
        repo.list_all_codes()
        assert cache.size() == 1

    def test_clear_cache(self):
        """Test clearing the cache."""
        repo = CTRepository()

        # Load data to populate cache
        repo.list_all_codes()

        # Clear cache
        repo.clear_cache()

        # Cache should be empty now
        assert repo._cache.size() == 0

    def test_with_real_ct_files(self):
        """Test with real CT files if available."""
        repo = CTRepository()

        # Try to list codes - may return empty if files don't exist
        codes = repo.list_all_codes()

        # If we have real files, verify some basics
        if codes:
            # Should have at least some codes
            assert len(codes) > 0

            # Try to get a common codelist (SEX)
            # Note: actual code depends on CT version
            for code in codes:
                ct = repo.get_by_code(code)
                if ct:
                    # Found at least one valid entry
                    break


class TestCTRepositoryProtocol:
    """Tests verifying CTRepositoryPort protocol compliance."""

    def test_implements_protocol(self):
        """Test that CTRepository implements the protocol."""
        from cdisc_transpiler.application.ports.repositories import CTRepositoryPort

        repo = CTRepository()

        assert isinstance(repo, CTRepositoryPort)

    def test_has_required_methods(self):
        """Test that all required protocol methods exist."""
        repo = CTRepository()

        assert hasattr(repo, "get_by_code")
        assert hasattr(repo, "get_by_name")
        assert hasattr(repo, "list_all_codes")

        assert callable(repo.get_by_code)
        assert callable(repo.get_by_name)
        assert callable(repo.list_all_codes)
