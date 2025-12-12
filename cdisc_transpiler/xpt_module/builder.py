"""DataFrame construction and orchestration for SDTM domains.

This module provides the core builder class that orchestrates the construction
of SDTM-compliant DataFrames from source data and mapping configurations.

Note: This is Phase 4 Step 2 - Initial builder extraction. Complex domain-specific
processing is still delegated to the original xpt.py implementation and will be
refactored in subsequent steps (Steps 3-6).
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd

from ..mapping import MappingConfig

# Import from original xpt module - this provides backward compatibility
# while we gradually refactor functionality into the new modular structure
from ..xpt import (
    XportGenerationError,
    _DomainFrameBuilder as _OriginalDomainFrameBuilder,
    build_domain_dataframe as _original_build_domain_dataframe,
)

if TYPE_CHECKING:
    from ..metadata import StudyMetadata


# Re-export the exception for API compatibility
__all__ = ["XportGenerationError", "build_domain_dataframe", "DomainFrameBuilder"]


def build_domain_dataframe(
    frame: pd.DataFrame,
    config: MappingConfig,
    *,
    reference_starts: dict[str, str] | None = None,
    lenient: bool = False,
    metadata: "StudyMetadata | None" = None,
) -> pd.DataFrame:
    """Return a pandas DataFrame that matches the SDTM domain layout.

    Args:
        frame: The source DataFrame.
        config: The mapping configuration.
        reference_starts: Optional mapping of USUBJID -> RFSTDTC for study day calculations.
        lenient: If True, skip validation of required values (useful for Dataset-XML).
        metadata: Optional StudyMetadata for codelist transformations.

    Returns:
        A DataFrame with columns matching the SDTM domain layout.
        
    Note:
        This currently delegates to the original implementation in xpt.py.
        As Phase 4 progresses (Steps 3-6), functionality will be gradually
        migrated to the new modular transformer and validator classes.
    """
    # Delegate to original implementation for now
    # This will be refactored in Steps 3-6 to use new modular components
    return _original_build_domain_dataframe(
        frame,
        config,
        reference_starts=reference_starts,
        lenient=lenient,
        metadata=metadata,
    )


class DomainFrameBuilder(_OriginalDomainFrameBuilder):
    """Builds SDTM-compliant DataFrames from source data.
    
    This class orchestrates the construction of domain DataFrames by:
    1. Creating a blank DataFrame with domain variables
    2. Applying column mappings from source to target
    3. Performing transformations (dates, codelists, numeric)
    4. Validating and enforcing SDTM requirements
    5. Reordering columns to match domain specification
    
    Note:
        This is Phase 4 Step 2 - Currently inherits from the original implementation
        in xpt.py to maintain backward compatibility. In Steps 3-6, functionality will
        be gradually refactored into modular transformer and validator classes:
        - Step 3: Date transformers (transformers/date.py)
        - Step 4: Codelist transformers (transformers/codelist.py)  
        - Step 5: Numeric & text transformers (transformers/numeric.py, text.py)
        - Step 6: Validators (validators.py)
    """
    
    # Inherits all methods from _OriginalDomainFrameBuilder for now
    # Future steps will override methods with modular implementations
    pass
