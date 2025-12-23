from __future__ import annotations

from collections.abc import Sequence

import pandas as pd

class Metadata: ...

def read_sas7bdat(
    path: str,
    *,
    encoding: str | None = ...,
    encoding_errors: str | None = ...,
    usecols: Sequence[str] | None = ...,
    disable_datetime_conversion: bool | None = ...,
    formats_as_category: bool | None = ...,
    **kwargs: object,
) -> tuple[pd.DataFrame, Metadata]: ...
def write_xport(
    df: pd.DataFrame,
    path: str,
    *,
    file_label: str | None = ...,
    column_labels: Sequence[str] | None = ...,
    table_name: str | None = ...,
    file_format_version: int | None = ...,
    **kwargs: object,
) -> None: ...
