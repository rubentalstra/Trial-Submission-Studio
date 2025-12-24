from collections.abc import Mapping
from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class ColumnHint:
    is_numeric: bool
    unique_ratio: float
    null_ratio: float


Hints = Mapping[str, ColumnHint]
