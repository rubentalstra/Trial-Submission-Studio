"""Infrastructure adapter for building SDTM domain dataframes."""

from __future__ import annotations

import pandas as pd

from ...application.ports import DomainFrameBuilderPort
from ...domain.entities.mapping import MappingConfig
from ...domain.entities.sdtm_domain import SDTMDomain
from ...domain.entities.study_metadata import StudyMetadata
from ...domain.services.domain_frame_builder import build_domain_dataframe


class DomainFrameBuilderAdapter(DomainFrameBuilderPort):
    def build_domain_dataframe(
        self,
        frame: pd.DataFrame,
        config: MappingConfig,
        domain: SDTMDomain,
        *,
        reference_starts: dict[str, str] | None = None,
        lenient: bool = False,
        metadata: StudyMetadata | None = None,
    ) -> pd.DataFrame:
        return build_domain_dataframe(
            frame,
            config,
            domain,
            reference_starts=reference_starts,
            lenient=lenient,
            metadata=metadata,
        )
