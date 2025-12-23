"""Codelist and controlled terminology transformation utilities."""

from __future__ import annotations

from collections.abc import Callable, Sequence
from typing import TYPE_CHECKING, Any, Protocol

import pandas as pd

from ....pandas_utils import ensure_numeric_series, ensure_series
from ...entities.controlled_terminology import ControlledTerminology
from ...entities.sdtm_domain import SDTMVariable

if TYPE_CHECKING:
    from ...entities.study_metadata import StudyMetadata


class CTResolver(Protocol):
    def __call__(
        self, *, codelist_code: str | None = None, variable: str | None = None
    ) -> ControlledTerminology | None: ...


class CodelistTransformer:
    """Transforms and validates controlled terminology values for SDTM compliance."""

    def __init__(
        self,
        metadata: StudyMetadata | None = None,
        *,
        ct_resolver: CTResolver | None = None,
    ):
        self.metadata = metadata
        self._ct_resolver = ct_resolver

    def apply_codelist_transformation(
        self,
        source_data: Any,
        codelist_name: str,
        code_column: str | None = None,
        source_frame: pd.DataFrame | None = None,
        unquote_func: Callable[[str], str] | None = None,
    ) -> pd.Series:
        if not self.metadata:
            return ensure_series(source_data)

        codelist = self.metadata.get_codelist(codelist_name)
        if not codelist:
            return ensure_series(source_data)

        if code_column and source_frame is not None:
            code_col = code_column
            if code_col not in source_frame.columns and unquote_func:
                alt = unquote_func(code_col)
                if alt in source_frame.columns:
                    code_col = alt

            if code_col in source_frame.columns:
                code_values = ensure_series(
                    source_frame[code_col],
                    index=source_data.index if hasattr(source_data, "index") else None,
                )

                def transform(code_val: Any) -> Any:
                    if pd.isna(code_val):
                        return None
                    text = codelist.get_text(code_val)
                    return text if text is not None else str(code_val)

                return ensure_series(
                    code_values.map(transform), index=code_values.index
                )

        def transform_value(val: Any) -> Any:
            if pd.isna(val):
                return val
            text = codelist.get_text(val)
            if text is not None:
                return text
            return val

        series = ensure_series(source_data)
        return ensure_series(series.map(transform_value), index=series.index)

    @staticmethod
    def apply_codelist_validations(
        frame: pd.DataFrame,
        domain_variables: Sequence[SDTMVariable],
        *,
        ct_resolver: CTResolver | None = None,
    ) -> None:
        for var in domain_variables:
            if var.codelist_code and var.name in frame.columns:
                if var.name == "TSVCDREF":
                    continue

                resolver = ct_resolver
                ct_lookup = None
                if resolver is not None:
                    ct_lookup = resolver(codelist_code=var.codelist_code) or resolver(
                        variable=var.name
                    )
                if ct_lookup is None:
                    continue
                normalizer = ct_lookup.normalize

                series = ensure_series(frame[var.name]).astype("string")
                trimmed = series.str.strip()
                mask = trimmed.notna() & (trimmed != "")

                def _normalize_ct_value(value: str) -> str:
                    normalized = normalizer(value)
                    return normalized or value

                normalized = series.copy()
                # Explicitly cast the result of apply to Series[str] via ensure_series if needed,
                # but here we are assigning to a slice.
                # To help pyright, we can extract the series to apply on.
                to_normalize = ensure_series(trimmed.loc[mask])
                normalized.loc[mask] = to_normalize.map(_normalize_ct_value)
                frame[var.name] = normalized.astype("string")

    @staticmethod
    def validate_controlled_terms(
        frame: pd.DataFrame,
        domain_variables: list[SDTMVariable],
        *,
        ct_resolver: CTResolver | None = None,
    ) -> None:
        for variable in domain_variables:
            if not variable.codelist_code:
                continue
            if ct_resolver is None:
                continue
            ct = ct_resolver(codelist_code=variable.codelist_code)
            if not ct or variable.name not in frame.columns:
                continue
            invalid = ct.invalid_values(frame[variable.name])
            if invalid:
                canonical_default = sorted(ct.submission_values)[0]
                series = ensure_series(frame[variable.name]).astype(str)
                frame[variable.name] = series.where(
                    ~series.isin(list(invalid)), canonical_default
                )

    @staticmethod
    def validate_paired_terms(frame: pd.DataFrame) -> None:
        pairs = [
            ("AETEST", "AETESTCD"),
            ("LBTEST", "LBTESTCD"),
            ("VSTEST", "VSTESTCD"),
            ("QSTEST", "QSTESTCD"),
            ("MHDECOD", "MHTERM"),
        ]

        for left, right in pairs:
            if left not in frame.columns or right not in frame.columns:
                continue
            left_series = frame[left].astype(str).str.strip()
            right_series = frame[right].astype(str).str.strip()
            missing_right = (left_series != "") & (right_series == "")
            missing_left = (right_series != "") & (left_series == "")
            if missing_right.any() or missing_left.any():
                raise ValueError(
                    f"Paired terminology mismatch for {left}/{right}: both must be populated together"
                )

    @staticmethod
    def populate_meddra_defaults(frame: pd.DataFrame) -> None:
        aetext = frame.get("AETERM", pd.Series(["" for _ in frame.index]))
        if "AEDECOD" in frame.columns:
            decod = frame["AEDECOD"].astype("string")
            frame["AEDECOD"] = decod.where(decod.str.strip() != "", aetext)
        else:
            frame["AEDECOD"] = aetext

        for soc_var, term in {
            "AEBODSYS": "GENERAL DISORDERS",
            "AESOC": "GENERAL DISORDERS",
            "AEHLGT": "GENERAL DISORDERS",
            "AEHLT": "GENERAL DISORDERS",
            "AELLT": "GENERAL DISORDERS",
        }.items():
            if soc_var in frame.columns:
                s = ensure_series(frame[soc_var]).astype("string")
                s = ensure_series(s.fillna(""))
                s = ensure_series(s.replace("", term))
                s = ensure_series(s.fillna(term))
                frame[soc_var] = s
            else:
                frame[soc_var] = term

        for code_var in (
            "AEPTCD",
            "AEHLGTCD",
            "AEHLTCD",
            "AELLTCD",
            "AESOCCD",
            "AEBDSYCD",
        ):
            if code_var in frame.columns:
                code_series = ensure_numeric_series(frame[code_var], index=frame.index)
                s_filled = ensure_series(code_series.fillna(999999))
                frame[code_var] = s_filled.astype("Int64")
            else:
                frame[code_var] = pd.Series(
                    [999999 for _ in frame.index], dtype="Int64"
                )
