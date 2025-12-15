"""Base domain processor for SDTM domain-specific transformations.

This module provides the base class for domain-specific processors that handle
post-processing logic unique to each SDTM domain.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, Any

import pandas as pd

if TYPE_CHECKING:
    from ....domains_module import SDTMDomain
    from ....metadata_module import StudyMetadata

from ....xpt_module.transformers import TextTransformer


class BaseDomainProcessor(ABC):
    """Base class for domain-specific processors.

    Each SDTM domain may have unique processing requirements beyond the standard
    transformations. Domain processors encapsulate this domain-specific logic.
    """

    def __init__(
        self,
        domain: "SDTMDomain",
        reference_starts: dict[str, str] | None = None,
        metadata: "StudyMetadata | None" = None,
    ):
        """Initialize the domain processor.

        Args:
            domain: SDTM domain definition
            reference_starts: Mapping of USUBJID -> RFSTDTC for study day calculations
            metadata: Study metadata (optional)
        """
        self.domain = domain
        self.reference_starts = reference_starts or {}
        self.metadata = metadata
        self.config: Any | None = None

    @abstractmethod
    def process(self, frame: pd.DataFrame) -> None:
        """Process the domain DataFrame in-place.

        This method performs domain-specific transformations, validations,
        and data quality improvements.

        Args:
            frame: Domain DataFrame to process in-place
        """
        pass

    def _drop_placeholder_rows(self, frame: pd.DataFrame) -> None:
        """Drop placeholder/header rows without subject identifiers.

        Args:
            frame: DataFrame to clean in-place
        """
        if "USUBJID" in frame.columns:
            missing_ids = frame["USUBJID"].isna() | frame["USUBJID"].astype(
                "string"
            ).str.strip().str.upper().isin({"", "NAN", "<NA>", "NONE", "NULL"})
            if missing_ids.any():
                frame.drop(index=frame.index[missing_ids].to_list(), inplace=True)
                frame.reset_index(drop=True, inplace=True)


class DefaultDomainProcessor(BaseDomainProcessor):
    """Default processor for domains without specific processing needs."""

    def process(self, frame: pd.DataFrame) -> None:
        """Apply common processing to any domain.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Set default EPOCH if present and empty
        if "EPOCH" in frame.columns:
            frame["EPOCH"] = TextTransformer.replace_unknown(
                frame["EPOCH"], "TREATMENT"
            )
