"""Domain synthesis service.

This module provides services for synthesizing SDTM domains that are not
present in source data. This includes trial design domains (TS, TA, TE,
SE, DS) and empty observation domains (AE, LB, VS, EX).

SDTM Reference:
    Trial Design domains are defined in SDTMIG v3.4 Section 5.
    Observation class domains are defined in Section 6.

This service is a pure domain service - it returns only domain data
(DataFrames + metadata), with no file I/O or infrastructure concerns.
File generation is handled by the application layer.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Any
from collections.abc import Callable

import pandas as pd

from ..entities.sdtm_domain import SDTMDomain

from .domain_frame_builder import build_domain_dataframe

if TYPE_CHECKING:
    from ..entities.mapping import MappingConfig


@dataclass
class SynthesisResult:
    """Result of domain synthesis (pure domain data).

    This is a pure domain object containing only synthesized data,
    with no file paths or I/O concerns. File generation is handled
    by the application layer.

    Attributes:
        domain_code: SDTM domain code
        records: Number of records in the domain
        domain_dataframe: The synthesized domain DataFrame
        config: Mapping configuration used
        success: Whether synthesis succeeded
        error: Error message if synthesis failed
    """

    domain_code: str
    records: int = 0
    domain_dataframe: pd.DataFrame | None = None
    config: "MappingConfig | None" = None
    success: bool = True
    error: str | None = None

    def to_dict(self) -> dict[str, Any]:
        """Convert to a plain dictionary representation."""
        return {
            "domain_code": self.domain_code,
            "records": self.records,
            "domain_dataframe": self.domain_dataframe,
            "config": self.config,
        }


class SynthesisService:
    """Pure domain service for synthesizing SDTM domains.

    This service creates scaffold domains when source data doesn't include them.
    These domains are required by validation tools like Pinnacle 21 for
    regulatory submission packages.

    This is a pure domain service with no I/O concerns. It returns only
    domain data (DataFrames + configs). File generation is handled by the
    application layer using the FileGeneratorPort.

    Example:
        >>> service = SynthesisService(domain_resolver=my_domain_resolver)
        >>> result = service.synthesize_trial_design(
        ...     domain_code="TS",
        ...     study_id="STUDY001",
        ... )
        >>> if result.success:
        ...     print(f"Generated {result.records} records")
        ...     # Application layer handles file generation
    """

    def __init__(self, *, domain_resolver: Callable[[str], SDTMDomain]):
        self._domain_resolver = domain_resolver

    def synthesize_trial_design(
        self,
        domain_code: str,
        study_id: str,
        reference_starts: dict[str, str] | None = None,
    ) -> SynthesisResult:
        """Synthesize a trial design domain.

        Creates scaffold trial design domains (TS, TA, TE, SE, DS) with
        minimal required data to pass validation.

        Args:
            domain_code: Domain code (TS, TA, TE, SE, DS)
            study_id: Study identifier
            reference_starts: Reference start dates by subject

        Returns:
            SynthesisResult with generated DataFrame and config
        """
        try:
            # Resolve the domain dynamically (no hardcoded fallbacks).
            domain = self._domain_resolver(domain_code)

            # Build a minimal scaffold frame based on the resolved domain
            # variable definitions. Values are intentionally left empty;
            # the SDTM schema comes from `SDTMDomain`.
            frame = self._build_scaffold_frame(domain)

            config = self._build_identity_config(domain_code, frame, study_id)

            # Build SDTM-compliant DataFrame (lenient: allow empty required values)
            domain_dataframe = build_domain_dataframe(
                frame,
                config,
                domain,
                reference_starts=reference_starts,
                lenient=True,
            )

            return SynthesisResult(
                domain_code=domain_code,
                records=len(domain_dataframe),
                domain_dataframe=domain_dataframe,
                config=config,
                success=True,
            )

        except Exception as exc:
            return SynthesisResult(
                domain_code=domain_code,
                success=False,
                error=str(exc),
            )

    def synthesize_observation(
        self,
        domain_code: str,
        study_id: str,
        reference_starts: dict[str, str] | None = None,
    ) -> SynthesisResult:
        """Synthesize an empty observation domain.

        Creates minimal observation domains (AE, LB, VS, EX) with required
        structure but minimal data.

        Args:
            domain_code: Domain code (AE, LB, VS, EX)
            study_id: Study identifier
            reference_starts: Reference start dates by subject

        Returns:
            SynthesisResult with generated DataFrame and config
        """
        try:
            domain = self._domain_resolver(domain_code)
            frame = self._build_scaffold_frame(domain)
            config = self._build_identity_config(domain_code, frame, study_id)
            domain_dataframe = build_domain_dataframe(
                frame,
                config,
                domain,
                reference_starts=reference_starts,
                lenient=True,
            )

            return SynthesisResult(
                domain_code=domain_code,
                records=len(domain_dataframe),
                domain_dataframe=domain_dataframe,
                config=config,
                success=True,
            )

        except Exception as exc:
            return SynthesisResult(
                domain_code=domain_code,
                success=False,
                error=str(exc),
            )

    def _build_scaffold_frame(
        self, domain: SDTMDomain, *, rows: int = 1
    ) -> pd.DataFrame:
        """Return a minimal scaffold DataFrame for a resolved domain.

        The schema comes from the SDTM spec (the resolved `SDTMDomain`). Values
        are intentionally left empty to avoid hardcoded synthesis content.
        """
        return pd.DataFrame(
            {
                var.name: pd.Series([None] * rows, dtype=var.pandas_dtype())
                for var in domain.variables
            }
        )

    def _build_identity_config(
        self, domain_code: str, frame: pd.DataFrame, study_id: str
    ) -> MappingConfig:
        """Build identity mapping configuration."""
        from ..entities.mapping import ColumnMapping, build_config

        mappings = [
            ColumnMapping(
                source_column=col,
                target_variable=col,
                transformation=None,
                confidence_score=1.0,
            )
            for col in frame.columns
        ]
        config = build_config(domain_code, mappings)
        config.study_id = study_id
        return config
