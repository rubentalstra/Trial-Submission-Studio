"""Basic mapping engine for SDTM variable suggestion.

This module provides the MappingEngine class which suggests mappings
from source columns to SDTM target variables using fuzzy matching and
pattern recognition.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd
from rapidfuzz import fuzz

if TYPE_CHECKING:
    from ....io_module import Hints

from ....domains_module import get_domain, SDTMDomain, SDTMVariable
from ....domain.entities.mapping import ColumnMapping, MappingSuggestions, Suggestion
from .pattern_builder import build_variable_patterns
from .utils import normalize_text, safe_column_name


class MappingEngine:
    """Engine for suggesting SDTM variable mappings from source columns.

    This engine uses fuzzy matching and pattern recognition to suggest
    mappings between source data columns and SDTM target variables.

    Example:
        >>> engine = MappingEngine("DM", min_confidence=0.7)
        >>> suggestions = engine.suggest(source_df)
        >>> for mapping in suggestions.mappings:
        ...     print(f"{mapping.source_column} -> {mapping.target_variable}")
    """

    def __init__(
        self,
        domain_code: str,
        *,
        min_confidence: float = 0.5,
        column_hints: Hints | None = None,
    ) -> None:
        """Initialize the mapping engine.

        Args:
            domain_code: SDTM domain code (e.g., "DM", "AE")
            min_confidence: Minimum confidence score for suggestions (0.0-1.0)
            column_hints: Optional column hints for improved matching
        """
        self.domain: SDTMDomain = get_domain(domain_code)
        self.min_confidence = min_confidence
        self.column_hints: Hints = column_hints or {}
        self.valid_targets: set[str] = set(self.domain.variable_names())

        # Build dynamic patterns from domain metadata
        self._variable_patterns = build_variable_patterns(self.domain)

    def suggest(self, frame: pd.DataFrame) -> MappingSuggestions:
        """Suggest mappings for all columns in the DataFrame.

        Args:
            frame: Source DataFrame to analyze

        Returns:
            MappingSuggestions with suggested mappings and unmapped columns
        """
        suggestions: list[ColumnMapping] = []
        unmapped: list[str] = []
        assigned_targets: set[str] = set()

        # First pass: collect alias overrides
        column_details: list[tuple[str, str | None]] = [
            (column, self._alias_override(column)) for column in frame.columns
        ]

        alias_mappings: dict[str, ColumnMapping] = {}
        alias_collisions: set[str] = set()

        # Process aliases first (highest confidence)
        for column, alias_target in column_details:
            if not alias_target:
                continue
            if alias_target in assigned_targets:
                alias_collisions.add(column)
                continue
            assigned_targets.add(alias_target)
            alias_mappings[column] = ColumnMapping(
                source_column=safe_column_name(column),
                target_variable=alias_target,
                transformation=None,
                confidence_score=1.0,
            )

        # Second pass: process all columns
        for column, alias_target in column_details:
            # Use alias mapping if available
            if column in alias_mappings:
                suggestions.append(alias_mappings[column])
                continue

            # Skip if there was an alias collision
            if column in alias_collisions:
                unmapped.append(column)
                continue

            # Try fuzzy matching
            match = self._best_match(column)
            if match is None or match.confidence < self.min_confidence:
                unmapped.append(column)
                continue

            candidate = match.candidate
            confidence = match.confidence

            # Skip if target already assigned
            if candidate in assigned_targets:
                unmapped.append(column)
                continue

            assigned_targets.add(candidate)
            suggestions.append(
                ColumnMapping(
                    source_column=safe_column_name(column),
                    target_variable=candidate,
                    transformation=None,
                    confidence_score=confidence,
                )
            )

        return MappingSuggestions(mappings=suggestions, unmapped_columns=unmapped)

    def _best_match(self, column: str) -> Suggestion | None:
        """Find the best matching SDTM variable for a column.

        Args:
            column: Source column name

        Returns:
            Best matching suggestion or None if no good match found
        """
        normalized = normalize_text(column)
        best: Suggestion | None = None

        for variable in self.domain.variables:
            # Try both raw and normalized matching
            score_raw = fuzz.token_set_ratio(column.upper(), variable.name)
            score_norm = fuzz.ratio(normalized, variable.name)
            score = max(score_raw, score_norm) / 100

            # Apply hint-based adjustments
            score = self._apply_hints(column, variable, score)

            if not best or score > best.confidence:
                best = Suggestion(
                    column=column, candidate=variable.name, confidence=score
                )

        return best

    def _alias_override(self, column: str) -> str | None:
        """Check if column matches a known pattern from domain metadata.

        Args:
            column: Source column name

        Returns:
            Target SDTM variable name if pattern matches, None otherwise
        """
        normalized = normalize_text(column)

        # Check against dynamic patterns for each variable
        for target_var, patterns in self._variable_patterns.items():
            if target_var not in self.valid_targets:
                continue

            for pattern in patterns:
                if normalized == pattern:
                    return target_var

        return None

    def _apply_hints(self, column: str, variable: SDTMVariable, score: float) -> float:
        """Apply column hint adjustments to matching score.

        Args:
            column: Source column name
            variable: Target SDTM variable
            score: Initial matching score

        Returns:
            Adjusted score
        """
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
