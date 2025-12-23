"""Text transformation utilities for SDTM domains."""

from typing import TYPE_CHECKING

from ....pandas_utils import ensure_numeric_series, ensure_series

if TYPE_CHECKING:
    import pandas as pd


class TextTransformer:
    """Transforms text values for SDTM compliance."""

    @staticmethod
    def replace_unknown(series: object, default: str) -> pd.Series:
        normalized = ensure_series(series).astype("string").fillna("")
        upper = normalized.str.upper()
        missing_tokens = {"", "UNK", "UNKNOWN", "NA", "N/A", "NONE", "NAN", "<NA>"}
        normalized.loc[upper.isin(missing_tokens)] = default
        normalized = normalized.fillna(default)
        return normalized.astype(str)

    @staticmethod
    def normalize_visit(frame: pd.DataFrame) -> None:
        def _visit_label(value: object) -> str:
            try:
                return f"Visit {int(float(str(value)))}"
            except (TypeError, ValueError):
                return "Visit 1"

        if "VISITNUM" in frame.columns:
            frame.loc[:, "VISITNUM"] = (
                ensure_numeric_series(frame["VISITNUM"], frame.index)
                .fillna(1)
                .astype(int)
            )
            frame.loc[:, "VISIT"] = frame["VISITNUM"].map(_visit_label).astype("string")
        elif "VISIT" in frame.columns:
            visit_text = frame["VISIT"].astype("string").str.extract(r"(\d+)")[0]
            frame.loc[:, "VISITNUM"] = (
                ensure_numeric_series(visit_text, frame.index).fillna(1).astype(int)
            )
            frame.loc[:, "VISIT"] = frame["VISITNUM"].map(_visit_label).astype("string")
