"""Domain processor for Medical History (MH) domain."""

import pandas as pd

from ....pandas_utils import ensure_numeric_series
from ..transformers import DateTransformer, NumericTransformer
from .base import BaseDomainProcessor


class MHProcessor(BaseDomainProcessor):
    """Medical History domain processor.

    Handles domain-specific processing for the MH domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process MH domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Ensure USUBJID is populated - derive from source if present
        if "USUBJID" in frame.columns:
            usub = frame["USUBJID"].astype("string").str.strip()
            missing_usubjid = usub.str.lower().isin({"", "nan", "<na>", "none", "null"})
            if missing_usubjid.any():
                frame.drop(index=frame.index[missing_usubjid].to_list(), inplace=True)
                frame.reset_index(drop=True, inplace=True)
        if "MHSEQ" not in frame.columns:
            frame.loc[:, "MHSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        else:
            # Always regenerate MHSEQ - source values may not be unique (SD0005)
            frame.loc[:, "MHSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "MHSEQ"] = NumericTransformer.force_numeric(frame["MHSEQ"])
        # MHTERM is required; if it's missing but MHDECOD is present, derive it.
        if "MHTERM" in frame.columns:
            frame.loc[:, "MHTERM"] = (
                frame["MHTERM"].astype("string").fillna("").str.strip()
            )
            empty_mhterm = frame["MHTERM"].astype("string").fillna("").str.strip() == ""
            if bool(empty_mhterm.any()) and "MHDECOD" in frame.columns:
                mhdecod = frame["MHDECOD"].astype("string").fillna("").str.strip()
                frame.loc[empty_mhterm & (mhdecod != ""), "MHTERM"] = mhdecod

            # Drop rows where we still have no topic term.
            still_empty = frame["MHTERM"].astype("string").fillna("").str.strip() == ""
            if bool(still_empty.any()):
                frame.drop(index=frame.index[still_empty], inplace=True)
                frame.reset_index(drop=True, inplace=True)
        for col in ("MHSTDTC", "MHENDTC", "MHDTC"):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].apply(DateTransformer.coerce_iso8601)

        # Normalize end-relative-to-reference-period values to CDISC submission
        # values (CT C66728). Some sources provide boolean-like ongoing flags.
        if "MHENRF" in frame.columns:
            mh_enrf = frame["MHENRF"].astype("string").fillna("").str.strip()
            upper = mh_enrf.str.upper()
            mapped = upper.replace(
                {
                    # Boolean-like / ongoing indicators
                    "Y": "ONGOING",
                    "YES": "ONGOING",
                    "TRUE": "ONGOING",
                    "1": "ONGOING",
                    "N": "",
                    "NO": "",
                    "FALSE": "",
                    "0": "",
                    # Common synonyms → submission values
                    "PRIOR": "BEFORE",
                    "POST": "AFTER",
                    "CONCURRENT": "COINCIDENT",
                    "UNK": "UNKNOWN",
                    "U": "UNKNOWN",
                }
            )
            frame.loc[:, "MHENRF"] = mapped
        # Compute study day for MHDTC into MHDY.
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
