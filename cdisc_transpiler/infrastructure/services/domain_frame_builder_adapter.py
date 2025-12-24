from dataclasses import replace
from typing import TYPE_CHECKING, cast, override

from ...application.ports.services import DomainFrameBuilderPort
from ...domain.services.domain_frame_builder import build_domain_dataframe
from ...domain.services.domain_processors.registry import get_domain_processor
from ...domain.services.transformers.codelist import CodelistTransformer
from ...domain.services.transformers.date import DateTransformer
from ...domain.services.transformers.numeric import NumericTransformer
from .xpt_validator import XPTValidator

if TYPE_CHECKING:
    import pandas as pd

    from ...application.ports.repositories import CTRepositoryPort
    from ...domain.entities.controlled_terminology import ControlledTerminology
    from ...domain.entities.sdtm_domain import SDTMDomain
    from ...domain.entities.study_metadata import StudyMetadata
    from ...domain.services.domain_frame_builder import (
        DomainFrameBuildRequest,
        TransformerRegistry,
        ValidatorRegistry,
    )
    from ...domain.services.domain_processors.base import BaseDomainProcessor


class DomainFrameBuilderAdapter(DomainFrameBuilderPort):
    pass

    def __init__(self, *, ct_repository: CTRepositoryPort | None = None) -> None:
        super().__init__()
        self._ct_repository = ct_repository

    @override
    def build_domain_dataframe(self, request: DomainFrameBuildRequest) -> pd.DataFrame:
        validators: ValidatorRegistry | None = None
        if not request.lenient:
            validators = cast("ValidatorRegistry", {"xpt": XPTValidator()})
        transformers: TransformerRegistry = {
            "date": DateTransformer,
            "codelist": CodelistTransformer,
            "numeric": NumericTransformer,
        }

        def domain_processor_factory(
            dom: SDTMDomain,
            ref_starts: dict[str, str] | None,
            meta: StudyMetadata | None,
        ) -> BaseDomainProcessor:
            ct_repository = self._ct_repository

            def ct_resolver(
                codelist_code: str | None, variable: str | None
            ) -> ControlledTerminology | None:
                if ct_repository is None:
                    return None
                if codelist_code:
                    return ct_repository.get_by_code(codelist_code)
                if variable:
                    return ct_repository.get_by_name(variable)
                return None

            return get_domain_processor(dom, ref_starts, meta, ct_resolver=ct_resolver)

        enriched_request = replace(
            request,
            domain_processor_factory=domain_processor_factory,
            transformers=transformers,
            validators=validators,
        )
        return build_domain_dataframe(enriched_request)
