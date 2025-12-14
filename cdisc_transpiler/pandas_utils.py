"""Lightweight pandas typing helpers.

These utilities centralize conversions to :class:`pandas.Series` so that
type checkers can reason about the resulting objects. The codebase heavily
relies on dynamic pandas operations where ``DataFrame.__getitem__`` returns
either a :class:`Series` or :class:`DataFrame`; by routing through these
helpers we guarantee a concrete ``Series`` instance and stable return types.
"""

from __future__ import annotations

from typing import Any

import pandas as pd


def ensure_series(value: Any, index: pd.Index | None = None) -> pd.Series:
    """Coerce ``value`` to a :class:`pandas.Series`.

    The function is intentionally permissive: it accepts scalars, lists,
    ``Series`` instances, and single-column ``DataFrame`` objects. Multi-column
    frames fall back to the first column, which preserves legacy behaviour
    where a frame lookup was expected to yield a series.
    """
    if isinstance(value, pd.Series):
        return value
    if isinstance(value, pd.DataFrame):
        if value.shape[1] == 0:
            return pd.Series(index=value.index, dtype="object")
        return value.iloc[:, 0]
    return pd.Series(value, index=index)


def ensure_numeric_series(value: Any, index: pd.Index | None = None) -> pd.Series:
    """Return a numeric :class:`Series` with ``NaN`` on conversion failures."""
    series = ensure_series(value, index=index)
    numeric = pd.to_numeric(series, errors="coerce")
    return ensure_series(numeric, index=series.index)
