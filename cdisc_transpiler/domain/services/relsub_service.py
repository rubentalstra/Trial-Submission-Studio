"""RELSUB (Related Subjects) service.

SDTMIG v3.4 Section 8, Representing Relationships and Data, describes RELSUB
as a standard way to represent relationships between subjects (or between a
pooled identifier and a subject).

This service is intentionally conservative:
- RELSUB relationships are usually not inferable from the raw domain datasets
  without explicit relationship inputs.
- Therefore, by default it produces an empty, correctly structured scaffold.

The application layer is responsible for shaping the dataset using domain
metadata (via DomainFrameBuilder) and generating output files.
"""

import pandas as pd

from ..entities.mapping import ColumnMapping, MappingConfig


class RelsubService:
    """Service for building RELSUB relationship records."""

    _REL_SUB_COLUMNS: tuple[str, ...] = (
        "STUDYID",
        "USUBJID",
        "POOLID",
        "RSUBJID",
        "SREL",
    )

    def build_relsub(
        self,
        *,
        domain_dataframes: dict[str, pd.DataFrame] | None = None,
        study_id: str,
    ) -> tuple[pd.DataFrame, MappingConfig]:
        """Build RELSUB dataframe and mapping config.

        Args:
            domain_dataframes: Optional domain dataframes (reserved for future
                inference strategies; currently unused).
            study_id: Study identifier

        Returns:
            Tuple of (RELSUB dataframe, mapping config)
        """
        _ = domain_dataframes  # reserved for future use

        df = pd.DataFrame(
            {col: pd.Series(dtype="string") for col in self._REL_SUB_COLUMNS}
        )
        df.loc[:, "STUDYID"] = df["STUDYID"].astype("string")

        mappings = [
            ColumnMapping(
                source_column=col,
                target_variable=col,
                transformation=None,
                confidence_score=1.0,
            )
            for col in df.columns
        ]
        config = MappingConfig(domain="RELSUB", study_id=study_id, mappings=mappings)

        return df, config
