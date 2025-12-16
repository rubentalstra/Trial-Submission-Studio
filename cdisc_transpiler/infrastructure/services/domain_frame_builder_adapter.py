"""Infrastructure adapter for building SDTM domain dataframes."""

from __future__ import annotations

import pandas as pd

from ...application.ports import DomainFrameBuilderPort
from ...application.ports.repositories import CTRepositoryPort
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
    def __init__(self, *, ct_repository: CTRepositoryPort | None = None) -> None:
        self._ct_repository = ct_repository

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
            ct_repository = self._ct_repository

            def ct_resolver(codelist_code: str | None, variable: str | None):
                if ct_repository is None:
                    return None
                if codelist_code:
                    return ct_repository.get_by_code(codelist_code)
                if variable:
                    return ct_repository.get_by_name(variable)
                return None

            return get_domain_processor(dom, ref_starts, meta, ct_resolver=ct_resolver)

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
