"""Infrastructure adapter for SUPPQUAL operations."""

from typing import TYPE_CHECKING, override

from ...application.ports.services import SuppqualPort
from ...domain.services.suppqual_service import (
    build_suppqual,
    extract_used_columns,
    finalize_suppqual,
)

if TYPE_CHECKING:
    import pandas as pd

    from ...domain.entities.mapping import MappingConfig
    from ...domain.entities.sdtm_domain import SDTMDomain
    from ...domain.services.suppqual_service import SuppqualBuildRequest


class SuppqualServiceAdapter(SuppqualPort):
    @override
    def extract_used_columns(self, config: MappingConfig | None) -> set[str]:
        return extract_used_columns(config)

    @override
    def build_suppqual(
        self, request: SuppqualBuildRequest
    ) -> tuple[pd.DataFrame | None, set[str]]:
        return build_suppqual(request)

    @override
    def finalize_suppqual(
        self,
        supp_df: pd.DataFrame,
        *,
        supp_domain_def: SDTMDomain | None = None,
    ) -> pd.DataFrame:
        return finalize_suppqual(
            supp_df,
            supp_domain_def=supp_domain_def,
        )
