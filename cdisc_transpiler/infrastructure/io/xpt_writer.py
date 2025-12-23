"""XPT writer adapter.

This module provides an adapter implementation for writing XPT (SAS Transport)
files while conforming to the XPTWriterPort protocol.
"""

from __future__ import annotations

from pathlib import Path

import numpy as np
import pandas as pd
import pyreadstat

from cdisc_transpiler.infrastructure.sdtm_spec.registry import get_domain


class XportGenerationError(RuntimeError):
    """Raised when XPT export cannot be completed."""


def _order_columns_for_domain(dataset: pd.DataFrame, *, domain: object) -> list[str]:
    """Return dataset columns ordered per SDTMIG spec for the domain.

    Important: do NOT move unknown/sponsor columns.
    Only reorders the subset of columns known to the SDTMIG spec, while leaving
    any non-spec columns anchored in their original positions.
    """
    dataset_columns = [str(c) for c in dataset.columns]
    present_upper = {c.upper() for c in dataset_columns}

    spec_order_upper: list[str] = []
    domain_vars = getattr(domain, "variables", None) or []
    for var in domain_vars:
        name = getattr(var, "name", None)
        if not name:
            continue
        upper = str(name).upper()
        if upper in present_upper:
            spec_order_upper.append(upper)

    # Preserve original casing of the incoming dataset columns.
    by_upper = {c.upper(): c for c in dataset_columns}
    spec_set = set(spec_order_upper)
    spec_iter = iter(spec_order_upper)

    ordered: list[str] = []
    for col in dataset_columns:
        if col.upper() in spec_set:
            try:
                next_upper = next(spec_iter)
            except StopIteration:  # pragma: no cover
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
    """Persist the DataFrame as a SAS v5 transport file."""
    output_path = Path(path)

    # Force lower-case disk names to match MSG sample package convention
    output_path = output_path.with_name(output_path.name.lower())

    if len(output_path.stem) > 8:
        raise XportGenerationError(
            "XPT filename stem must be <=8 characters to satisfy SDTM v5: "
            f"{output_path.name}"
        )

    output_path.parent.mkdir(parents=True, exist_ok=True)

    if output_path.exists():
        output_path.unlink()

    domain = get_domain(domain_code)
    dataset_name = (table_name or domain.resolved_dataset_name()).upper()[:8]

    ordered_columns = _order_columns_for_domain(dataset, domain=domain)
    dataset = dataset.loc[:, ordered_columns]

    # Normalize domain-variable lookup to uppercase so column casing differences
    # don't cause silent misses (e.g., expected numeric EXDOSE written as Char).
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

    # pyreadstat does not reliably handle pandas extension dtypes
    # (e.g., Int64/BooleanDtype/StringDtype backed by IntegerArray/BooleanArray).
    # Normalize to numpy-friendly dtypes at the infrastructure boundary.
    #
    # Note: build a fresh DataFrame to avoid Copy-on-Write / chained-assignment warnings
    # when callers pass in a slice/view.
    export_df = pd.DataFrame(index=dataset.index)
    for column_index, col in enumerate(dataset.columns):
        series = dataset.iloc[:, column_index]

        col_upper = str(col).upper()
        expected_type = type_lookup.get(col_upper)

        values: np.ndarray

        expected_upper = (expected_type or "").strip().upper()

        # Preserve incoming dtype/semantics by default (important for validation
        # fixtures that round-trip official XPTs), but force numeric for a small
        # allow-list where validators expect numeric even when the source dtype
        # is object/string.
        force_numeric = col_upper in {"EXDOSE"} and expected_upper == "NUM"

        is_char_like = not force_numeric and (
            expected_upper == "CHAR"
            or isinstance(series.dtype, pd.CategoricalDtype)
            or isinstance(series.dtype, pd.StringDtype)
            or pd.api.types.is_object_dtype(series.dtype)
        )

        if force_numeric:
            values = pd.to_numeric(series, errors="coerce").to_numpy(
                dtype="float64", na_value=np.nan
            )
        elif is_char_like:
            # Keep literal strings (e.g., "NONE") intact; only normalize actual missing
            # values to empty strings so pyreadstat writes consistent character fields.
            normalized = series.astype(object)
            normalized = pd.Series(normalized, index=dataset.index).where(
                ~pd.isna(normalized), ""
            )

            # pyreadstat derives SAS character widths from the max observed string
            # length. If a column is entirely empty, that can yield a zero-length
            # character variable in the XPT, which downstream validators flag.
            # Use a single space to force width=1 while still representing SAS missing.
            lengths = normalized.astype("string").fillna("").str.len()
            max_length = lengths.max()
            if pd.isna(max_length) or int(max_length) == 0:
                normalized = pd.Series([" "] * len(dataset.index), index=dataset.index)
            values = normalized.to_numpy(dtype=object)
        # Otherwise preserve incoming dtype/semantics.
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
    except Exception as exc:  # pragma: no cover
        raise XportGenerationError(f"Failed to write XPT file: {exc}") from exc


class XPTWriter:
    """Adapter for writing XPT (SAS Transport) files.

    This class implements the XPTWriterPort protocol and delegates to the
    concrete infrastructure writer in this module.

    Example:
        >>> writer = XPTWriter()
        >>> df = pd.DataFrame({"STUDYID": ["001"], "USUBJID": ["001-001"]})
        >>> writer.write(df, "DM", Path("output/dm.xpt"))
    """

    def write(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        output_path: Path,
        *,
        file_label: str | None = None,
        table_name: str | None = None,
    ) -> None:
        """Write a DataFrame to an XPT file.

        Args:
            dataframe: Data to write
            domain_code: SDTM domain code (e.g., "DM", "AE")
            output_path: Path where XPT file should be written

        Raises:
            Exception: If writing fails

        Example:
            >>> writer = XPTWriter()
            >>> df = pd.DataFrame({"STUDYID": ["001"]})
            >>> writer.write(df, "DM", Path("dm.xpt"))
        """
        write_xpt_file(
            dataframe,
            domain_code,
            output_path,
            file_label=file_label,
            table_name=table_name,
        )
