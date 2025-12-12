"""Metadata loading and parsing for automatic SDTM mapping.

This module provides functionality to:
- Load Items.csv (source column definitions) and CodeLists.csv (value mappings)
- Parse and validate metadata structures
- Provide automatic mapping suggestions from source data to SDTM variables
- Apply codelist transformations to convert coded values to their text equivalents
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import pandas as pd


# =============================================================================
# Data Models
# =============================================================================


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
    values: list[CodeListValue] = field(default_factory=list)

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

    items: dict[str, SourceColumn] = field(default_factory=dict)
    codelists: dict[str, CodeList] = field(default_factory=dict)
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


# =============================================================================
# Metadata Loaders
# =============================================================================


class MetadataLoadError(Exception):
    """Raised when metadata cannot be loaded."""


def _detect_header_row(df: pd.DataFrame) -> int:
    """Detect which row contains the actual header.

    Items.csv and CodeLists.csv often have a human-readable header row
    followed by a code row. This detects that pattern.
    """
    if len(df) < 2:
        return 0

    # Check if first row has spaces (human-readable) and second row is codes
    first_row = df.iloc[0].astype(str)
    second_row = df.iloc[1].astype(str)

    first_has_spaces = first_row.str.contains(r"\s").any()
    second_is_codes = second_row.str.match(r"^[A-Za-z][A-Za-z0-9_]*$").all()

    if first_has_spaces and second_is_codes:
        return 1

    return 0


def load_items_csv(path: Path) -> dict[str, SourceColumn]:
    """Load Items.csv and return a dictionary of source columns.

    Args:
        path: Path to the Items.csv file

    Returns:
        Dictionary mapping column ID to SourceColumn

    Raises:
        MetadataLoadError: If the file cannot be loaded or parsed
    """
    if not path.exists():
        raise MetadataLoadError(f"Items.csv not found: {path}")

    try:
        # Read with no header first to detect structure
        df_raw = pd.read_csv(path, header=None, nrows=5)
        header_row = _detect_header_row(df_raw)

        # Re-read with correct header
        df = pd.read_csv(path, header=header_row)

        # Normalize column names
        df.columns = [str(c).strip() for c in df.columns]

        # Map various column name variations
        col_mappings = {
            "id": ["ID", "Id", "ColumnId", "Column_Id", "ColumnID"],
            "label": ["Label", "ColumnLabel", "Column_Label", "Description"],
            "data_type": ["DataType", "Data Type", "Data_Type", "Type"],
            "mandatory": ["Mandatory", "Required", "Req"],
            "format_name": [
                "FormatName",
                "Format Name",
                "Format_Name",
                "CodeList",
                "Codelist",
            ],
            "content_length": [
                "ContentLength",
                "Content Length",
                "Content_Length",
                "Length",
            ],
        }

        def find_column(df: pd.DataFrame, options: list[str]) -> str | None:
            for opt in options:
                if opt in df.columns:
                    return opt
                # Case-insensitive match
                for col in df.columns:
                    if col.lower() == opt.lower():
                        return col
            return None

        id_col = find_column(df, col_mappings["id"])
        label_col = find_column(df, col_mappings["label"])
        dtype_col = find_column(df, col_mappings["data_type"])
        mandatory_col = find_column(df, col_mappings["mandatory"])
        format_col = find_column(df, col_mappings["format_name"])
        length_col = find_column(df, col_mappings["content_length"])

        if not id_col:
            raise MetadataLoadError(f"Could not find ID column in {path}")

        items: dict[str, SourceColumn] = {}

        for _, row in df.iterrows():
            col_id = str(row.get(id_col, "")).strip()
            if not col_id or col_id.lower() in ("id", "columnid"):
                continue

            label = str(row.get(label_col, col_id)) if label_col else col_id
            data_type = str(row.get(dtype_col, "text")).lower() if dtype_col else "text"

            # Handle mandatory field
            mandatory_val = row.get(mandatory_col, "") if mandatory_col else ""
            mandatory = str(mandatory_val).strip().lower() in (
                "true",
                "yes",
                "1",
                "y",
                "req",
            )

            # Handle format name
            format_name = None
            if format_col and pd.notna(row.get(format_col)):
                format_val = str(row.get(format_col)).strip()
                if format_val and format_val.lower() not in ("", "nan", "none"):
                    format_name = format_val

            # Handle content length
            content_length = None
            if length_col and pd.notna(row.get(length_col)):
                try:
                    content_length = int(float(row.get(length_col)))
                except (ValueError, TypeError):
                    pass

            items[col_id.upper()] = SourceColumn(
                id=col_id,
                label=label,
                data_type=data_type,
                mandatory=mandatory,
                format_name=format_name,
                content_length=content_length,
            )

        return items

    except Exception as exc:
        if isinstance(exc, MetadataLoadError):
            raise
        raise MetadataLoadError(f"Failed to load Items.csv: {exc}") from exc


def load_codelists_csv(path: Path) -> dict[str, CodeList]:
    """Load CodeLists.csv and return a dictionary of codelists.

    Args:
        path: Path to the CodeLists.csv file

    Returns:
        Dictionary mapping format name to CodeList

    Raises:
        MetadataLoadError: If the file cannot be loaded or parsed
    """
    if not path.exists():
        raise MetadataLoadError(f"CodeLists.csv not found: {path}")

    try:
        # Read with no header first to detect structure
        df_raw = pd.read_csv(path, header=None, nrows=5)
        header_row = _detect_header_row(df_raw)

        # Re-read with correct header
        df = pd.read_csv(path, header=header_row)

        # Normalize column names
        df.columns = [str(c).strip() for c in df.columns]

        # Map various column name variations
        col_mappings = {
            "format_name": [
                "FormatName",
                "Format Name",
                "Format_Name",
                "CodeListName",
                "Name",
            ],
            "data_type": ["DataType", "Data Type", "Data_Type", "Type"],
            "code_value": ["CodeValue", "Code Value", "Code_Value", "Code", "Value"],
            "code_text": [
                "CodeText",
                "Code Text",
                "Code_Text",
                "Text",
                "Label",
                "Decode",
            ],
        }

        def find_column(df: pd.DataFrame, options: list[str]) -> str | None:
            for opt in options:
                if opt in df.columns:
                    return opt
                for col in df.columns:
                    if col.lower() == opt.lower():
                        return col
            return None

        format_col = find_column(df, col_mappings["format_name"])
        dtype_col = find_column(df, col_mappings["data_type"])
        value_col = find_column(df, col_mappings["code_value"])
        text_col = find_column(df, col_mappings["code_text"])

        if not format_col or not value_col or not text_col:
            raise MetadataLoadError(
                f"Could not find required columns in {path}. Found: {list(df.columns)}"
            )

        codelists: dict[str, CodeList] = {}

        for _, row in df.iterrows():
            format_name = str(row.get(format_col, "")).strip()
            if not format_name or format_name.lower() in ("formatname", "format name"):
                continue

            code_value = str(row.get(value_col, "")).strip()
            code_text = str(row.get(text_col, "")).strip()
            data_type = str(row.get(dtype_col, "text")).lower() if dtype_col else "text"

            if not code_value or not code_text:
                continue

            format_key = format_name.upper()
            if format_key not in codelists:
                codelists[format_key] = CodeList(format_name=format_name)

            codelists[format_key].values.append(
                CodeListValue(
                    code_value=code_value,
                    code_text=code_text,
                    data_type=data_type,
                )
            )

        return codelists

    except Exception as exc:
        if isinstance(exc, MetadataLoadError):
            raise
        raise MetadataLoadError(f"Failed to load CodeLists.csv: {exc}") from exc


def discover_metadata_files(study_folder: Path) -> tuple[Path | None, Path | None]:
    """Discover Items.csv and CodeLists.csv files in a study folder.

    Args:
        study_folder: Path to the study folder

    Returns:
        Tuple of (items_path, codelists_path), either can be None if not found
    """
    items_path = None
    codelists_path = None

    for csv_file in study_folder.glob("*.csv"):
        filename_upper = csv_file.stem.upper()

        if "ITEMS" in filename_upper:
            items_path = csv_file
        elif "CODELIST" in filename_upper:
            codelists_path = csv_file

    return items_path, codelists_path


def load_study_metadata(study_folder: Path) -> StudyMetadata:
    """Load all metadata from a study folder.

    Args:
        study_folder: Path to the study folder

    Returns:
        StudyMetadata container with all loaded metadata
    """
    items_path, codelists_path = discover_metadata_files(study_folder)

    metadata = StudyMetadata(source_path=study_folder)

    if items_path:
        try:
            metadata.items = load_items_csv(items_path)
        except MetadataLoadError:
            pass  # Continue without items

    if codelists_path:
        try:
            metadata.codelists = load_codelists_csv(codelists_path)
        except MetadataLoadError:
            pass  # Continue without codelists

    return metadata


# =============================================================================
# SDTM Mapping Utilities
# =============================================================================


# Known patterns for mapping source columns to SDTM variables
# These are common patterns in EDC exports that map to SDTM
_SDTM_COLUMN_PATTERNS: dict[str, list[str]] = {
    # Demographics (DM)
    "USUBJID": ["SUBJECTID", "SUBJECTIDENTIFIER", "PATIENTID", "SUBJECT"],
    "SEX": ["SEX", "GENDER"],
    "AGE": ["AGE"],
    "AGEU": ["AGEU", "AGEUNIT", "AGEUNITS"],
    "RACE": ["RACE"],
    "ETHNIC": ["ETHNIC", "ETHNICITY"],
    "RFSTDTC": ["ICDAT", "INFORMEDCONSENTDATE", "RFSTDTC"],
    "BRTHDTC": ["BRTHDTC", "BIRTHDATE", "DOB"],
    "COUNTRY": ["COUNTRY", "COUNTRYCD"],
    "SITEID": ["SITEID", "SITECODE", "SITE"],
    # Common timing variables
    "EPOCH": ["EPOCH", "VISITEPOCH"],
    "VISITNUM": ["VISITNUM", "VISITNUMBER"],
    "VISIT": ["VISIT", "VISITNAME", "EVENTNAME"],
    # Common result variables (findings)
    "--ORRES": ["ORRES", "RESULT", "VALUE"],
    "--ORRESU": ["ORRESU", "UNIT", "UNITS"],
    "--STRESC": ["STRESC", "STANDARDRESULT"],
    "--STRESN": ["STRESN", "NUMERICRESULT"],
    "--STRESU": ["STRESU", "STANDARDUNIT"],
    # Sequence
    "--SEQ": ["SEQ", "EVENTSEQ", "EVENTSEQUENCENUMBER"],
    # Start/end dates
    "--STDTC": ["STDTC", "STDAT", "STARTDATE", "STARTDATETIME"],
    "--ENDTC": ["ENDTC", "ENDAT", "ENDDATE", "ENDDATETIME"],
}


def _normalize_column_name(name: str) -> str:
    """Normalize a column name for comparison."""
    return re.sub(r"[^A-Z0-9]", "", name.upper())


def infer_sdtm_target(
    source_column: str,
    domain_code: str,
    items: dict[str, SourceColumn] | None = None,
) -> str | None:
    """Infer the SDTM target variable for a source column.

    Args:
        source_column: The source column name
        domain_code: The target SDTM domain code
        items: Optional Items.csv metadata

    Returns:
        The inferred SDTM variable name, or None if no match
    """
    normalized = _normalize_column_name(source_column)
    domain_prefix = domain_code.upper()

    # Check if column already has domain prefix (e.g., AETERM, LBORRES)
    if normalized.startswith(domain_prefix):
        # Could be a valid SDTM variable already
        return source_column.upper()

    # Check common patterns
    for sdtm_var, patterns in _SDTM_COLUMN_PATTERNS.items():
        for pattern in patterns:
            if _normalize_column_name(pattern) == normalized:
                # Handle domain-specific variables
                if sdtm_var.startswith("--"):
                    return domain_prefix + sdtm_var[2:]
                return sdtm_var

    # Check if Items.csv provides hints
    if items:
        item = items.get(normalized)
        if item:
            # If label contains SDTM variable name hints
            label_normalized = _normalize_column_name(item.label)
            for sdtm_var, patterns in _SDTM_COLUMN_PATTERNS.items():
                for pattern in patterns:
                    if _normalize_column_name(pattern) in label_normalized:
                        if sdtm_var.startswith("--"):
                            return domain_prefix + sdtm_var[2:]
                        return sdtm_var

    return None


def get_value_transformer(
    source_column: str,
    metadata: StudyMetadata,
    target_variable: str,
) -> callable | None:
    """Get a transformation function for a source column.

    Args:
        source_column: The source column name
        metadata: The study metadata
        target_variable: The SDTM target variable

    Returns:
        A callable that transforms values, or None if no transformation needed
    """
    # Check if there's a code column with a codelist
    code_column = source_column + "CD"
    column_def = metadata.get_column(code_column)

    if column_def and column_def.format_name:
        codelist = metadata.get_codelist(column_def.format_name)
        if codelist:
            # Return a transformer that converts codes to text
            def transformer(value: Any) -> Any:
                if pd.isna(value):
                    return value
                result = codelist.get_text(value)
                return result if result is not None else value

            return transformer

    # Check the column itself for a codelist
    column_def = metadata.get_column(source_column)
    if column_def and column_def.format_name:
        codelist = metadata.get_codelist(column_def.format_name)
        if codelist:

            def transformer(value: Any) -> Any:
                if pd.isna(value):
                    return value
                result = codelist.get_text(value)
                return result if result is not None else value

            return transformer

    return None
