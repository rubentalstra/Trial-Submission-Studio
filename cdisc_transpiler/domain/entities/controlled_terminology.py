from dataclasses import dataclass, field
from difflib import get_close_matches
import re

from ...pandas_utils import ensure_series

CODELIKE_MAX_LENGTH = 12
FUZZY_SUGGEST_MIN_LENGTH = 6


def _empty_submission_value_synonyms() -> dict[str, tuple[str, ...]]:
    return {}


def _empty_nci_codes() -> dict[str, str]:
    return {}


def _empty_standards() -> set[str]:
    return set()


def _empty_sources() -> set[str]:
    return set()


def _empty_definitions() -> dict[str, str]:
    return {}


def _empty_preferred_terms() -> dict[str, str]:
    return {}


@dataclass(frozen=True, slots=True)
class ControlledTerminology:
    codelist_code: str | None
    codelist_name: str
    submission_values: set[str]
    codelist_extensible: bool = False
    synonyms: dict[str, str] | None = None
    submission_value_synonyms: dict[str, tuple[str, ...]] = field(
        default_factory=_empty_submission_value_synonyms
    )
    nci_codes: dict[str, str] = field(default_factory=_empty_nci_codes)
    standards: set[str] = field(default_factory=_empty_standards)
    sources: set[str] = field(default_factory=_empty_sources)
    definitions: dict[str, str] = field(default_factory=_empty_definitions)
    preferred_terms: dict[str, str] = field(default_factory=_empty_preferred_terms)
    variable: str | None = None

    def _canonicalize_for_match(self, raw: str) -> str:
        text = raw.strip()
        if not text:
            return ""
        text = re.sub("\\s*/\\s*", "/", text)
        return re.sub("\\s+", " ", text)

    def _looks_like_code_value(self, text: str) -> bool:
        if not text:
            return False
        if " " in text:
            return False
        if len(text) > CODELIKE_MAX_LENGTH:
            return False
        return bool(re.fullmatch("[A-Z0-9_./-]+", text.upper()))

    def _normalize_query_text(self, raw_value: object) -> str | None:
        if raw_value is None:
            return None
        raw_text = str(raw_value).strip()
        if not raw_text:
            return None
        query_text = self._canonicalize_for_match(raw_text)
        return query_text or None

    def _build_candidate_index(self) -> dict[str, str]:
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
        return candidate_to_canonical

    def _suggest_from_candidates(
        self, query_text: str, candidate_to_canonical: dict[str, str], *, limit: int
    ) -> list[str]:
        query = query_text.upper()
        exact = candidate_to_canonical.get(query)
        if exact:
            return [exact]
        if (
            self._looks_like_code_value(query_text)
            or len(query_text) < FUZZY_SUGGEST_MIN_LENGTH
        ):
            return []
        if not candidate_to_canonical:
            return []
        candidates = list(candidate_to_canonical.keys())
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
        query_text = self._normalize_query_text(raw_value)
        if query_text is None:
            return []
        candidate_to_canonical = self._build_candidate_index()
        return self._suggest_from_candidates(
            query_text, candidate_to_canonical, limit=limit
        )

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
