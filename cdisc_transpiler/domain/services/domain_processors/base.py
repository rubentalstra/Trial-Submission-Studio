"""Base domain processor for SDTM domain-specific transformations.

This module provides the base class for domain-specific processors that handle
post-processing logic unique to each SDTM domain.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from collections.abc import Callable
from typing import TYPE_CHECKING, Any

import pandas as pd

if TYPE_CHECKING:
    from ...entities.sdtm_domain import SDTMDomain
    from ...entities.controlled_terminology import ControlledTerminology
    from ...entities.study_metadata import StudyMetadata

from ..transformers import TextTransformer


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
        ct_resolver: Callable[[str | None, str | None], "ControlledTerminology | None"]
        | None = None,
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
        self._ct_resolver = ct_resolver
        self.config: Any | None = None

    def _get_controlled_terminology(
        self,
        *,
        codelist_code: str | None = None,
        variable: str | None = None,
    ) -> "ControlledTerminology | None":
        if self._ct_resolver is None:
            return None
        return self._ct_resolver(codelist_code, variable)

    @abstractmethod
    def process(self, frame: pd.DataFrame) -> None:
        """Process the domain DataFrame in-place.

        This method performs domain-specific transformations, validations,
        and data quality improvements.

        Args:
            frame: Domain DataFrame to process in-place
        """
        raise NotImplementedError

    def _drop_placeholder_rows(self, frame: pd.DataFrame) -> None:
        """Drop placeholder/header rows without subject identifiers.

        Args:
            frame: DataFrame to clean in-place
        """
        # Some source extracts/mappings omit USUBJID but provide SUBJID and STUDYID,
        # which is sufficient to derive USUBJID. Do this before dropping rows so
        # we don't delete valid subject records.
        if "USUBJID" in frame.columns:
            usubjid = frame["USUBJID"].astype("string").fillna("").str.strip()
            missing_ids = usubjid.str.upper().isin({"", "NAN", "<NA>", "NONE", "NULL"})

            if missing_ids.any():
                studyid = (
                    frame["STUDYID"].astype("string").fillna("").str.strip()
                    if "STUDYID" in frame.columns
                    else pd.Series([""] * len(frame), index=frame.index, dtype="string")
                )
                # Prefer SUBJID when present; fall back to common raw identifiers.
                if "SUBJID" in frame.columns:
                    subjid = frame["SUBJID"].astype("string").fillna("").str.strip()
                elif "SubjectId" in frame.columns:
                    subjid = frame["SubjectId"].astype("string").fillna("").str.strip()
                elif "SUBJECTID" in frame.columns:
                    subjid = frame["SUBJECTID"].astype("string").fillna("").str.strip()
                else:
                    subjid = pd.Series(
                        [""] * len(frame), index=frame.index, dtype="string"
                    )

                # Avoid turning header-placeholder rows into "valid" USUBJIDs.
                placeholder_subjid = subjid.str.upper().isin(
                    {"SUBJID", "SUBJECTID", "SUBJECT ID", "SUBJECTID", "SUBJECTID"}
                )
                can_fill = missing_ids & ~placeholder_subjid & (subjid != "")

                derived = studyid.where(studyid != "", "")
                derived = (derived + "-" + subjid).where(
                    derived != "-" + subjid, subjid
                )
                usubjid = usubjid.where(~can_fill, derived)
                frame.loc[:, "USUBJID"] = usubjid

                # Recompute missing after attempted fill.
                missing_ids = (
                    frame["USUBJID"]
                    .astype("string")
                    .fillna("")
                    .str.strip()
                    .str.upper()
                    .isin({"", "NAN", "<NA>", "NONE", "NULL"})
                )

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
            frame.loc[:, "EPOCH"] = TextTransformer.replace_unknown(
                frame["EPOCH"], "TREATMENT"
            )
