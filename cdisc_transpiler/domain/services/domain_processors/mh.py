from typing import TYPE_CHECKING, override

if TYPE_CHECKING:
    import pandas as pd
from ....pandas_utils import ensure_numeric_series
from ..transformers.date import DateTransformer
from ..transformers.numeric import NumericTransformer
from .base import BaseDomainProcessor


class MHProcessor(BaseDomainProcessor):
    pass

    @override
    def process(self, frame: pd.DataFrame) -> None:
        self._drop_placeholder_rows(frame)
        if "USUBJID" in frame.columns:
            usub = frame["USUBJID"].astype("string").str.strip()
            missing_usubjid = usub.str.lower().isin({"", "nan", "<na>", "none", "null"})
            if missing_usubjid.any():
                frame.drop(index=frame.index[missing_usubjid].to_list(), inplace=True)
                frame.reset_index(drop=True, inplace=True)
        if "MHSEQ" not in frame.columns:
            frame.loc[:, "MHSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        else:
            frame.loc[:, "MHSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "MHSEQ"] = NumericTransformer.force_numeric(frame["MHSEQ"])
        if "MHTERM" in frame.columns:
            frame.loc[:, "MHTERM"] = (
                frame["MHTERM"].astype("string").fillna("").str.strip()
            )
            empty_mhterm = frame["MHTERM"].astype("string").fillna("").str.strip() == ""
            if bool(empty_mhterm.any()) and "MHDECOD" in frame.columns:
                mhdecod = frame["MHDECOD"].astype("string").fillna("").str.strip()
                frame.loc[empty_mhterm & (mhdecod != ""), "MHTERM"] = mhdecod
            still_empty = frame["MHTERM"].astype("string").fillna("").str.strip() == ""
            if bool(still_empty.any()):
                frame.drop(index=frame.index[still_empty], inplace=True)
                frame.reset_index(drop=True, inplace=True)
        for col in ("MHSTDTC", "MHENDTC", "MHDTC"):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].apply(DateTransformer.coerce_iso8601)
        if "MHENRF" in frame.columns:
            mh_enrf = frame["MHENRF"].astype("string").fillna("").str.strip()
            upper = mh_enrf.str.upper()
            mapped = upper.replace(
                {
                    "Y": "ONGOING",
                    "YES": "ONGOING",
                    "TRUE": "ONGOING",
                    "1": "ONGOING",
                    "N": "",
                    "NO": "",
                    "FALSE": "",
                    "0": "",
                    "PRIOR": "BEFORE",
                    "POST": "AFTER",
                    "CONCURRENT": "COINCIDENT",
                    "UNK": "UNKNOWN",
                    "U": "UNKNOWN",
                }
            )
            frame.loc[:, "MHENRF"] = mapped
        if "MHDTC" in frame.columns and "MHDY" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "MHDTC",
                "MHDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
            frame.loc[:, "MHDY"] = ensure_numeric_series(
                frame["MHDY"], frame.index
            ).astype("float64")
        NumericTransformer.assign_sequence(frame, "MHSEQ", "USUBJID")
