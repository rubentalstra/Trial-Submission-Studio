"""Numeric transformation utilities for SDTM domains."""

from __future__ import annotations

from typing import Any

import pandas as pd

from ....pandas_utils import ensure_numeric_series


class NumericTransformer:
    """Transforms numeric values for SDTM compliance."""

    @staticmethod
    def populate_stresc_from_orres(frame: pd.DataFrame, domain_code: str) -> None:
        orres_col = f"{domain_code}ORRES"
        stresc_col = f"{domain_code}STRESC"

        if orres_col in frame.columns and stresc_col in frame.columns:
            orres_str = (
                frame[orres_col]
                .astype(str)
                .replace({"nan": "", "None": "", "<NA>": ""})
            )
            stresc_str = (
                frame[stresc_col]
                .astype(str)
                .replace({"nan": "", "None": "", "<NA>": ""})
            )

            mask = (stresc_str.str.strip() == "") & (orres_str.str.strip() != "")
            frame.loc[mask, stresc_col] = orres_str.loc[mask]

    @staticmethod
    def force_numeric(series: Any) -> pd.Series:
        return ensure_numeric_series(series)

    @staticmethod
    def assign_sequence(frame: pd.DataFrame, seq_var: str, group_by: str) -> None:
        if seq_var not in frame.columns or group_by not in frame.columns:
            return
        frame[seq_var] = frame.groupby(group_by).cumcount() + 1
