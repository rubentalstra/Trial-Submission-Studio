"""Dataset-XML writer adapter.

This module provides an adapter implementation for writing Dataset-XML files.
It conforms to the DatasetXMLWriterPort protocol.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import pandas as pd

if TYPE_CHECKING:
    from cdisc_transpiler.domain.entities.mapping import MappingConfig

from .dataset_xml.writer import write_dataset_xml


class DatasetXMLWriter:
    """Adapter for writing Dataset-XML files.

    This class implements the DatasetXMLWriterPort protocol and delegates to
    the concrete infrastructure writer in `infrastructure.io.dataset_xml`.

    Example:
        >>> writer = DatasetXMLWriter()
        >>> df = pd.DataFrame({"STUDYID": ["001"], "USUBJID": ["001-001"]})
        >>> writer.write(df, "DM", config, Path("output/dm.xml"))
    """

    def write(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        config: MappingConfig,
        output_path: Path,
    ) -> None:
        """Write a DataFrame to a Dataset-XML file.

        Args:
            dataframe: Data to write
            domain_code: SDTM domain code (e.g., "DM", "AE")
            config: Mapping configuration with column metadata
            output_path: Path where XML file should be written

        Raises:
            Exception: If writing fails

        Example:
            >>> writer = DatasetXMLWriter()
            >>> df = pd.DataFrame({"STUDYID": ["001"]})
            >>> writer.write(df, "DM", config, Path("dm.xml"))
        """
        write_dataset_xml(dataframe, domain_code, config, output_path)
