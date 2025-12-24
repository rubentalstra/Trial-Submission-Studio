from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, Any, override

import pandas as pd

if TYPE_CHECKING:
    from collections.abc import Callable

    from ...entities.controlled_terminology import ControlledTerminology
    from ...entities.sdtm_domain import SDTMDomain
    from ...entities.study_metadata import StudyMetadata
from ..transformers.text import TextTransformer


class BaseDomainProcessor(ABC):
    pass

    def __init__(
        self,
        domain: SDTMDomain,
        reference_starts: dict[str, str] | None = None,
        metadata: StudyMetadata | None = None,
        ct_resolver: Callable[[str | None, str | None], ControlledTerminology | None]
        | None = None,
    ) -> None:
        super().__init__()
        self.domain = domain
        self.reference_starts = reference_starts or {}
        self.metadata = metadata
        self._ct_resolver = ct_resolver
        self.config: Any | None = None

    def _get_controlled_terminology(
        self, *, codelist_code: str | None = None, variable: str | None = None
    ) -> ControlledTerminology | None:
        if self._ct_resolver is None:
            return None
        resolved_code = codelist_code
        if not resolved_code and variable:
            var_upper = variable.strip().upper()
            for var_def in getattr(self.domain, "variables", []) or []:
                if getattr(var_def, "name", "").strip().upper() == var_upper:
                    resolved_code = getattr(var_def, "codelist_code", None)
                    break
        return self._ct_resolver(resolved_code, variable)

    @abstractmethod
    def process(self, frame: pd.DataFrame) -> None: ...

    def _drop_placeholder_rows(self, frame: pd.DataFrame) -> None:
        if "USUBJID" in frame.columns:
            usubjid = frame["USUBJID"].astype("string").fillna("").str.strip()
            missing_ids = usubjid.str.upper().isin({"", "NAN", "<NA>", "NONE", "NULL"})
            if missing_ids.any():
                studyid = (
                    frame["STUDYID"].astype("string").fillna("").str.strip()
                    if "STUDYID" in frame.columns
                    else pd.Series([""] * len(frame), index=frame.index, dtype="string")
                )
                if "SUBJID" in frame.columns:
                    subjid = frame["SUBJID"].astype("string").fillna("").str.strip()
                elif "SubjectId" in frame.columns:
                    subjid = frame["SubjectId"].astype("string").fillna("").str.strip()
                elif "SUBJECTID" in frame.columns:
                    subjid = frame["SUBJECTID"].astype("string").fillna("").str.strip()
                else:
                    subjid = pd.Series(
                        [""] * len(frame), index=frame.index, dtype="string"
                    )
                placeholder_subjid = subjid.str.upper().isin(
                    {"SUBJID", "SUBJECTID", "SUBJECT ID"}
                )
                can_fill = missing_ids & ~placeholder_subjid & (subjid != "")
                derived = studyid.where(studyid != "", "")
                derived = (derived + "-" + subjid).where(
                    derived != "-" + subjid, subjid
                )
                usubjid = usubjid.where(~can_fill, derived)
                frame.loc[:, "USUBJID"] = usubjid
                missing_ids = (
                    frame["USUBJID"]
                    .astype("string")
                    .fillna("")
                    .str.strip()
                    .str.upper()
                    .isin({"", "NAN", "<NA>", "NONE", "NULL"})
                )
            if missing_ids.any():
                frame.drop(index=frame.index[missing_ids].to_list(), inplace=True)
                frame.reset_index(drop=True, inplace=True)
            study_id = ""
            if "STUDYID" in frame.columns:
                study_series = frame["STUDYID"].astype("string").fillna("").str.strip()
                study_id = next((v for v in study_series.tolist() if v), "")
            if not study_id and self.config is not None:
                study_id = str(getattr(self.config, "study_id", "") or "").strip()
            if study_id:
                prefix = f"{study_id}-"
                current = frame["USUBJID"].astype("string").fillna("").str.strip()
                needs_prefix = (current != "") & ~current.str.startswith(prefix)
                if bool(needs_prefix.any()):
                    frame.loc[needs_prefix, "USUBJID"] = (
                        prefix + current.loc[needs_prefix]
                    )

    def _replace_frame_preserving_schema(
        self, frame: pd.DataFrame, replacement: pd.DataFrame
    ) -> None:
        original_columns = list(frame.columns)
        original_dtypes = {col: frame[col].dtype for col in original_columns}
        normalized = replacement.copy()
        for col in original_columns:
            if col not in normalized.columns:
                normalized[col] = pd.NA
        normalized = normalized.reindex(columns=original_columns)
        for col, dtype in original_dtypes.items():
            series = normalized[col]
            if pd.api.types.is_string_dtype(dtype):
                normalized.loc[:, col] = series.astype("string")
            elif pd.api.types.is_numeric_dtype(dtype):
                normalized.loc[:, col] = pd.to_numeric(series, errors="coerce").astype(
                    "float64"
                )
            else:
                normalized.loc[:, col] = series
        frame.drop(index=frame.index.tolist(), inplace=True)
        frame.drop(columns=list(frame.columns), inplace=True)
        for col in original_columns:
            frame[col] = normalized[col].values


class DefaultDomainProcessor(BaseDomainProcessor):
    pass

    @override
    def process(self, frame: pd.DataFrame) -> None:
        self._drop_placeholder_rows(frame)
        if "EPOCH" in frame.columns:
            frame.loc[:, "EPOCH"] = TextTransformer.replace_unknown(frame["EPOCH"], "")
