"""Domain processor for Trial Summary (TS) domain."""

from typing import TYPE_CHECKING, override

if TYPE_CHECKING:
    import pandas as pd

from cdisc_transpiler.pandas_utils import ensure_series

from .base import BaseDomainProcessor


class TSProcessor(BaseDomainProcessor):
    """Trial Summary domain processor.

    Handles domain-specific processing for the TS domain.
    """

    @override
    def process(self, frame: pd.DataFrame) -> None:
        """Process TS domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)

        if frame.empty:
            return

        # Conservative cleanup only: strip strings and blank invalid CT values.
        for col in (
            "STUDYID",
            "DOMAIN",
            "TSPARMCD",
            "TSPARM",
            "TSVAL",
            "TSVALCD",
            "TSVCDREF",
            "TSVCDVER",
            "TSGRPID",
            "TSVALNF",
        ):
            if col in frame.columns:
                s = ensure_series(frame[col]).astype("string")
                s = ensure_series(s.fillna(""))
                frame.loc[:, col] = s.str.strip()

        ct_parmcd = self._get_controlled_terminology(variable="TSPARMCD")
        if ct_parmcd and "TSPARMCD" in frame.columns:
            s = ensure_series(frame["TSPARMCD"]).astype("string")
            s = ensure_series(s.fillna(""))
            raw = s.str.strip()
            canonical = ensure_series(raw.apply(ct_parmcd.normalize))
            valid = canonical.isin(ct_parmcd.submission_values)
            frame.loc[:, "TSPARMCD"] = canonical.where(valid, "")

        ct_parm = self._get_controlled_terminology(variable="TSPARM")
        if ct_parm and "TSPARM" in frame.columns:
            s = ensure_series(frame["TSPARM"]).astype("string")
            s = ensure_series(s.fillna(""))
            raw = s.str.strip()
            canonical = ensure_series(raw.apply(ct_parm.normalize))
            valid = canonical.isin(ct_parm.submission_values)
            frame.loc[:, "TSPARM"] = canonical.where(valid, "")

        ct_dict = self._get_controlled_terminology(variable="TSVCDREF")
        if ct_dict and "TSVCDREF" in frame.columns:
            s = ensure_series(frame["TSVCDREF"]).astype("string")
            s = ensure_series(s.fillna(""))
            raw = s.str.strip()
            canonical = ensure_series(raw.apply(ct_dict.normalize))
            valid = canonical.isin(ct_dict.submission_values)
            frame.loc[:, "TSVCDREF"] = canonical.where(valid, "")
