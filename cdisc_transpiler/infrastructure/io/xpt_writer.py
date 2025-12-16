"""XPT writer adapter.

This module provides an adapter implementation for writing XPT (SAS Transport)
files while conforming to the XPTWriterPort protocol.
"""

from __future__ import annotations

from pathlib import Path

import pandas as pd

from .xpt_write import write_xpt_file


class XPTWriter:
    """Adapter for writing XPT (SAS Transport) files.

    This class implements the XPTWriterPort protocol and delegates to the
    concrete infrastructure writer in `infrastructure.io.xpt_write`.

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
