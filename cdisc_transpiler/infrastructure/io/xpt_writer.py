"""XPT writer adapter.

This module provides an adapter implementation for writing XPT (SAS Transport)
files while conforming to the XPTWriterPort protocol.
"""

from __future__ import annotations

from pathlib import Path

import pandas as pd
import pyreadstat  # type: ignore[import-untyped]

from cdisc_transpiler.infrastructure.sdtm_spec.registry import get_domain


class XportGenerationError(RuntimeError):
    """Raised when XPT export cannot be completed."""


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

    label_lookup = {variable.name: variable.label for variable in domain.variables}
    column_labels = [str(label_lookup.get(col, col))[:40] for col in dataset.columns]

    default_label = (domain.label or domain.description or dataset_name).strip()
    label = default_label if file_label is None else file_label
    label = (label or "").strip()[:40] or None

    try:
        pyreadstat.write_xport(
            dataset,
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
