from collections.abc import Sequence

import pandas as pd

__all__ = [
    "read_sas7bdat",
    "read_xport",
    "write_xport",
]

def read_sas7bdat(path: str, **kwargs: object) -> tuple[pd.DataFrame, object]: ...
def read_xport(path: str, **kwargs: object) -> tuple[pd.DataFrame, object]: ...
def write_xport(
    df: pd.DataFrame,
    path: str,
    *,
    file_label: str | None = None,
    column_labels: Sequence[str] | None = None,
    table_name: str | None = None,
    file_format_version: int | None = None,
    **kwargs: object,
) -> None: ...
