"""Column mapping engine and configuration.

This module provides:
- MappingEngine: Suggests SDTM variable mappings from source columns
- MappingConfig: Configuration model for column-to-variable mappings
- Utility functions for loading/saving mapping configurations
- MetadataAwareMapper: Uses Items.csv and CodeLists.csv for automatic mapping
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Iterable

import pandas as pd
from pydantic import BaseModel, Field
from rapidfuzz import fuzz

from .io import Hints
from .domains import SDTMDomain, get_domain

if TYPE_CHECKING:
    from .metadata import StudyMetadata


# =============================================================================
# Configuration Models
# =============================================================================


class ColumnMapping(BaseModel):
    """Mapping from a source column to an SDTM target variable."""

    source_column: str
    target_variable: str
    transformation: str | None = None
    confidence_score: float = Field(ge=0.0, le=1.0)
    # New fields for metadata-driven mappings
    codelist_name: str | None = None  # Name of codelist to apply
    use_code_column: str | None = None  # Column containing coded values

    def to_assignment(self) -> str:
        """Return SAS assignment snippet for the mapping."""
        expr = self.transformation or self.source_column
        return f"{self.target_variable} = {expr};"


class MappingConfig(BaseModel):
    """Configuration for mapping source data to an SDTM domain."""

    domain: str
    study_id: str | None = None
    mappings: list[ColumnMapping]

    @property
    def target_variables(self) -> set[str]:
        return {m.target_variable for m in self.mappings}

    def enforce_domain(self) -> None:
        get_domain(self.domain)  # raises if invalid

    def missing_required(self) -> set[str]:
        domain = get_domain(self.domain)
        required = {
            var.name
            for var in domain.variables
            if (var.core or "").strip().lower() == "req"
        }
        auto_populated = {"STUDYID", "DOMAIN"}
        return (required - auto_populated) - self.target_variables

    def validate_required(self) -> None:
        missing = self.missing_required()
        if missing:
            raise ValueError(
                f"Missing required variables for domain {self.domain}: {sorted(missing)}"
            )


def load_config(path: str | Path) -> MappingConfig:
    """Load a MappingConfig from a JSON file."""
    file_path = Path(path)
    with file_path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    config = MappingConfig.model_validate(data)
    config.enforce_domain()
    return config


def save_config(config: MappingConfig, path: str | Path) -> None:
    """Save a MappingConfig to a JSON file."""
    file_path = Path(path)
    file_path.parent.mkdir(parents=True, exist_ok=True)
    payload = config.model_dump()
    with file_path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2)


def merge_mappings(
    base: MappingConfig, extra: Iterable[ColumnMapping]
) -> MappingConfig:
    """Merge additional mappings into an existing config."""
    existing = {m.target_variable: m for m in base.mappings}
    for mapping in extra:
        existing.setdefault(mapping.target_variable, mapping)
    base.mappings = list(existing.values())
    return base


def build_config(domain_code: str, mappings: Iterable[ColumnMapping]) -> MappingConfig:
    """Build a MappingConfig from a list of column mappings."""
    config = MappingConfig(domain=domain_code, mappings=list(mappings))
    config.enforce_domain()
    return config


@dataclass
class Suggestion:
    column: str
    candidate: str
    confidence: float


@dataclass
class MappingSuggestions:
    mappings: list[ColumnMapping]
    unmapped_columns: list[str]


def _normalize(text: str) -> str:
    return re.sub(r"[^A-Z0-9]", "", text.upper())


class MappingEngine:
    def __init__(
        self,
        domain_code: str,
        *,
        min_confidence: float = 0.5,
        column_hints: Hints | None = None,
    ) -> None:
        self.domain: SDTMDomain = get_domain(domain_code)
        self.min_confidence = min_confidence
        self.column_hints: Hints = column_hints or {}
        self.valid_targets: set[str] = set(self.domain.variable_names())

    def suggest(self, frame: pd.DataFrame) -> MappingSuggestions:
        suggestions: list[ColumnMapping] = []
        unmapped: list[str] = []
        assigned_targets: set[str] = set()
        column_details: list[tuple[str, str | None]] = [
            (column, self._alias_override(column)) for column in frame.columns
        ]

        alias_mappings: dict[str, ColumnMapping] = {}
        alias_collisions: set[str] = set()

        for column, alias_target in column_details:
            if not alias_target:
                continue
            if alias_target in assigned_targets:
                alias_collisions.add(column)
                continue
            assigned_targets.add(alias_target)
            alias_mappings[column] = ColumnMapping(
                source_column=self._safe_column(column),
                target_variable=alias_target,
                transformation=None,
                confidence_score=1.0,
            )

        for column, alias_target in column_details:
            if column in alias_mappings:
                suggestions.append(alias_mappings[column])
                continue
            if column in alias_collisions:
                unmapped.append(column)
                continue
            match = self._best_match(column)
            if match is None or match.confidence < self.min_confidence:
                unmapped.append(column)
                continue
            candidate = match.candidate
            confidence = match.confidence
            if candidate in assigned_targets:
                unmapped.append(column)
                continue
            assigned_targets.add(candidate)
            suggestions.append(
                ColumnMapping(
                    source_column=self._safe_column(column),
                    target_variable=candidate,
                    transformation=None,
                    confidence_score=confidence,
                )
            )
        return MappingSuggestions(mappings=suggestions, unmapped_columns=unmapped)

    def _best_match(self, column: str) -> Suggestion | None:
        normalized = _normalize(column)
        best: Suggestion | None = None
        for variable in self.domain.variables:
            score_raw = fuzz.token_set_ratio(column.upper(), variable.name)
            score_norm = fuzz.ratio(normalized, variable.name)
            score = max(score_raw, score_norm) / 100
            score = self._apply_hints(column, variable, score)
            if not best or score > best.confidence:
                best = Suggestion(
                    column=column, candidate=variable.name, confidence=score
                )
        return best

    def _alias_override(self, column: str) -> str | None:
        """Check if column matches a known pattern from _SDTM_INFERENCE_PATTERNS."""
        normalized = _normalize(column)

        # Check global patterns
        for target, sources in _SDTM_INFERENCE_PATTERNS.get("_GLOBAL", {}).items():
            if normalized in [_normalize(s) for s in sources]:
                if target in self.valid_targets:
                    return target

        # Check domain-specific suffix patterns
        domain_code = self.domain.code.upper()
        for suffix, sources in _SDTM_INFERENCE_PATTERNS.get(
            "_DOMAIN_SUFFIXES", {}
        ).items():
            target = domain_code + suffix
            # Check with domain prefix
            if normalized in [_normalize(domain_code + s) for s in sources]:
                if target in self.valid_targets:
                    return target
            # Check suffix patterns directly
            if normalized in [_normalize(s) for s in sources]:
                if target in self.valid_targets:
                    return target

        return None

    def _apply_hints(self, column: str, variable, score: float) -> float:
        hint = self.column_hints.get(column)
        if not hint:
            return score
        adjusted = score
        variable_is_numeric = variable.type.lower() == "num"
        if variable_is_numeric != hint.is_numeric:
            adjusted *= 0.85
        if variable.name.endswith("SEQ") and hint.unique_ratio < 0.5:
            adjusted *= 0.9
        if hint.null_ratio > 0.5 and (variable.core or "").strip().lower() == "req":
            adjusted *= 0.9
        return adjusted

    @staticmethod
    def _safe_column(column: str) -> str:
        if re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", column):
            return column
        escaped = column.replace('"', '""')
        return f'"{escaped}"n'


# =============================================================================
# Metadata-Aware Mapping Engine
# =============================================================================


# Patterns for inferring SDTM variable names from source column names
# These patterns map normalized source column names to SDTM variables
_SDTM_INFERENCE_PATTERNS: dict[str, dict[str, list[str]]] = {
    # Global patterns (apply to all domains)
    "_GLOBAL": {
        "USUBJID": [
            "SUBJECTID",
            "SUBJECTIDENTIFIER",
            "PATIENTID",
            "SUBJECT",
            "PATIENTIDENTIFIER",
        ],
        "SEX": ["SEX", "GENDER"],
        "AGE": ["AGE"],
        "AGEU": ["AGEU", "AGEUNIT", "AGEUNITS"],
        "RACE": ["RACE"],
        "ETHNIC": ["ETHNIC", "ETHNICITY"],
        "RFSTDTC": ["ICDAT", "INFORMEDCONSENTDATE", "RFSTDTC"],
        "BRTHDTC": ["BRTHDTC", "BIRTHDATE", "DOB"],
        "COUNTRY": ["COUNTRY", "COUNTRYCD"],
        "SITEID": ["SITEID", "SITECODE", "SITE"],
        "EPOCH": ["EPOCH", "VISITEPOCH"],
        "TAETORD": ["TAETORD", "ELEMENTORDER", "PLANNEDORDER"],
        "VISITNUM": ["VISITNUM", "VISITNUMBER"],
        "VISIT": ["VISIT", "VISITNAME", "EVENTNAME"],
        "VISITDY": ["VISITDY", "PLANNEDSTUDYDAYOFVISIT"],
        "SPDEVID": ["SPDEVID", "DEVICEID", "DEVICEIDENTIFIER"],
    },
    # Domain-specific patterns (--VARIABLE becomes DOMAIN + VARIABLE)
    "_DOMAIN_SUFFIXES": {
        "TERM": ["TERM", "REPORTEDTERM", "VERBATIM"],
        "DECOD": ["DECOD", "DECODE", "DICTIONARYTERM", "STANDARDTERM"],
        "CAT": ["CAT", "CATEGORY"],
        "SCAT": ["SCAT", "SUBCATEGORY"],
        "ORRES": ["ORRES", "RESULT", "ORIGINALRESULT", "VALUE"],
        "ORRESU": ["ORRESU", "UNIT", "UNITS", "ORIGINALUNIT"],
        "STRESC": ["STRESC", "STANDARDRESULT", "STANDARDIZEDRESULT"],
        "STRESN": ["STRESN", "NUMERICRESULT", "NUMERICVALUE"],
        "STRESU": ["STRESU", "STANDARDUNIT"],
        "SEQ": ["SEQ", "EVENTSEQ", "SEQUENCENUMBER", "EVENTSEQUENCENUMBER"],
        "GRPID": ["GRPID", "GROUPID", "GROUP"],
        "REFID": ["REFID", "REFERENCEID", "REFIDENTIFIER"],
        "SPID": ["SPID", "SPONSORID", "SPONSORIDENTIFIER"],
        "LNKID": ["LNKID", "LINKID", "LINKIDENTIFIER", "LINK"],
        "LNKGRP": ["LNKGRP", "LINKGROUP", "LINKGRP"],
        "STDTC": ["STDTC", "STDAT", "STARTDATE", "STARTDATETIME"],
        "ENDTC": ["ENDTC", "ENDAT", "ENDDATE", "ENDDATETIME"],
        "DTC": ["DTC", "DAT", "DATE", "DATETIME"],
        "RFTDTC": ["RFTDTC", "REFERENCEDTC", "REFERENCETIMEPOINT"],
        "DY": ["DY", "STUDYDAY"],
        "STDY": ["STDY", "STARTDY", "STUDYDAYSTART"],
        "ENDY": ["ENDY", "ENDDY", "STUDYDAYEND"],
        "DUR": ["DUR", "DURATION"],
        "ELTM": ["ELTM", "ELAPSEDTIME", "ELAPSED"],
        "TPT": ["TPT", "TIMEPOINT", "PLANNEDTIMEPOINT"],
        "TPTNUM": ["TPTNUM", "TIMEPOINTNUM", "TPTNUMBER"],
        "TPTREF": ["TPTREF", "TIMEPOINTREF", "REFERENCEPOINT"],
        "STRTPT": ["STRTPT", "STARTTPT", "STARTREFERENCE"],
        "STTPT": ["STTPT", "STARTTP", "STARTTIMEPOINT"],
        "ENRTPT": ["ENRTPT", "ENDRTPT", "ENDREFERENCEPOINT"],
        "ENTPT": ["ENTPT", "ENDTPT", "ENDTIMEPOINT"],
        "ENRF": ["ENRF", "ENDREF", "ENDREFERENCE"],
        "STRF": ["STRF", "STARTREF", "STARTREFERENCE"],
        "STAT": ["STAT", "STATUS", "COMPLETIONSTATUS"],
        "REASND": ["REASND", "REASONNOTDONE", "REASON"],
        "TEST": ["TEST", "TESTNAME"],
        "TESTCD": ["TESTCD", "TESTCODE"],
        "POS": ["POS", "POSITION"],
        "PERF": ["PERF", "PERFORMED"],
        "BODSYS": ["BODSYS", "BODYSYSTEM", "ORGANCLASS"],
        "LOC": ["LOC", "LOCATION", "SITE"],
        "LAT": ["LAT", "LATERALITY"],
        "DIR": ["DIR", "DIRECTION"],
        "METHOD": ["METHOD", "COLLECTIONMETHOD"],
        "SPEC": ["SPEC", "SPECIMEN", "SPECIMENTYPE"],
        "TRT": ["TRT", "TREATMENT", "MEDICATION", "DRUG"],
        "DOSE": ["DOSE", "DOSEAMOUNT"],
        "DOSU": ["DOSU", "DOSEUNIT", "DOSEUNITS"],
        "DOSFRM": ["DOSFRM", "DOSEFORM"],
        "DOSFRQ": ["DOSFRQ", "DOSINGFREQUENCY", "FREQUENCY"],
        "ROUTE": ["ROUTE", "ADMINISTRATIONROUTE"],
        "SER": ["SER", "SERIOUS", "SERIOUSEVENT"],
        "SEV": ["SEV", "SEVERITY", "INTENSITY"],
        "REL": ["REL", "RELATIONSHIP", "CAUSALITY"],
        "OUT": ["OUT", "OUTCOME"],
        "ACN": ["ACN", "ACTION", "ACTIONTAKEN"],
    },
}


class MetadataAwareMapper:
    """Mapping engine that uses Items.csv and CodeLists.csv metadata.

    This mapper provides automatic mapping from source columns to SDTM variables
    by analyzing the source metadata and applying intelligent mapping rules.
    """

    def __init__(
        self,
        domain_code: str,
        metadata: "StudyMetadata | None" = None,
        *,
        min_confidence: float = 0.5,
        column_hints: Hints | None = None,
    ) -> None:
        """Initialize the metadata-aware mapper.

        Args:
            domain_code: Target SDTM domain code
            metadata: Optional StudyMetadata with Items.csv and CodeLists.csv data
            min_confidence: Minimum confidence threshold for fuzzy matches
            column_hints: Optional column hints from source data analysis
        """
        self.domain: SDTMDomain = get_domain(domain_code)
        self.domain_code = domain_code.upper()
        self.metadata = metadata
        self.min_confidence = min_confidence
        self.column_hints: Hints = column_hints or {}
        self.valid_targets: set[str] = set(self.domain.variable_names())

        # Build combined alias dictionary for this domain
        self._build_alias_dictionary()

    def _build_alias_dictionary(self) -> None:
        """Build the alias dictionary from metadata and static patterns."""
        self._aliases: dict[str, str] = {}

        # Start with global patterns
        for target, sources in _SDTM_INFERENCE_PATTERNS.get("_GLOBAL", {}).items():
            for src in sources:
                normalized = _normalize(src)
                if target in self.valid_targets:
                    self._aliases[normalized] = target

        # Add domain-specific suffix patterns
        for suffix, sources in _SDTM_INFERENCE_PATTERNS.get(
            "_DOMAIN_SUFFIXES", {}
        ).items():
            target = self.domain_code + suffix
            if target in self.valid_targets:
                for src in sources:
                    # Add with domain prefix
                    self._aliases[_normalize(self.domain_code + src)] = target
                    # Add suffix-only pattern for common cases
                    self._aliases[_normalize(src)] = target

        # Add metadata-driven mappings (highest priority)
        if self.metadata and self.metadata.items:
            self._add_metadata_aliases()

    def _add_metadata_aliases(self) -> None:
        """Add aliases from Items.csv metadata."""
        if not self.metadata:
            return

        for col_id, item in self.metadata.items.items():
            normalized = _normalize(col_id)

            # Check if column ID matches or starts with domain code
            if col_id.startswith(self.domain_code):
                # Direct SDTM variable match
                if col_id in self.valid_targets:
                    self._aliases[normalized] = col_id
                    continue

            # Check if it's a known suffix pattern
            for suffix in _SDTM_INFERENCE_PATTERNS.get("_DOMAIN_SUFFIXES", {}).keys():
                if col_id.endswith(suffix):
                    target = self.domain_code + suffix
                    if target in self.valid_targets:
                        self._aliases[normalized] = target
                        break

            # Try to infer from label
            if item.label:
                label_normalized = _normalize(item.label)
                for target in self.valid_targets:
                    if (
                        target in label_normalized
                        or _normalize(target) == label_normalized
                    ):
                        self._aliases[normalized] = target
                        break

    def suggest(self, frame: pd.DataFrame) -> MappingSuggestions:
        """Generate mapping suggestions for the DataFrame.

        This method analyzes the source columns and suggests SDTM mappings
        using a priority order:
        1. Metadata-driven exact matches
        2. Static alias dictionary matches
        3. Fuzzy matching with confidence threshold

        Args:
            frame: Source DataFrame to map

        Returns:
            MappingSuggestions with mappings and unmapped columns
        """
        suggestions: list[ColumnMapping] = []
        unmapped: list[str] = []
        assigned_targets: set[str] = set()

        # First pass: identify alias-based mappings
        column_aliases: dict[str, tuple[str, str | None, str | None]] = {}

        for column in frame.columns:
            normalized = _normalize(column)
            target = self._aliases.get(normalized)

            # Check for codelist association
            codelist_name = None
            code_column = None

            if self.metadata:
                # Check if there's a corresponding code column
                if column.endswith("CD"):
                    # This is a code column, try to find its text column
                    text_col = column[:-2]
                    if text_col in frame.columns:
                        continue  # Skip code columns, we'll use text columns
                else:
                    # Check if there's a code column for this text column
                    code_col = column + "CD"
                    if code_col in frame.columns:
                        code_column = code_col
                        col_def = self.metadata.get_column(code_col)
                        if col_def and col_def.format_name:
                            codelist_name = col_def.format_name
                    else:
                        # Check the column itself for a codelist
                        col_def = self.metadata.get_column(column)
                        if col_def and col_def.format_name:
                            codelist_name = col_def.format_name

            if target:
                column_aliases[column] = (target, codelist_name, code_column)

        # Process alias mappings first
        for column, (target, codelist_name, code_column) in column_aliases.items():
            if target in assigned_targets:
                unmapped.append(column)
                continue

            assigned_targets.add(target)
            suggestions.append(
                ColumnMapping(
                    source_column=self._safe_column(column),
                    target_variable=target,
                    transformation=None,
                    confidence_score=1.0,
                    codelist_name=codelist_name,
                    use_code_column=code_column,
                )
            )

        # Second pass: fuzzy matching for remaining columns
        for column in frame.columns:
            if column in column_aliases:
                continue  # Already processed

            # Skip code columns
            if column.endswith("CD"):
                base_col = column[:-2]
                if base_col in frame.columns:
                    continue

            match = self._best_fuzzy_match(column)
            if match is None or match.confidence < self.min_confidence:
                unmapped.append(column)
                continue

            if match.candidate in assigned_targets:
                unmapped.append(column)
                continue

            assigned_targets.add(match.candidate)

            # Check for codelist
            codelist_name = None
            if self.metadata:
                col_def = self.metadata.get_column(column)
                if col_def and col_def.format_name:
                    codelist_name = col_def.format_name

            suggestions.append(
                ColumnMapping(
                    source_column=self._safe_column(column),
                    target_variable=match.candidate,
                    transformation=None,
                    confidence_score=match.confidence,
                    codelist_name=codelist_name,
                )
            )

        return MappingSuggestions(mappings=suggestions, unmapped_columns=unmapped)

    def _best_fuzzy_match(self, column: str) -> Suggestion | None:
        """Find the best fuzzy match for a column."""
        normalized = _normalize(column)
        best: Suggestion | None = None

        for variable in self.domain.variables:
            # Calculate similarity scores
            score_raw = fuzz.token_set_ratio(column.upper(), variable.name)
            score_norm = fuzz.ratio(normalized, variable.name)
            score = max(score_raw, score_norm) / 100

            # Apply hints if available
            score = self._apply_hints(column, variable, score)

            if not best or score > best.confidence:
                best = Suggestion(
                    column=column, candidate=variable.name, confidence=score
                )

        return best

    def _apply_hints(self, column: str, variable, score: float) -> float:
        """Apply column hints to adjust the confidence score."""
        hint = self.column_hints.get(column)
        if not hint:
            return score

        adjusted = score
        variable_is_numeric = variable.type.lower() == "num"

        # Penalize type mismatches
        if variable_is_numeric != hint.is_numeric:
            adjusted *= 0.85

        # SEQ variables should have high uniqueness
        if variable.name.endswith("SEQ") and hint.unique_ratio < 0.5:
            adjusted *= 0.9

        # Required variables shouldn't have high null ratio
        if hint.null_ratio > 0.5 and (variable.core or "").strip().lower() == "req":
            adjusted *= 0.9

        return adjusted

    @staticmethod
    def _safe_column(column: str) -> str:
        """Make column name safe for SAS."""
        if re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", column):
            return column
        escaped = column.replace('"', '""')
        return f'"{escaped}"n'


def create_mapper(
    domain_code: str,
    metadata: "StudyMetadata | None" = None,
    *,
    min_confidence: float = 0.5,
    column_hints: Hints | None = None,
) -> MetadataAwareMapper | MappingEngine:
    """Factory function to create the appropriate mapper.

    If metadata is provided, returns a MetadataAwareMapper that uses
    Items.csv and CodeLists.csv for intelligent mapping.
    Otherwise, returns the standard MappingEngine.

    Args:
        domain_code: Target SDTM domain code
        metadata: Optional StudyMetadata
        min_confidence: Minimum confidence threshold
        column_hints: Optional column hints

    Returns:
        Appropriate mapper instance
    """
    if metadata is not None and (metadata.items or metadata.codelists):
        return MetadataAwareMapper(
            domain_code,
            metadata,
            min_confidence=min_confidence,
            column_hints=column_hints,
        )
    return MappingEngine(
        domain_code,
        min_confidence=min_confidence,
        column_hints=column_hints,
    )
