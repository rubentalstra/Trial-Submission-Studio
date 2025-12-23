"""Controlled terminology domain entity.

This is a pure data model representing CDISC controlled terminology.

It is used by ports and repositories so the application layer does not
depend on infrastructure adapters or legacy wrappers.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from difflib import get_close_matches
import re


@dataclass(frozen=True)
class ControlledTerminology:
    """Represents controlled terminology using column-aligned names from CT CSVs."""

    codelist_code: str | None
    codelist_name: str
    submission_values: set[str]
    codelist_extensible: bool = False
    synonyms: dict[str, str] | None = None
    # Maps canonical CDISC Submission Value -> tuple of its CDISC Synonym(s) (as seen in CT files).
    submission_value_synonyms: dict[str, tuple[str, ...]] = field(default_factory=dict)
    nci_codes: dict[str, str] = field(default_factory=dict)
    standards: set[str] = field(default_factory=set)
    sources: set[str] = field(default_factory=set)
    definitions: dict[str, str] = field(default_factory=dict)
    preferred_terms: dict[str, str] = field(default_factory=dict)
    variable: str | None = None

    def _canonicalize_for_match(self, raw: str) -> str:
        text = raw.strip()
        if not text:
            return ""
        text = re.sub(r"\s*/\s*", "/", text)
        text = re.sub(r"\s+", " ", text)
        return text

    def _looks_like_code_value(self, text: str) -> bool:
        """Heuristic: code-like values (e.g., test codes) should not get fuzzy suggestions."""
        if not text:
            return False
        if " " in text:
            return False
        # Typical code values are short and mostly A–Z/0–9 with a few separators.
        if len(text) > 12:
            return False
        return bool(re.fullmatch(r"[A-Z0-9_./-]+", text.upper()))

    def synonyms_for_submission_value(self, submission_value: str) -> tuple[str, ...]:
        if not submission_value:
            return ()
        return self.submission_value_synonyms.get(submission_value, ())

    def format_submission_value_with_synonyms(
        self, submission_value: str, *, max_synonyms: int = 3
    ) -> str:
        synonyms = list(self.synonyms_for_submission_value(submission_value))
        if not synonyms:
            return submission_value
        shown = synonyms[:max_synonyms]
        suffix = (
            ""
            if len(synonyms) <= max_synonyms
            else f"; +{len(synonyms) - max_synonyms}"
        )
        joined = "; ".join(shown) + suffix
        return f"{submission_value} (synonyms: {joined})"

    def suggest_submission_values(
        self, raw_value: object, *, limit: int = 3
    ) -> list[str]:
        """Suggest canonical CDISC Submission Value(s) for a raw value.

        Uses CT content (Submission Value + Synonym(s) + Preferred Term) to suggest
        the most likely canonical Submission Value.
        """
        if raw_value is None:
            return []
        raw_text = str(raw_value).strip()
        if not raw_text:
            return []

        query_text = self._canonicalize_for_match(raw_text)
        if not query_text:
            return []

        query = query_text.upper()

        candidate_to_canonical: dict[str, str] = {}

        for submission in self.submission_values:
            key = self._canonicalize_for_match(submission).upper()
            if key:
                candidate_to_canonical[key] = submission

        if self.synonyms:
            for syn_upper, canonical in self.synonyms.items():
                key = self._canonicalize_for_match(syn_upper).upper()
                if key:
                    candidate_to_canonical.setdefault(key, canonical)

        for canonical, preferred in self.preferred_terms.items():
            pref = self._canonicalize_for_match(preferred)
            if pref:
                candidate_to_canonical.setdefault(pref.upper(), canonical)

        # High-confidence: exact match (case-insensitive) against Submission Value, Synonym(s), or Preferred Term.
        exact = candidate_to_canonical.get(query)
        if exact:
            return [exact]

        # Conservative: avoid fuzzy suggestions for short / code-like values (e.g., TESTCD variables).
        if self._looks_like_code_value(query_text) or len(query_text) < 6:
            return []

        candidates = list(candidate_to_canonical.keys())
        if not candidates:
            return []

        # Low-confidence fuzzy match (used only for longer, non-code-like free-text).
        matches = get_close_matches(query, candidates, n=limit * 3, cutoff=0.85)
        suggestions: list[str] = []
        seen: set[str] = set()
        for match in matches:
            canonical = candidate_to_canonical.get(match)
            if not canonical or canonical in seen:
                continue
            seen.add(canonical)
            suggestions.append(canonical)
            if len(suggestions) >= limit:
                break
        return suggestions

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
