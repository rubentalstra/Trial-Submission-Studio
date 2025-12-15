"""SAS writer adapter.

This module provides an adapter implementation for generating and writing
SAS programs. It wraps the existing sas_module functionality while conforming
to the SASWriterPort protocol.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ...mapping_module import MappingConfig

# Import the existing SAS generation and writing functions
from ...sas_module import generate_sas_program, write_sas_file


class SASWriter:
    """Adapter for generating and writing SAS programs.
    
    This class implements the SASWriterPort protocol by wrapping the
    existing sas_module.generate_sas_program and sas_module.write_sas_file
    functions. It provides a clean interface that can be injected into
    other components.
    
    Example:
        >>> writer = SASWriter()
        >>> writer.write("DM", config, Path("output/dm.sas"), "work.dm", "sdtm.dm")
    """
    
    def write(
        self,
        domain_code: str,
        config: MappingConfig,
        output_path: Path,
        input_dataset: str | None = None,
        output_dataset: str | None = None,
    ) -> None:
        """Generate and write a SAS program.
        
        Args:
            domain_code: SDTM domain code (e.g., "DM", "AE")
            config: Mapping configuration with column metadata
            output_path: Path where SAS file should be written
            input_dataset: Input dataset name (e.g., "work.dm"), optional
            output_dataset: Output dataset name (e.g., "sdtm.dm"), optional
            
        Raises:
            Exception: If generation or writing fails (propagated from sas_module)
            
        Example:
            >>> writer = SASWriter()
            >>> writer.write("DM", config, Path("dm.sas"), "raw.demo", "final.dm")
        """
        # Generate SAS code
        sas_code = generate_sas_program(
            domain_code,
            config,
            input_dataset=input_dataset,
            output_dataset=output_dataset,
        )
        
        # Write to file
        write_sas_file(sas_code, output_path)
