"""Define-XML generator adapter.

This module provides an adapter implementation for generating Define-XML files.
It wraps the existing xml_module.define_module functionality while conforming
to the DefineXmlGeneratorPort protocol.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from typing import Iterable
    from ...xml_module.define_module import StudyDataset


class DefineXmlGenerator:
    """Adapter for generating Define-XML 2.1 files.
    
    This class implements the DefineXmlGeneratorPort protocol by wrapping
    the existing xml_module.define_module.write_study_define_file function.
    It provides a clean interface that can be injected into other components.
    
    Example:
        >>> generator = DefineXmlGenerator()
        >>> datasets = [StudyDataset(...), StudyDataset(...)]
        >>> generator.generate(datasets, Path("define.xml"), sdtm_version="3.4", context="Submission")
    """
    
    def generate(
        self,
        datasets: Iterable[StudyDataset],
        output_path: Path,
        *,
        sdtm_version: str,
        context: str,
    ) -> None:
        """Generate a Define-XML 2.1 file for the given study datasets.
        
        Args:
            datasets: Iterable of StudyDataset objects containing domain metadata
            output_path: Path where Define-XML file should be written
            sdtm_version: SDTM-IG version (e.g., "3.4")
            context: Define-XML context - 'Submission' or 'Other'
            
        Raises:
            Exception: If generation or writing fails (propagated from xml_module)
            
        Example:
            >>> generator = DefineXmlGenerator()
            >>> datasets = [StudyDataset(...)]
            >>> generator.generate(datasets, Path("define.xml"), sdtm_version="3.4", context="Submission")
        """
        # Import at runtime to avoid circular import
        from ...xml_module.define_module import write_study_define_file
        
        write_study_define_file(
            datasets,
            output_path,
            sdtm_version=sdtm_version,
            context=context,
        )
