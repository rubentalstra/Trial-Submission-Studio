"""Unit tests for CSVReader."""

from pathlib import Path

import pandas as pd
import pytest

from cdisc_transpiler.infrastructure.io.csv_reader import CSVReader, CSVReadOptions
from cdisc_transpiler.infrastructure.io.exceptions import (
    DataParseError,
    DataSourceNotFoundError,
)


class TestCSVReader:
    """Test suite for CSVReader class."""

    def test_read_simple_csv(self, tmp_path: Path):
        """Test reading a simple CSV file."""
        # Arrange
        csv_file = tmp_path / "test.csv"
        csv_file.write_text("Column1,Column2,Column3\nA,B,C\n1,2,3\n")
        reader = CSVReader()

        # Act
        df = reader.read(csv_file)

        # Assert
        assert len(df) == 2
        assert list(df.columns) == ["Column1", "Column2", "Column3"]
        assert df.iloc[0, 0] == "A"

    def test_read_with_header_normalization(self, tmp_path: Path):
        """Test that headers are normalized (whitespace stripped)."""
        # Arrange
        csv_file = tmp_path / "test.csv"
        # Make sure header detection doesn't trigger - no spaces in values
        csv_file.write_text(" Column1 , Column2  ,Column3\n1,2,3\n")
        reader = CSVReader()

        # Act
        df = reader.read(csv_file)

        # Assert
        assert list(df.columns) == ["Column1", "Column2", "Column3"]

    def test_read_without_header_normalization(self, tmp_path: Path):
        """Test that header normalization can be disabled."""
        # Arrange
        csv_file = tmp_path / "test.csv"
        csv_file.write_text(" Column1 , Column2  ,Column3\n1,2,3\n")
        reader = CSVReader()
        options = CSVReadOptions(normalize_headers=False)

        # Act
        df = reader.read(csv_file, options=options)

        # Assert
        assert " Column1 " in df.columns
        assert " Column2  " in df.columns

    def test_read_with_na_handling(self, tmp_path: Path):
        """Test strict NA handling."""
        # Arrange
        csv_file = tmp_path / "test.csv"
        csv_file.write_text("Col1,Col2,Col3\nA,,C\n,B,\n")
        reader = CSVReader()

        # Act - strict NA handling treats empty strings as NA
        df = reader.read(csv_file)

        # Assert
        assert pd.isna(df.iloc[0, 1])
        assert pd.isna(df.iloc[1, 0])

    def test_detect_header_row_with_human_readable_first_row(self, tmp_path: Path):
        """Test detection of code row as header when first row is human-readable."""
        # Arrange
        csv_file = tmp_path / "test.csv"
        # First row has spaces (human-readable), second row is codes
        csv_file.write_text(
            "Subject Identifier,Test Code,Test Value\n"
            "USUBJID,TESTCD,TESTVAL\n"
            "001,HR,72\n"
        )
        reader = CSVReader()

        # Act
        df = reader.read(csv_file)

        # Assert
        assert list(df.columns) == ["USUBJID", "TESTCD", "TESTVAL"]
        assert df.iloc[0, 0] == "001"  # First data row

    def test_detect_header_row_normal_case(self, tmp_path: Path):
        """Test that normal headers are used when no code row detected."""
        # Arrange
        csv_file = tmp_path / "test.csv"
        csv_file.write_text("USUBJID,TESTCD,TESTVAL\n001,HR,72\n")
        reader = CSVReader()

        # Act
        df = reader.read(csv_file)

        # Assert
        assert list(df.columns) == ["USUBJID", "TESTCD", "TESTVAL"]
        assert df.iloc[0, 0] == "001"

    def test_read_file_not_found(self, tmp_path: Path):
        """Test error handling when file doesn't exist."""
        # Arrange
        csv_file = tmp_path / "nonexistent.csv"
        reader = CSVReader()

        # Act & Assert
        with pytest.raises(DataSourceNotFoundError, match="File not found"):
            reader.read(csv_file)

    def test_read_malformed_csv(self, tmp_path: Path):
        """Test error handling for malformed CSV."""
        # Arrange
        csv_file = tmp_path / "malformed.csv"
        # Create a CSV with inconsistent column counts
        csv_file.write_text("Col1,Col2,Col3\nA,B\nC,D,E,F\n")
        reader = CSVReader()

        # Act & Assert
        with pytest.raises(DataParseError, match="Failed to parse"):
            reader.read(csv_file)

    def test_read_empty_csv(self, tmp_path: Path):
        """Test error handling for empty CSV file."""
        # Arrange
        csv_file = tmp_path / "empty.csv"
        csv_file.write_text("")
        reader = CSVReader()

        # Act & Assert
        with pytest.raises(DataParseError):
            reader.read(csv_file)

    def test_read_csv_with_no_columns(self, tmp_path: Path):
        """Test error handling for CSV with no columns."""
        # Arrange
        csv_file = tmp_path / "nocols.csv"
        csv_file.write_text("\n\n\n")
        reader = CSVReader()

        # Act & Assert
        with pytest.raises(DataParseError, match="empty"):
            reader.read(csv_file)

    def test_read_with_custom_dtype(self, tmp_path: Path):
        """Test reading with custom dtype specification."""
        # Arrange
        csv_file = tmp_path / "test.csv"
        csv_file.write_text("Col1,Col2\n1,2\n3,4\n")
        reader = CSVReader()
        options = CSVReadOptions(dtype={"Col1": int, "Col2": int})

        # Act
        df = reader.read(csv_file, options=options)

        # Assert
        assert df["Col1"].dtype == int
        assert df["Col2"].dtype == int

    def test_read_with_encoding(self, tmp_path: Path):
        """Test reading CSV with different encoding."""
        # Arrange
        csv_file = tmp_path / "test.csv"
        # Write UTF-8 with BOM
        csv_file.write_bytes(b"\xef\xbb\xbfCol1,Col2\nA,B\n")
        reader = CSVReader()
        options = CSVReadOptions(encoding="utf-8-sig")

        # Act
        df = reader.read(csv_file, options=options)

        # Assert
        assert "Col1" in df.columns
        assert not df.columns[0].startswith("\ufeff")


class TestCSVReadOptions:
    """Test suite for CSVReadOptions dataclass."""

    def test_default_options(self):
        """Test default option values."""
        options = CSVReadOptions()

        assert options.normalize_headers is True
        assert options.strict_na_handling is True
        assert options.dtype == str
        assert options.encoding == "utf-8"
        assert options.detect_header_row is True

    def test_custom_options(self):
        """Test creating options with custom values."""
        options = CSVReadOptions(
            normalize_headers=False,
            strict_na_handling=False,
            dtype=None,
            encoding="latin-1",
            detect_header_row=False,
        )

        assert options.normalize_headers is False
        assert options.strict_na_handling is False
        assert options.dtype is None
        assert options.encoding == "latin-1"
        assert options.detect_header_row is False
