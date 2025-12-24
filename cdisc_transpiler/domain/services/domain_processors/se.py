from typing import TYPE_CHECKING, override

if TYPE_CHECKING:
    import pandas as pd
from ..transformers.date import DateTransformer
from ..transformers.numeric import NumericTransformer
from .base import BaseDomainProcessor


class SEProcessor(BaseDomainProcessor):
    pass

    @override
    def process(self, frame: pd.DataFrame) -> None:
        self._drop_placeholder_rows(frame)
        for col in (
            "STUDYID",
            "DOMAIN",
            "USUBJID",
            "ETCD",
            "ELEMENT",
            "EPOCH",
            "SESTDTC",
            "SEENDTC",
        ):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()
        DateTransformer.ensure_date_pair_order(frame, "SESTDTC", "SEENDTC")
        if "SESTDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "SESTDTC",
                "SESTDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        if "SEENDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "SEENDTC",
                "SEENDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        NumericTransformer.assign_sequence(frame, "SESEQ", "USUBJID")
