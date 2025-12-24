from typing import TYPE_CHECKING

from rapidfuzz import fuzz

if TYPE_CHECKING:
    import pandas as pd

    from ....domain.entities.study_metadata import StudyMetadata
    from ...entities.column_hints import Hints
    from ...entities.sdtm_domain import SDTMDomain, SDTMVariable
from ....domain.entities.mapping import ColumnMapping, MappingSuggestions, Suggestion
from .pattern_builder import build_variable_patterns
from .utils import normalize_text, safe_column_name

SEQ_UNIQUENESS_MIN = 0.5
REQUIRED_NULL_RATIO_MAX = 0.5


class MetadataAwareMapper:
    pass

    def __init__(
        self,
        domain: SDTMDomain,
        metadata: StudyMetadata | None = None,
        *,
        min_confidence: float = 0.5,
        column_hints: Hints | None = None,
    ) -> None:
        super().__init__()
        self.domain = domain
        self.domain_code = domain.code.upper()
        self.metadata = metadata
        self.min_confidence = min_confidence
        self.column_hints: Hints = column_hints or {}
        self.valid_targets: set[str] = set(self.domain.variable_names())
        self._aliases: dict[str, str] = {}
        self._variable_patterns = build_variable_patterns(self.domain)
        self._build_alias_dictionary()

    def _build_alias_dictionary(self) -> None:
        for target_var, patterns in self._variable_patterns.items():
            if target_var not in self.valid_targets:
                continue
            for pattern in patterns:
                if pattern not in self._aliases:
                    self._aliases[pattern] = target_var
        if self.metadata and self.metadata.items:
            self._add_metadata_aliases()
        self._add_domain_specific_aliases()

    def _add_domain_specific_aliases(self) -> None:
        def _add(pattern: str, target: str) -> None:
            key = normalize_text(pattern)
            if key not in self._aliases:
                self._aliases[key] = target

        dom = self.domain_code
        if dom == "EX":
            _add("EXSTDAT", "EXSTDTC")
            _add("EXENDAT", "EXENDTC")
        if dom == "DS":
            _add("LCDAT", "DSSTDTC")
            _add("ETDAT", "DSSTDTC")

    def _add_metadata_aliases(self) -> None:
        if not self.metadata:
            return
        for col_id, item in self.metadata.items.items():
            normalized = normalize_text(col_id)
            if col_id.startswith(self.domain_code) and col_id in self.valid_targets:
                self._aliases[normalized] = col_id
                continue
            for target_var, patterns in self._variable_patterns.items():
                if target_var not in self.valid_targets:
                    continue
                if normalized in patterns:
                    self._aliases[normalized] = target_var
                    break
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
        alias_candidates_by_target = self._collect_alias_candidates(frame)
        suggestions, unmapped, assigned_targets = self._resolve_alias_candidates(
            frame, alias_candidates_by_target
        )
        self._apply_fuzzy_matching(frame, suggestions, unmapped, assigned_targets)
        return MappingSuggestions(mappings=suggestions, unmapped_columns=unmapped)

    def _collect_alias_candidates(
        self, frame: pd.DataFrame
    ) -> dict[str, list[tuple[str, str | None, str | None]]]:
        alias_candidates_by_target: dict[
            str, list[tuple[str, str | None, str | None]]
        ] = {}
        for column in frame.columns:
            normalized = normalize_text(column)
            target = self._aliases.get(normalized)
            if not target:
                continue
            codelist_name = None
            code_column = None
            if self.metadata:
                codelist_name, code_column, skip = self._resolve_codelist_info(
                    frame, column
                )
                if skip:
                    continue
            alias_candidates_by_target.setdefault(target, []).append(
                (column, codelist_name, code_column)
            )
        return alias_candidates_by_target

    def _resolve_codelist_info(
        self, frame: pd.DataFrame, column: str
    ) -> tuple[str | None, str | None, bool]:
        if not self.metadata:
            return (None, None, False)
        if column.endswith("CD"):
            text_col = column[:-2]
            if text_col in frame.columns:
                return (None, None, True)
        codelist_name = None
        code_column = None
        code_col = column + "CD"
        if code_col in frame.columns:
            code_column = code_col
            col_def = self.metadata.get_column(code_col)
            if col_def and col_def.format_name:
                codelist_name = col_def.format_name
        else:
            col_def = self.metadata.get_column(column)
            if col_def and col_def.format_name:
                codelist_name = col_def.format_name
        return (codelist_name, code_column, False)

    def _alias_priority(self, target: str | None, column: str) -> int:
        t = (target or "").strip().upper()
        col_norm = normalize_text(column)
        if self.domain_code == "DS" and t == "DSSTDTC":
            score = 0
            if "LASTCONTACT" in col_norm or (
                "LAST" in col_norm and "CONTACT" in col_norm
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

    def _resolve_alias_candidates(
        self,
        frame: pd.DataFrame,
        alias_candidates_by_target: dict[str, list[tuple[str, str | None, str | None]]],
    ) -> tuple[list[ColumnMapping], list[str], set[str]]:
        suggestions: list[ColumnMapping] = []
        unmapped: list[str] = []
        assigned_targets: set[str] = set()
        column_positions = {name: idx for idx, name in enumerate(frame.columns)}
        for target, candidates in alias_candidates_by_target.items():
            if target in assigned_targets:
                unmapped.extend((col for col, _, _ in candidates))
                continue
            best = max(
                candidates,
                key=lambda item: (
                    self._alias_priority(target, item[0]),
                    -column_positions[item[0]],
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
        return (suggestions, unmapped, assigned_targets)

    def _apply_fuzzy_matching(
        self,
        frame: pd.DataFrame,
        suggestions: list[ColumnMapping],
        unmapped: list[str],
        assigned_targets: set[str],
    ) -> None:
        already_mapped_columns = {m.source_column.strip('"') for m in suggestions}
        for column in frame.columns:
            if column in already_mapped_columns:
                continue
            if column.endswith("CD") and column[:-2] in frame.columns:
                continue
            match = self._best_fuzzy_match(column)
            if match is None or match.confidence < self.min_confidence:
                unmapped.append(column)
                continue
            if match.candidate in assigned_targets:
                unmapped.append(column)
                continue
            assigned_targets.add(match.candidate)
            codelist_name = self._codelist_for_column(column)
            suggestions.append(
                ColumnMapping(
                    source_column=safe_column_name(column),
                    target_variable=match.candidate,
                    transformation=None,
                    confidence_score=match.confidence,
                    codelist_name=codelist_name,
                )
            )

    def _codelist_for_column(self, column: str) -> str | None:
        if not self.metadata:
            return None
        col_def = self.metadata.get_column(column)
        if col_def and col_def.format_name:
            return col_def.format_name
        return None

    def _best_fuzzy_match(self, column: str) -> Suggestion | None:
        normalized = normalize_text(column)
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
            if source_is_date_like:
                if variable_is_test_like or var_name.endswith(("CAT", "SCAT", "STAT")):
                    score *= 0.05
                elif variable_is_date_like:
                    score = min(1.0, score * 1.2)
                else:
                    score *= 0.35
            if source_is_numeric_like and (not variable_is_numeric):
                score *= 0.7
            if not source_is_numeric_like and variable_is_numeric and source_type:
                score *= 0.85
            score = self._apply_hints(column, variable, score)
            if not best or score > best.confidence:
                best = Suggestion(
                    column=column, candidate=variable.name, confidence=score
                )
        return best

    def _apply_hints(self, column: str, variable: SDTMVariable, score: float) -> float:
        hint = self.column_hints.get(column)
        if not hint:
            return score
        adjusted = score
        variable_is_numeric = variable.type.lower() == "num"
        if variable_is_numeric != hint.is_numeric:
            adjusted *= 0.85
        if variable.name.endswith("SEQ") and hint.unique_ratio < SEQ_UNIQUENESS_MIN:
            adjusted *= 0.9
        if (
            hint.null_ratio > REQUIRED_NULL_RATIO_MAX
            and (variable.core or "").strip().lower() == "req"
        ):
            adjusted *= 0.9
        return adjusted
