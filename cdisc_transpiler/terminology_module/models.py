"""Data models for controlled terminology.

This module contains dataclasses for representing CDISC
controlled terminology codelists and values.
"""

from __future__ import annotations

from dataclasses import dataclass, field

import pandas as pd


@dataclass(frozen=True)
class ControlledTerminology:
    """Represents controlled terminology using column-aligned names from CT CSVs.

    Attributes:
        codelist_code: NCI code for the codelist (e.g., "C66767")
        codelist_name: Human-readable name of the codelist
        submission_values: Set of valid submission values
        codelist_extensible: Whether the codelist allows extended values
        synonyms: Mapping of uppercase synonyms to canonical values
        nci_codes: Mapping of values to NCI codes (C-codes)
        standards: Set of standard names (e.g., "SDTM_CT_2025-09-26")
        sources: Set of source CSV filenames
        definitions: Mapping of values to CDISC definitions
        preferred_terms: Mapping of values to NCI preferred terms
        variable: Optional mapping back to variable name
    """

    codelist_code: str | None
    codelist_name: str
    submission_values: set[str]
    codelist_extensible: bool = False
    synonyms: dict[str, str] | None = None
    nci_codes: dict[str, str] = field(default_factory=dict)
    standards: set[str] = field(default_factory=set)
    sources: set[str] = field(default_factory=set)
    definitions: dict[str, str] = field(default_factory=dict)
    preferred_terms: dict[str, str] = field(default_factory=dict)
    variable: str | None = None

    def normalize(self, raw_value: object) -> str:
        """Normalize a raw input value to canonical form preserving CDISC case.

        Args:
            raw_value: Value to normalize

        Returns:
            Normalized canonical value or empty string if invalid
        """
        if raw_value is None:
            return ""
        text = str(raw_value).strip()
        if not text:
            return ""
        lookup_key = text.upper()
        if self.synonyms:
            canonical = self.synonyms.get(lookup_key)
            if canonical is not None:
                return canonical
        return text

    def lookup_code(self, value: str) -> str | None:
        """Look up the NCI C-code for a submission value.

        NCI codes are external identifiers used in CDISC controlled terminology
        (e.g., C49488 for "Y", C20197 for "M").

        Args:
            value: The submission value to look up

        Returns:
            NCI C-code or None if not found
        """
        if not value:
            return None
        return self.nci_codes.get(value) or self.nci_codes.get(value.upper())

    def invalid_values(self, series: object) -> set[str]:
        """Return invalid raw values given the canonical CT list.

        Args:
            series: Pandas series of values to validate

        Returns:
            Set of invalid values not in the controlled terminology
        """
        invalid: set[str] = set()
        from ..pandas_utils import ensure_series

        series_values = ensure_series(series)

        for raw_value in series_values.dropna().unique():
            normalized = self.normalize(raw_value)
            if not normalized:
                continue
            if normalized in self.submission_values:
                continue
            if self.codelist_extensible:
                continue
            invalid.add(str(raw_value))
        return invalid
