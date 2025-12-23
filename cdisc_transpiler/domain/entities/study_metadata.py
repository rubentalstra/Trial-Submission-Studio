"""Data models for metadata structures."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import pandas as pd


def _empty_codelist_values() -> list[CodeListValue]:
    return []


def _empty_items() -> dict[str, SourceColumn]:
    return {}


def _empty_codelists() -> dict[str, CodeList]:
    return {}


@dataclass
class CodeListValue:
    """A single value in a codelist."""

    code_value: str  # The code (e.g., "F", "1", "Y")
    code_text: str  # The text (e.g., "Female", "Asian", "Yes")
    data_type: str  # The data type (e.g., "text", "integer")


@dataclass
class CodeList:
    """A codelist with its values for normalizing source data."""

    format_name: str  # The codelist identifier (e.g., "SEX", "RACE", "YESNO")
    values: list[CodeListValue] = field(default_factory=_empty_codelist_values)

    def get_text(self, code: Any) -> str | None:
        """Get the text for a code value.

        Args:
            code: The code value to look up (can be any type, will be normalized)

        Returns:
            The text value if found, None otherwise
        """
        if code is None or pd.isna(code):
            return None

        # Normalize the lookup value
        code_str = str(code).strip().upper()

        for value in self.values:
            if str(value.code_value).strip().upper() == code_str:
                return value.code_text

        return None

    def get_code(self, text: Any) -> str | None:
        """Get the code for a text value (reverse lookup).

        Args:
            text: The text value to look up

        Returns:
            The code value if found, None otherwise
        """
        if text is None or pd.isna(text):
            return None

        text_str = str(text).strip().upper()

        for value in self.values:
            if str(value.code_text).strip().upper() == text_str:
                return value.code_value

        return None


@dataclass
class SourceColumn:
    """A source column definition from Items.csv."""

    id: str  # The column ID (e.g., "SEX", "SEXCD", "AGE")
    label: str  # Human-readable label
    data_type: str  # Data type (text, integer, double, date, time)
    mandatory: bool  # Whether the column is mandatory
    format_name: str | None  # Link to CodeLists.csv (e.g., "SEX", "RACE")
    content_length: int | None  # Expected content length

    @property
    def is_code_column(self) -> bool:
        """Check if this is a coded column (ends with CD)."""
        return self.id.endswith("CD") and self.format_name is not None

    @property
    def base_column_id(self) -> str:
        """Get the base column ID without CD suffix."""
        if self.id.endswith("CD"):
            return self.id[:-2]
        return self.id


@dataclass
class StudyMetadata:
    """Container for all metadata loaded from a study folder."""

    items: dict[str, SourceColumn] = field(default_factory=_empty_items)
    codelists: dict[str, CodeList] = field(default_factory=_empty_codelists)
    source_path: Path | None = None

    def get_column(self, column_id: str) -> SourceColumn | None:
        """Get a source column by ID (case-insensitive)."""
        return self.items.get(column_id.upper())

    def get_codelist(self, format_name: str) -> CodeList | None:
        """Get a codelist by format name (case-insensitive)."""
        return self.codelists.get(format_name.upper())

    def get_codelist_for_column(self, column_id: str) -> CodeList | None:
        """Get the codelist associated with a column."""
        column = self.get_column(column_id)
        if column and column.format_name:
            return self.get_codelist(column.format_name)
        return None

    def transform_value(self, column_id: str, value: Any) -> Any:
        """Transform a value using its codelist if applicable.

        Args:
            column_id: The source column ID
            value: The raw value to transform

        Returns:
            The transformed value, or the original if no transformation applies
        """
        column = self.get_column(column_id)
        if not column:
            return value

        # If this is a code column, look up the corresponding text value
        if column.is_code_column:
            codelist = (
                self.get_codelist(column.format_name) if column.format_name else None
            )
            if codelist:
                text = codelist.get_text(value)
                if text is not None:
                    return text

        return value
