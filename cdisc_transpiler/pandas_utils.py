from typing import Any, cast

import pandas as pd

from .constants import MissingValues


def ensure_series(value: object, index: pd.Index[Any] | None = None) -> pd.Series[Any]:
    if isinstance(value, pd.Series):
        return cast("pd.Series[Any]", value)
    if isinstance(value, pd.DataFrame):
        if value.shape[1] == 0:
            return pd.Series(index=value.index, dtype="object")
        return value.iloc[:, 0]
    return pd.Series(cast("Any", value), index=index)


def ensure_numeric_series(
    value: object, index: pd.Index[Any] | None = None
) -> pd.Series[Any]:
    series = ensure_series(value, index=index)
    numeric = pd.to_numeric(series, errors="coerce")
    return ensure_series(numeric, index=series.index)


def is_missing_scalar(value: object) -> bool:
    try:
        return cast("bool", pd.isna(cast("Any", value)))
    except (TypeError, ValueError):
        return False


def normalize_missing_strings(
    value: object, *, replacement: str = "", markers: set[str] | None = None
) -> pd.Series[str]:
    series = ensure_series(value).astype("string")
    stripped = series.str.strip()
    marker_set = {m.upper() for m in markers or MissingValues.STRING_MARKERS}
    upper = stripped.str.upper()
    marker_mask = upper.isin(marker_set)
    return stripped.mask(marker_mask, replacement)
