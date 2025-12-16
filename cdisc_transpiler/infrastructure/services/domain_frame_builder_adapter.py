"""Infrastructure adapter for building SDTM domain dataframes."""

from __future__ import annotations

import pandas as pd

from ...application.ports import DomainFrameBuilderPort
from ...domain.entities.mapping import MappingConfig
from ...domain.entities.sdtm_domain import SDTMDomain
from ...domain.entities.study_metadata import StudyMetadata
from ...domain.services.domain_frame_builder import build_domain_dataframe
from ...domain.services.domain_processors import get_domain_processor
from ...domain.services.transformers import (
    CodelistTransformer,
    DateTransformer,
    NumericTransformer,
)

from .xpt_validator import XPTValidator


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
        validators = {"xpt": XPTValidator()} if not lenient else None
        transformers = {
            "date": DateTransformer,
            "codelist": CodelistTransformer,
            "numeric": NumericTransformer,
        }

        def domain_processor_factory(
            dom: SDTMDomain,
            ref_starts: dict[str, str] | None,
            meta: StudyMetadata | None,
        ):
            return get_domain_processor(dom, ref_starts, meta)

        return build_domain_dataframe(
            frame,
            config,
            domain,
            reference_starts=reference_starts,
            lenient=lenient,
            metadata=metadata,
            domain_processor_factory=domain_processor_factory,
            transformers=transformers,
            validators=validators,
        )
