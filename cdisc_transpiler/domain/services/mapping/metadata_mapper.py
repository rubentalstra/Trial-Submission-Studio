"""Metadata-aware mapping engine for intelligent SDTM variable suggestion.

This module provides the MetadataAwareMapper class which uses Items.csv and
CodeLists.csv metadata to provide more accurate mapping suggestions.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd
from rapidfuzz import fuzz

if TYPE_CHECKING:
    from ...entities.column_hints import Hints
    from ....domain.entities.study_metadata import StudyMetadata

from ....domain.entities.mapping import ColumnMapping, MappingSuggestions, Suggestion
from ...entities.sdtm_domain import SDTMDomain, SDTMVariable
from .pattern_builder import build_variable_patterns
from .utils import normalize_text, safe_column_name


class MetadataAwareMapper:
    """Mapping engine that uses Items.csv and CodeLists.csv metadata.

    This mapper provides automatic mapping from source columns to SDTM variables
    by analyzing the source metadata and applying intelligent mapping rules.

    The mapper uses a priority order for suggestions:
    1. Metadata-driven exact matches
    2. Static alias dictionary matches
    3. Fuzzy matching with confidence threshold

    Example:
        >>> # Provide `StudyMetadata` from the application/infrastructure layer.
        >>> metadata = None
        >>> mapper = MetadataAwareMapper("DM", metadata, min_confidence=0.7)
        >>> suggestions = mapper.suggest(source_df)
    """

    def __init__(
        self,
        domain: SDTMDomain,
        metadata: StudyMetadata | None = None,
        *,
        min_confidence: float = 0.5,
        column_hints: Hints | None = None,
    ) -> None:
        """Initialize the metadata-aware mapper.

        Args:
            domain: Target SDTM domain definition
            metadata: Optional StudyMetadata with Items.csv and CodeLists.csv data
            min_confidence: Minimum confidence threshold for fuzzy matches
            column_hints: Optional column hints from source data analysis
        """
        self.domain = domain
        self.domain_code = domain.code.upper()
        self.metadata = metadata
        self.min_confidence = min_confidence
        self.column_hints: Hints = column_hints or {}
        self.valid_targets: set[str] = set(self.domain.variable_names())

        # Build dynamic patterns from domain metadata
        self._variable_patterns = build_variable_patterns(self.domain)

        # Build combined alias dictionary for this domain
        self._build_alias_dictionary()

    def _build_alias_dictionary(self) -> None:
        """Build the alias dictionary from metadata and dynamic patterns."""
        self._aliases: dict[str, str] = {}

        # Add dynamic patterns from domain variables
        for target_var, patterns in self._variable_patterns.items():
            if target_var not in self.valid_targets:
                continue
            for pattern in patterns:
                # Only add if not already mapped (first match wins)
                if pattern not in self._aliases:
                    self._aliases[pattern] = target_var

        # Add metadata-driven mappings (highest priority - overwrite if needed)
        if self.metadata and self.metadata.items:
            self._add_metadata_aliases()

    def _add_metadata_aliases(self) -> None:
        """Add aliases from Items.csv metadata."""
        if not self.metadata:
            return

        for col_id, item in self.metadata.items.items():
            normalized = normalize_text(col_id)

            # Check if column ID matches or starts with domain code
            if col_id.startswith(self.domain_code):
                # Direct SDTM variable match
                if col_id in self.valid_targets:
                    self._aliases[normalized] = col_id
                    continue

            # Check if it matches any variable pattern
            for target_var, patterns in self._variable_patterns.items():
                if target_var not in self.valid_targets:
                    continue
                if normalized in patterns:
                    self._aliases[normalized] = target_var
                    break

            # Try to infer from label
            if item.label:
                label_normalized = normalize_text(item.label)
                for target in self.valid_targets:
                    if (
                        target in label_normalized
                        or normalize_text(target) == label_normalized
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

        # First pass: identify alias-based mappings.
        # When multiple source columns map to the same SDTM target, pick the
        # best candidate (instead of letting DataFrame column order decide).
        alias_candidates_by_target: dict[
            str, list[tuple[str, str | None, str | None]]
        ] = {}

        for column in frame.columns:
            normalized = normalize_text(column)
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
                alias_candidates_by_target.setdefault(target, []).append(
                    (column, codelist_name, code_column)
                )

        def _alias_priority(target: str | None, column: str) -> int:
            """Return a higher number for better alias candidates."""
            t = (target or "").strip().upper()
            col_norm = normalize_text(column)

            # Domain-specific heuristic: DSSTDTC should prefer actual
            # disposition/contact/discontinuation dates over generic event dates.
            if self.domain_code == "DS" and t == "DSSTDTC":
                score = 0
                if (
                    "LASTCONTACT" in col_norm
                    or "LAST" in col_norm
                    and "CONTACT" in col_norm
                ):
                    score += 30
                if "EARLYDISCONTINUATION" in col_norm or "DISCONTINUATION" in col_norm:
                    score += 20
                if "WITHDRAW" in col_norm or "WITHDRAWAL" in col_norm:
                    score += 15
                if col_norm in {"EVENTDATE", "EVENTDATEOF"} or "EVENTDATE" in col_norm:
                    score -= 10
                return score

            return 0

        # Process alias mappings first (best candidate per target)
        for target, candidates in alias_candidates_by_target.items():
            if target in assigned_targets:
                for col, _, _ in candidates:
                    unmapped.append(col)
                continue

            # Prefer the best-scoring candidate; tie-break by source column order.
            best = max(
                candidates,
                key=lambda item: (
                    _alias_priority(target, item[0]),
                    -list(frame.columns).index(item[0]),
                ),
            )
            best_col, codelist_name, code_column = best

            assigned_targets.add(target)
            suggestions.append(
                ColumnMapping(
                    source_column=safe_column_name(best_col),
                    target_variable=target,
                    transformation=None,
                    confidence_score=1.0,
                    codelist_name=codelist_name,
                    use_code_column=code_column,
                )
            )

            for col, _, _ in candidates:
                if col != best_col:
                    unmapped.append(col)

        # Second pass: fuzzy matching for remaining columns
        # Second pass: fuzzy matching for remaining columns
        already_mapped_columns = {m.source_column.strip('"') for m in suggestions}
        for column in frame.columns:
            if column in already_mapped_columns:
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
                    source_column=safe_column_name(column),
                    target_variable=match.candidate,
                    transformation=None,
                    confidence_score=match.confidence,
                    codelist_name=codelist_name,
                )
            )

        return MappingSuggestions(mappings=suggestions, unmapped_columns=unmapped)

    def _best_fuzzy_match(self, column: str) -> Suggestion | None:
        """Find the best fuzzy match for a column.

        Args:
            column: Source column name

        Returns:
            Best matching suggestion or None if no good match found
        """
        normalized = normalize_text(column)

        # Use Items.csv metadata (when available) to avoid pathological matches.
        source_type = ""
        source_label = ""
        if self.metadata:
            col_def = self.metadata.get_column(column)
            if col_def:
                source_type = (col_def.data_type or "").strip().lower()
                source_label = (col_def.label or "").strip().lower()

        column_lower = column.strip().lower()
        source_is_date_like = source_type in {
            "date",
            "datetime",
            "time",
            "timestamp",
        } or ("date" in column_lower or "date" in source_label)
        source_is_numeric_like = source_type in {
            "int",
            "integer",
            "double",
            "float",
            "number",
            "numeric",
            "num",
            "decimal",
        }
        best: Suggestion | None = None

        for variable in self.domain.variables:
            # Calculate similarity scores
            score_raw = fuzz.token_set_ratio(column.upper(), variable.name)
            score_norm = fuzz.ratio(normalized, variable.name)
            score = max(score_raw, score_norm) / 100

            var_name = variable.name.upper()
            variable_is_numeric = variable.type.lower() == "num"
            variable_is_date_like = var_name.endswith("DTC") or var_name.endswith("DT")
            variable_is_test_like = var_name.endswith(("TEST", "TESTCD"))
            if var_name.startswith(self.domain_code):
                remainder = var_name[len(self.domain_code) :]
                if remainder.startswith("TEST"):
                    variable_is_test_like = True

            # If the source looks like a date/time column, strongly prefer --DTC/--DT
            # and prevent mapping to categorical/status/test variables.
            if source_is_date_like:
                if variable_is_test_like or var_name.endswith(("CAT", "SCAT", "STAT")):
                    score *= 0.05
                elif variable_is_date_like:
                    score = min(1.0, score * 1.20)
                else:
                    score *= 0.35

            # If the source metadata says it's numeric, prefer Num variables.
            if source_is_numeric_like and not variable_is_numeric:
                score *= 0.70
            if (not source_is_numeric_like) and variable_is_numeric and source_type:
                score *= 0.85

            # Apply hints if available
            score = self._apply_hints(column, variable, score)

            if not best or score > best.confidence:
                best = Suggestion(
                    column=column, candidate=variable.name, confidence=score
                )

        return best

    def _apply_hints(self, column: str, variable: SDTMVariable, score: float) -> float:
        """Apply column hints to adjust the confidence score.

        Args:
            column: Source column name
            variable: Target SDTM variable
            score: Initial confidence score

        Returns:
            Adjusted confidence score
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
