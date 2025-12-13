"""Metadata loaders for Items.csv and CodeLists.csv files."""

from __future__ import annotations

from pathlib import Path

import pandas as pd

from .csv_utils import detect_header_row, find_column, normalize_column_names
from .models import CodeList, CodeListValue, SourceColumn, StudyMetadata


class MetadataLoadError(Exception):
    """Raised when metadata cannot be loaded."""


# Column name mappings for Items.csv
_ITEMS_COLUMN_MAPPINGS = {
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

# Column name mappings for CodeLists.csv
_CODELISTS_COLUMN_MAPPINGS = {
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


def _parse_mandatory_field(value: any) -> bool:
    """Parse a mandatory field value to boolean."""
    if not value:
        return False
    return str(value).strip().lower() in ("true", "yes", "1", "y", "req")


def _parse_format_name(value: any) -> str | None:
    """Parse a format name field value."""
    if pd.isna(value):
        return None
    format_val = str(value).strip()
    if not format_val or format_val.lower() in ("", "nan", "none"):
        return None
    return format_val


def _parse_content_length(value: any) -> int | None:
    """Parse a content length field value."""
    if pd.isna(value):
        return None
    try:
        return int(float(value))
    except (ValueError, TypeError):
        return None


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
        header_row = detect_header_row(df_raw)

        # Re-read with correct header
        df = pd.read_csv(path, header=header_row)
        df = normalize_column_names(df)

        # Find required columns
        id_col = find_column(df, _ITEMS_COLUMN_MAPPINGS["id"])
        if not id_col:
            raise MetadataLoadError(f"Could not find ID column in {path}")

        # Find optional columns
        label_col = find_column(df, _ITEMS_COLUMN_MAPPINGS["label"])
        dtype_col = find_column(df, _ITEMS_COLUMN_MAPPINGS["data_type"])
        mandatory_col = find_column(df, _ITEMS_COLUMN_MAPPINGS["mandatory"])
        format_col = find_column(df, _ITEMS_COLUMN_MAPPINGS["format_name"])
        length_col = find_column(df, _ITEMS_COLUMN_MAPPINGS["content_length"])

        items: dict[str, SourceColumn] = {}

        for _, row in df.iterrows():
            col_id = str(row.get(id_col, "")).strip()
            # Skip header rows or empty rows
            if not col_id or col_id.lower() in ("id", "columnid"):
                continue

            label = str(row.get(label_col, col_id)) if label_col else col_id
            data_type = (
                str(row.get(dtype_col, "text")).lower() if dtype_col else "text"
            )
            mandatory = (
                _parse_mandatory_field(row.get(mandatory_col)) if mandatory_col else False
            )
            format_name = (
                _parse_format_name(row.get(format_col)) if format_col else None
            )
            content_length = (
                _parse_content_length(row.get(length_col)) if length_col else None
            )

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
        header_row = detect_header_row(df_raw)

        # Re-read with correct header
        df = pd.read_csv(path, header=header_row)
        df = normalize_column_names(df)

        # Find required columns
        format_col = find_column(df, _CODELISTS_COLUMN_MAPPINGS["format_name"])
        value_col = find_column(df, _CODELISTS_COLUMN_MAPPINGS["code_value"])
        text_col = find_column(df, _CODELISTS_COLUMN_MAPPINGS["code_text"])

        if not format_col or not value_col or not text_col:
            raise MetadataLoadError(
                f"Could not find required columns in {path}. Found: {list(df.columns)}"
            )

        # Find optional columns
        dtype_col = find_column(df, _CODELISTS_COLUMN_MAPPINGS["data_type"])

        codelists: dict[str, CodeList] = {}

        for _, row in df.iterrows():
            format_name = str(row.get(format_col, "")).strip()
            # Skip header rows or empty rows
            if not format_name or format_name.lower() in ("formatname", "format name"):
                continue

            code_value = str(row.get(value_col, "")).strip()
            code_text = str(row.get(text_col, "")).strip()
            data_type = (
                str(row.get(dtype_col, "text")).lower() if dtype_col else "text"
            )

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
