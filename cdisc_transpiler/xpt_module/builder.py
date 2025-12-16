"""DataFrame construction and orchestration for SDTM domains.

This module provides the core builder class that orchestrates the construction
of SDTM-compliant DataFrames from source data and mapping configurations.

NOTE: This module is a compatibility wrapper. The actual implementation
has been moved to `cdisc_transpiler.domain.services.domain_frame_builder`.

SDTM Reference:
    SDTMIG v3.4 Section 4.1 defines the general structure of SDTM datasets.
    Variables follow the General Observation Classes (Interventions, Events,
    Findings) and include Identifier, Topic, Timing, and Qualifier roles.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd

from ..mapping_module import MappingConfig
from ..domains_module import get_domain
from ..domain.services.domain_frame_builder import (
    DomainFrameBuilder as _DomainFrameBuilder,
)
from .domain_processors import get_domain_processor

# Import the modular components for injection
from .transformers import (
    DateTransformer,
    CodelistTransformer,
    NumericTransformer,
)
from .validators import XPTValidator

if TYPE_CHECKING:
    from ..metadata_module import StudyMetadata


class XportGenerationError(RuntimeError):
    """Raised when XPT export cannot be completed."""


# Re-export for API compatibility
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

    NOTE: This function is a compatibility wrapper. New code should use
    `cdisc_transpiler.domain.services.build_domain_dataframe()` directly.

    Args:
        frame: The source DataFrame.
        config: The mapping configuration.
        reference_starts: Optional mapping of USUBJID -> RFSTDTC for study day calculations.
        lenient: If True, skip validation of required values (useful for Dataset-XML).
        metadata: Optional StudyMetadata for codelist transformations.

    Returns:
        A DataFrame with columns matching the SDTM domain layout.
    """
    builder = DomainFrameBuilder(
        frame,
        config,
        reference_starts=reference_starts,
        lenient=lenient,
        metadata=metadata,
    )
    return builder.build()


class DomainFrameBuilder:
    """Builds SDTM-compliant DataFrames from source data.

    NOTE: This class is a compatibility wrapper. New code should use
    `cdisc_transpiler.domain.services.DomainFrameBuilder` directly.

    This class orchestrates the construction of domain DataFrames by:
    1. Creating a blank DataFrame with domain variables
    2. Applying column mappings from source to target
    3. Performing transformations (dates, codelists, numeric)
    4. Validating and enforcing SDTM requirements
    5. Reordering columns to match domain specification
    """

    def __init__(
        self,
        frame: pd.DataFrame,
        config: MappingConfig,
        *,
        reference_starts: dict[str, str] | None = None,
        lenient: bool = False,
        metadata: "StudyMetadata | None" = None,
    ) -> None:
        # Get domain from config (lookup here for backwards compatibility)
        domain = get_domain(config.domain)

        # Create transformers and validators dict for injection
        transformers = {
            "date": DateTransformer,
            "codelist": CodelistTransformer,
            "numeric": NumericTransformer,
        }
        validators = {
            "xpt": XPTValidator,
        }

        # Delegate to domain service
        self._builder = _DomainFrameBuilder(
            frame,
            config,
            domain,
            reference_starts=reference_starts,
            lenient=lenient,
            metadata=metadata,
            domain_processor_factory=get_domain_processor,
            transformers=transformers,
            validators=validators,
        )

    def build(self) -> pd.DataFrame:
        """Build the domain DataFrame using modular transformers and validators."""
        return self._builder.build()
