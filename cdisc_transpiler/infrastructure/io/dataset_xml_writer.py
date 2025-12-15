"""Dataset-XML writer adapter.

This module provides an adapter implementation for writing Dataset-XML files.
It wraps the existing xml_module.dataset_module functionality while conforming
to the DatasetXMLWriterPort protocol.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import pandas as pd

if TYPE_CHECKING:
    from ...mapping_module import MappingConfig

# Import the existing Dataset-XML writing function
from ...xml_module.dataset_module import write_dataset_xml


class DatasetXMLWriter:
    """Adapter for writing Dataset-XML files.
    
    This class implements the DatasetXMLWriterPort protocol by wrapping
    the existing xml_module.dataset_module.write_dataset_xml function.
    It provides a clean interface that can be injected into other components.
    
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
            Exception: If writing fails (propagated from xml_module)
            
        Example:
            >>> writer = DatasetXMLWriter()
            >>> df = pd.DataFrame({"STUDYID": ["001"]})
            >>> writer.write(df, "DM", config, Path("dm.xml"))
        """
        write_dataset_xml(dataframe, domain_code, config, output_path)
