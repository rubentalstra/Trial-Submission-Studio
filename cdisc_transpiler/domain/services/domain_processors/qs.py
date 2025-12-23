"""Domain processor for Questionnaires (QS) domain."""

import pandas as pd

from ....pandas_utils import ensure_series
from ..transformers import DateTransformer, NumericTransformer
from .base import BaseDomainProcessor


class QSProcessor(BaseDomainProcessor):
    """Questionnaires domain processor.

    Handles domain-specific processing for the QS domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process QS domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Always regenerate QSSEQ - source values may not be unique (SD0005)
        frame.loc[:, "QSSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "QSSEQ"] = NumericTransformer.force_numeric(frame["QSSEQ"])

        for col in (
            "QSTESTCD",
            "QSTEST",
            "QSCAT",
            "QSSCAT",
            "QSORRES",
            "QSSTRESC",
            "QSLOBXFL",
            "VISIT",
            "EPOCH",
        ):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()

        # If the source provides a PGA score field, populate results and (only if
        # missing) minimal identifying metadata for that instrument.
        #
        # In some real-world extracts, auto-mapping can mis-route the PGA score
        # into identifier-like variables (e.g., QSGRPID). We defensively recover
        # the score when QSORRES is empty and a plausible source score column is
        # present.
        source_score = None
        score_from_qsgrpid = False
        if "QSPGARS" in frame.columns:
            source_score = ensure_series(frame["QSPGARS"], index=frame.index)
        elif "QSPGARSCD" in frame.columns:
            source_score = ensure_series(frame["QSPGARSCD"], index=frame.index)
        elif "QSGRPID" in frame.columns and "QSORRES" in frame.columns:
            qsorres = frame["QSORRES"].astype("string").fillna("").str.strip()
            qsgrpid = frame["QSGRPID"].astype("string").fillna("").str.strip()
            # Treat QSGRPID as a mis-mapped source score only when QSORRES is
            # entirely empty and QSGRPID contains values.
            if bool((qsorres == "").all()) and bool((qsgrpid != "").any()):
                source_score = ensure_series(frame["QSGRPID"], index=frame.index)
                score_from_qsgrpid = True

        if source_score is not None:
            if "QSORRES" in frame.columns:
                empty_orres = (
                    frame["QSORRES"].astype("string").fillna("").str.strip() == ""
                )
                frame.loc[empty_orres, "QSORRES"] = source_score.astype(
                    "string"
                ).fillna("")
            else:
                frame.loc[:, "QSORRES"] = source_score.astype("string").fillna("")

            if score_from_qsgrpid and "QSGRPID" in frame.columns:
                # Clear mis-mapped identifier values once we've moved them into
                # QSORRES.
                frame.loc[:, "QSGRPID"] = ""

            if "QSTESTCD" in frame.columns:
                empty_testcd = (
                    frame["QSTESTCD"].astype("string").fillna("").str.strip() == ""
                )
                # Some mappings accidentally populate QSTESTCD with SITEID-like
                # tokens. If it matches the site segment of USUBJID, treat it as
                # mis-mapped and replace with the PGA short name.
                mis_mapped = pd.Series([False] * len(frame), index=frame.index)
                if "USUBJID" in frame.columns:
                    usubjid = frame["USUBJID"].astype("string").fillna("").str.strip()
                    parts = usubjid.str.split("-", n=2, expand=True)
                    if parts.shape[1] >= 2:
                        site_part = parts[1].astype("string").fillna("").str.strip()
                        testcd = (
                            frame["QSTESTCD"].astype("string").fillna("").str.strip()
                        )
                        mis_mapped = (
                            (testcd != "") & (site_part != "") & (testcd == site_part)
                        )

                frame.loc[empty_testcd | mis_mapped, "QSTESTCD"] = "PGAS"
            if "QSTEST" in frame.columns:
                empty_test = (
                    frame["QSTEST"].astype("string").fillna("").str.strip() == ""
                )
                frame.loc[empty_test, "QSTEST"] = "PHYSICIAN GLOBAL ASSESSMENT"
            if "QSCAT" in frame.columns:
                empty_cat = frame["QSCAT"].astype("string").fillna("").str.strip() == ""
                frame.loc[empty_cat, "QSCAT"] = "PGI"

        if "QSSTRESC" in frame.columns and "QSORRES" in frame.columns:
            empty_stresc = (
                frame["QSSTRESC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_stresc, "QSSTRESC"] = (
                frame.loc[empty_stresc, "QSORRES"].astype("string").fillna("")
            )

        if "QSLOBXFL" in frame.columns:
            frame.loc[:, "QSLOBXFL"] = (
                frame["QSLOBXFL"].astype("string").fillna("").replace("N", "")
            )

        if "QSDTC" in frame.columns:
            frame.loc[:, "QSDTC"] = frame["QSDTC"].apply(DateTransformer.coerce_iso8601)
            if "QSDY" in frame.columns:
                DateTransformer.compute_study_day(
                    frame,
                    "QSDTC",
                    "QSDY",
                    reference_starts=self.reference_starts,
                    ref="RFSTDTC",
                )

        # If timing support variables are absent, clear QSTPTREF to avoid
        # inconsistent partial timing specification.
        if "QSTPTREF" in frame.columns and {"QSELTM", "QSTPTNUM", "QSTPT"}.isdisjoint(
            frame.columns
        ):
            frame.loc[:, "QSTPTREF"] = ""
