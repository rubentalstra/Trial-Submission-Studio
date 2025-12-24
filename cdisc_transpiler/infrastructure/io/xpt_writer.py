from pathlib import Path
from typing import TYPE_CHECKING

import numpy as np
import pandas as pd
import pyreadstat

from cdisc_transpiler.infrastructure.sdtm_spec.registry import get_domain

if TYPE_CHECKING:
    from cdisc_transpiler.domain.entities.sdtm_domain import SDTMDomain
MAX_XPT_FILENAME_STEM = 8


class XportGenerationError(RuntimeError):
    pass


def _order_columns_for_domain(
    dataset: pd.DataFrame, *, domain: SDTMDomain
) -> list[str]:
    dataset_columns = [str(c) for c in dataset.columns]
    present_upper = {c.upper() for c in dataset_columns}
    spec_order_upper: list[str] = []
    for var in domain.variables:
        upper = var.name.upper()
        if upper in present_upper:
            spec_order_upper.append(upper)
    by_upper = {c.upper(): c for c in dataset_columns}
    spec_set = set(spec_order_upper)
    spec_iter = iter(spec_order_upper)
    ordered: list[str] = []
    for col in dataset_columns:
        if col.upper() in spec_set:
            try:
                next_upper = next(spec_iter)
            except StopIteration:
                ordered.append(col)
                continue
            ordered.append(by_upper.get(next_upper, col))
        else:
            ordered.append(col)
    return ordered


def write_xpt_file(
    dataset: pd.DataFrame,
    domain_code: str,
    path: str | Path,
    *,
    file_label: str | None = None,
    table_name: str | None = None,
) -> None:
    output_path = Path(path)
    output_path = output_path.with_name(output_path.name.lower())
    if len(output_path.stem) > MAX_XPT_FILENAME_STEM:
        raise XportGenerationError(
            f"XPT filename stem must be <=8 characters to satisfy SDTM v5: {output_path.name}"
        )
    output_path.parent.mkdir(parents=True, exist_ok=True)
    if output_path.exists():
        output_path.unlink()
    domain = get_domain(domain_code)
    dataset_name = (table_name or domain.resolved_dataset_name()).upper()[:8]
    ordered_columns = _order_columns_for_domain(dataset, domain=domain)
    dataset = dataset.loc[:, ordered_columns]
    label_lookup = {
        str(variable.name).upper(): variable.label for variable in domain.variables
    }
    type_lookup = {
        str(variable.name).upper(): str(variable.type) for variable in domain.variables
    }
    column_labels = [
        str(label_lookup.get(str(col).upper(), col))[:40] for col in dataset.columns
    ]
    default_label = (domain.label or domain.description or dataset_name).strip()
    label = default_label if file_label is None else file_label
    label = (label or "").strip()[:40] or None
    export_df = pd.DataFrame(index=dataset.index)
    for column_index, col in enumerate(dataset.columns):
        series = dataset.iloc[:, column_index]
        col_upper = str(col).upper()
        expected_type = type_lookup.get(col_upper)
        values: np.ndarray
        expected_upper = (expected_type or "").strip().upper()
        force_numeric = col_upper in {"EXDOSE"} and expected_upper == "NUM"
        is_char_like = not force_numeric and (
            expected_upper == "CHAR"
            or isinstance(series.dtype, (pd.CategoricalDtype, pd.StringDtype))
            or pd.api.types.is_object_dtype(series.dtype)
        )
        if force_numeric:
            values = pd.to_numeric(series, errors="coerce").to_numpy(
                dtype="float64", na_value=np.nan
            )
        elif is_char_like:
            normalized = series.astype(object)
            normalized = pd.Series(normalized, index=dataset.index).where(
                ~pd.isna(normalized), ""
            )
            lengths = normalized.astype("string").fillna("").str.len()
            max_length = lengths.max()
            if pd.isna(max_length) or int(max_length) == 0:
                normalized = pd.Series([" "] * len(dataset.index), index=dataset.index)
            values = normalized.to_numpy(dtype=object)
        elif (
            expected_upper == "NUM"
            or pd.api.types.is_bool_dtype(series.dtype)
            or pd.api.types.is_numeric_dtype(series.dtype)
        ):
            values = pd.to_numeric(series, errors="coerce").to_numpy(
                dtype="float64", na_value=np.nan
            )
        elif pd.api.types.is_extension_array_dtype(series.dtype):
            values = series.to_numpy(dtype=object)
        else:
            values = series.to_numpy()
        export_df.insert(column_index, col, values, allow_duplicates=True)
    try:
        pyreadstat.write_xport(
            export_df,
            str(output_path),
            file_label=label,
            column_labels=column_labels,
            table_name=dataset_name,
            file_format_version=5,
        )
    except Exception as exc:
        raise XportGenerationError(f"Failed to write XPT file: {exc}") from exc


class XPTWriter:
    pass

    def write(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        output_path: Path,
        *,
        file_label: str | None = None,
        table_name: str | None = None,
    ) -> None:
        write_xpt_file(
            dataframe,
            domain_code,
            output_path,
            file_label=file_label,
            table_name=table_name,
        )
