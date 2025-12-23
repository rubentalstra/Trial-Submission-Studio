"""Column-hint domain types.

These types model lightweight per-column statistics used by mapping heuristics.

They live in the domain layer so that mapping services (domain) and ports
(application) do not depend on infrastructure adapters or legacy wrappers.
"""

from collections.abc import Mapping
from dataclasses import dataclass


@dataclass(frozen=True)
class ColumnHint:
    """Lightweight stats about a column used during mapping heuristics."""

    is_numeric: bool
    unique_ratio: float
    null_ratio: float


Hints = Mapping[str, ColumnHint]
