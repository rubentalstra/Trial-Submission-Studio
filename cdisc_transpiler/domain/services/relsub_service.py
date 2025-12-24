import pandas as pd

from ..entities.mapping import ColumnMapping, MappingConfig


class RelsubService:
    _REL_SUB_COLUMNS: tuple[str, ...] = (
        "STUDYID",
        "USUBJID",
        "POOLID",
        "RSUBJID",
        "SREL",
    )

    def build_relsub(
        self, *, domain_dataframes: dict[str, pd.DataFrame] | None = None, study_id: str
    ) -> tuple[pd.DataFrame, MappingConfig]:
        _ = domain_dataframes
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
        return (df, config)
