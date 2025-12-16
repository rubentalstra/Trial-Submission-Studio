"""Controlled terminology domain entity.

This is a pure data model representing CDISC controlled terminology.

It is used by ports and repositories so the application layer does not
depend on infrastructure adapters or legacy wrappers.
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class ControlledTerminology:
    """Represents controlled terminology using column-aligned names from CT CSVs."""

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
        if not value:
            return None
        return self.nci_codes.get(value) or self.nci_codes.get(value.upper())

    def invalid_values(self, series: object) -> set[str]:
        invalid: set[str] = set()
        from ...pandas_utils import ensure_series

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
