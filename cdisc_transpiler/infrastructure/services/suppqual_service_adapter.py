"""Infrastructure adapter for SUPPQUAL operations."""

from typing import override

import pandas as pd

from ...application.ports.services import SuppqualPort
from ...domain.entities.mapping import MappingConfig
from ...domain.entities.sdtm_domain import SDTMDomain
from ...domain.services.suppqual_service import (
    build_suppqual,
    extract_used_columns,
    finalize_suppqual,
)


class SuppqualServiceAdapter(SuppqualPort):
    @override
    def extract_used_columns(self, config: MappingConfig | None) -> set[str]:
        return extract_used_columns(config)

    @override
    def build_suppqual(
        self,
        domain_code: str,
        source_df: pd.DataFrame,
        mapped_df: pd.DataFrame | None,
        domain_def: SDTMDomain,
        used_source_columns: set[str] | None = None,
        *,
        study_id: str | None = None,
        common_column_counts: dict[str, int] | None = None,
        total_files: int | None = None,
    ) -> tuple[pd.DataFrame | None, set[str]]:
        return build_suppqual(
            domain_code,
            source_df,
            mapped_df,
            domain_def,
            used_source_columns,
            study_id=study_id,
            common_column_counts=common_column_counts,
            total_files=total_files,
        )

    @override
    def finalize_suppqual(
        self,
        supp_df: pd.DataFrame,
        *,
        supp_domain_def: SDTMDomain | None = None,
        parent_domain_code: str = "DM",
    ) -> pd.DataFrame:
        return finalize_suppqual(
            supp_df,
            supp_domain_def=supp_domain_def,
            parent_domain_code=parent_domain_code,
        )
