from typing import TYPE_CHECKING, override

if TYPE_CHECKING:
    import pandas as pd
from .base import BaseDomainProcessor


class TEProcessor(BaseDomainProcessor):
    pass

    @override
    def process(self, frame: pd.DataFrame) -> None:
        self._drop_placeholder_rows(frame)
        for col in ("STUDYID", "DOMAIN", "ETCD", "ELEMENT", "TESTRL", "TEENRL"):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()
