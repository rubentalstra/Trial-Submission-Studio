"""Tests for StudyDataRepository."""

import pytest
from pathlib import Path

import pandas as pd

from cdisc_transpiler.infrastructure.repositories import StudyDataRepository
from cdisc_transpiler.infrastructure.io.exceptions import (
    DataParseError,
    DataSourceNotFoundError,
)


class TestStudyDataRepository:
    """Tests for StudyDataRepository functionality."""

    @pytest.fixture
    def repo(self):
        """Create a repository instance."""
        return StudyDataRepository()

    @pytest.fixture
    def mock_study_folder(self, tmp_path):
        """Create a mock study folder with test data."""
        study_dir = tmp_path / "study001"
        study_dir.mkdir()

        # Create DM.csv
        dm_csv = study_dir / "DM.csv"
        dm_csv.write_text(
            "STUDYID,DOMAIN,USUBJID,SUBJID,SEX,AGE\n"
            "STUDY001,DM,STUDY001-001,001,M,35\n"
            "STUDY001,DM,STUDY001-002,002,F,42\n"
        )

        # Create AE.csv
        ae_csv = study_dir / "AE.csv"
        ae_csv.write_text(
            "STUDYID,DOMAIN,USUBJID,AETERM,AESEV\n"
            "STUDY001,AE,STUDY001-001,Headache,MILD\n"
        )

        # Create Items.csv (metadata)
        items_csv = study_dir / "Items.csv"
        items_csv.write_text(
            "ID,Label,DataType\nSTUDYID,Study Identifier,text\nSEX,Sex,text\n"
        )

        # Create CodeLists.csv (metadata)
        codelists_csv = study_dir / "CodeLists.csv"
        codelists_csv.write_text(
            "FormatName,CodeValue,CodeText\nSEX,M,Male\nSEX,F,Female\n"
        )

        return study_dir

    def test_read_dataset_csv(self, repo, mock_study_folder):
        """Test reading a CSV dataset."""
        df = repo.read_dataset(mock_study_folder / "DM.csv")

        assert isinstance(df, pd.DataFrame)
        assert len(df) == 2
        assert "STUDYID" in df.columns
        assert "USUBJID" in df.columns

    def test_read_dataset_not_found(self, repo, tmp_path):
        """Test reading a non-existent file raises error."""
        with pytest.raises(DataSourceNotFoundError):
            repo.read_dataset(tmp_path / "nonexistent.csv")

    def test_read_dataset_unsupported_format(self, repo, tmp_path):
        """Test reading an unsupported format raises error."""
        unsupported = tmp_path / "data.json"
        unsupported.write_text('{"key": "value"}')

        with pytest.raises(DataParseError, match="Unsupported format"):
            repo.read_dataset(unsupported)

    def test_read_dataset_not_a_file(self, repo, tmp_path):
        """Test reading a directory raises error."""
        with pytest.raises(DataSourceNotFoundError, match="Not a file"):
            repo.read_dataset(tmp_path)

    def test_load_study_metadata(self, repo, mock_study_folder):
        """Test loading study metadata."""
        metadata = repo.load_study_metadata(mock_study_folder)

        assert metadata is not None
        assert metadata.source_path == mock_study_folder

        # Should have loaded items
        if metadata.items:
            assert "STUDYID" in metadata.items

    def test_load_study_metadata_missing_folder(self, repo, tmp_path):
        """Test loading metadata from missing folder returns empty metadata."""
        metadata = repo.load_study_metadata(tmp_path / "nonexistent")

        assert metadata is not None
        # Empty dict is acceptable for missing items/codelists
        assert not metadata.items or metadata.items == {}
        assert not metadata.codelists or metadata.codelists == {}

    def test_load_study_metadata_no_metadata_files(self, repo, tmp_path):
        """Test loading metadata when no metadata files exist."""
        empty_folder = tmp_path / "empty_study"
        empty_folder.mkdir()

        metadata = repo.load_study_metadata(empty_folder)

        assert metadata is not None
        # Should gracefully handle missing files (empty dict or None)
        assert not metadata.items or metadata.items == {}
        assert not metadata.codelists or metadata.codelists == {}

    def test_list_data_files(self, repo, mock_study_folder):
        """Test listing data files."""
        files = repo.list_data_files(mock_study_folder)

        assert isinstance(files, list)
        assert len(files) >= 2  # At least DM.csv and AE.csv

        # Check that CSV files are found
        filenames = [f.name for f in files]
        assert "DM.csv" in filenames
        assert "AE.csv" in filenames

    def test_list_data_files_with_pattern(self, repo, mock_study_folder):
        """Test listing files with a specific pattern."""
        # Create a txt file
        txt_file = mock_study_folder / "readme.txt"
        txt_file.write_text("readme")

        # List only txt files
        files = repo.list_data_files(mock_study_folder, pattern="*.txt")

        filenames = [f.name for f in files]
        assert "readme.txt" in filenames
        assert "DM.csv" not in filenames

    def test_list_data_files_nonexistent_folder(self, repo, tmp_path):
        """Test listing files from non-existent folder returns empty list."""
        files = repo.list_data_files(tmp_path / "nonexistent")

        assert files == []

    def test_list_data_files_sorted(self, repo, mock_study_folder):
        """Test that file list is sorted."""
        files = repo.list_data_files(mock_study_folder)

        # Should be sorted by name
        assert files == sorted(files)


class TestStudyDataRepositoryProtocol:
    """Tests verifying StudyDataRepositoryPort protocol compliance."""

    def test_implements_protocol(self):
        """Test that StudyDataRepository implements the protocol."""
        from cdisc_transpiler.application.ports.repositories import (
            StudyDataRepositoryPort,
        )

        repo = StudyDataRepository()

        assert isinstance(repo, StudyDataRepositoryPort)

    def test_has_required_methods(self):
        """Test that all required protocol methods exist."""
        repo = StudyDataRepository()

        assert hasattr(repo, "read_dataset")
        assert hasattr(repo, "load_study_metadata")
        assert hasattr(repo, "list_data_files")

        assert callable(repo.read_dataset)
        assert callable(repo.load_study_metadata)
        assert callable(repo.list_data_files)
