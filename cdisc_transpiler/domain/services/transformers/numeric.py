"""Numeric transformation utilities for SDTM domains."""

from typing import TYPE_CHECKING

from ....pandas_utils import ensure_numeric_series, ensure_series

if TYPE_CHECKING:
    import pandas as pd


class NumericTransformer:
    """Transforms numeric values for SDTM compliance."""

    @staticmethod
    def populate_stresc_from_orres(frame: pd.DataFrame, domain_code: str) -> None:
        orres_col = f"{domain_code}ORRES"
        stresc_col = f"{domain_code}STRESC"

        if orres_col in frame.columns and stresc_col in frame.columns:
            s_orres = ensure_series(frame[orres_col]).astype(str)
            orres_str = ensure_series(
                s_orres.replace({"nan": "", "None": "", "<NA>": ""})
            )
            s_stresc = ensure_series(frame[stresc_col]).astype(str)
            stresc_str = ensure_series(
                s_stresc.replace({"nan": "", "None": "", "<NA>": ""})
            )

            mask = (stresc_str.str.strip() == "") & (orres_str.str.strip() != "")
            frame.loc[mask, stresc_col] = orres_str.loc[mask]

    @staticmethod
    def force_numeric(series: object) -> pd.Series:
        return ensure_numeric_series(series)

    @staticmethod
    def assign_sequence(frame: pd.DataFrame, seq_var: str, group_by: str) -> None:
        if seq_var not in frame.columns or group_by not in frame.columns:
            return
        frame.loc[:, seq_var] = frame.groupby(group_by).cumcount() + 1
