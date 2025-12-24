from typing import override

import pandas as pd

from .base import BaseDomainProcessor


class TAProcessor(BaseDomainProcessor):
    pass

    @override
    def process(self, frame: pd.DataFrame) -> None:
        self._drop_placeholder_rows(frame)
        key_cols = [c for c in ("EPOCH", "ARMCD", "ARM", "ETCD") if c in frame.columns]
        if key_cols and len(frame) > 0:
            is_blank = pd.Series(True, index=frame.index)
            for col in key_cols:
                is_blank &= frame[col].astype("string").fillna("").str.strip().eq("")
            if bool(is_blank.any()):
                frame.drop(index=frame.index[is_blank].to_list(), inplace=True)
                frame.reset_index(drop=True, inplace=True)
        for col in ("EPOCH", "ARMCD", "ARM", "ETCD", "STUDYID", "DOMAIN"):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()
        if "TAETORD" in frame.columns:
            frame.loc[:, "TAETORD"] = pd.to_numeric(frame["TAETORD"], errors="coerce")
        if "TAETORD" in frame.columns:
            frame.sort_values(by=["TAETORD"], kind="stable", inplace=True)
            frame.reset_index(drop=True, inplace=True)
