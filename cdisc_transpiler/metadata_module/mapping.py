"""SDTM mapping utilities for inferring target variables and transformers."""

from __future__ import annotations

import re
from typing import Any, Callable

import pandas as pd

from .models import SourceColumn, StudyMetadata


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
    """Normalize a column name for comparison.

    Args:
        name: The column name to normalize

    Returns:
        Normalized column name (uppercase, alphanumeric only)
    """
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
) -> Callable | None:
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
