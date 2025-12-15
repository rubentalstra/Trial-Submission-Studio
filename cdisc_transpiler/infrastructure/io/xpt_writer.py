"""XPT writer adapter.

This module provides an adapter implementation for writing XPT (SAS Transport)
files. It wraps the existing xpt_module functionality while conforming to
the XPTWriterPort protocol.
"""

from __future__ import annotations

from pathlib import Path

import pandas as pd

# Import the existing XPT writing function
from ...xpt_module import write_xpt_file


class XPTWriter:
    """Adapter for writing XPT (SAS Transport) files.
    
    This class implements the XPTWriterPort protocol by wrapping the
    existing xpt_module.write_xpt_file function. It provides a clean
    interface that can be injected into other components.
    
    Example:
        >>> writer = XPTWriter()
        >>> df = pd.DataFrame({"STUDYID": ["001"], "USUBJID": ["001-001"]})
        >>> writer.write(df, "DM", Path("output/dm.xpt"))
    """
    
    def write(self, dataframe: pd.DataFrame, domain_code: str, output_path: Path) -> None:
        """Write a DataFrame to an XPT file.
        
        Args:
            dataframe: Data to write
            domain_code: SDTM domain code (e.g., "DM", "AE")
            output_path: Path where XPT file should be written
            
        Raises:
            Exception: If writing fails (propagated from xpt_module)
            
        Example:
            >>> writer = XPTWriter()
            >>> df = pd.DataFrame({"STUDYID": ["001"]})
            >>> writer.write(df, "DM", Path("dm.xpt"))
        """
        write_xpt_file(dataframe, domain_code, output_path)
