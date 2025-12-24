from dataclasses import dataclass
from typing import TYPE_CHECKING, cast

import pandas as pd

from ..entities.mapping import ColumnMapping, MappingConfig

if TYPE_CHECKING:
    from collections.abc import Iterable, Mapping


@dataclass(frozen=True, slots=True)
class _RelspecColumnContext:
    refid_col: str
    spec_col: str | None
    parent_col: str | None
    study_id: str


class RelspecService:
    _REL_SPEC_COLUMNS: tuple[str, ...] = (
        "STUDYID",
        "USUBJID",
        "REFID",
        "SPEC",
        "PARENT",
        "LEVEL",
    )

    def build_relspec(
        self, *, domain_dataframes: dict[str, pd.DataFrame], study_id: str
    ) -> tuple[pd.DataFrame, MappingConfig]:
        records = self._build_relspec_records(domain_dataframes, study_id)
        df = pd.DataFrame(records)
        if df.empty:
            df = pd.DataFrame(
                {col: pd.Series(dtype="string") for col in self._REL_SPEC_COLUMNS}
            )
        else:
            for col in ("STUDYID", "USUBJID", "REFID", "SPEC", "PARENT"):
                if col in df.columns:
                    df.loc[:, col] = df[col].astype("string")
            if "LEVEL" in df.columns:
                df.loc[:, "LEVEL"] = pd.to_numeric(df["LEVEL"], errors="coerce")
            df = df.reindex(columns=list(self._REL_SPEC_COLUMNS))
        mappings = [
            ColumnMapping(
                source_column=col,
                target_variable=col,
                transformation=None,
                confidence_score=1.0,
            )
            for col in df.columns
        ]
        config = MappingConfig(domain="RELSPEC", study_id=study_id, mappings=mappings)
        return (df, config)

    def _build_relspec_records(
        self, domain_dataframes: Mapping[str, pd.DataFrame | None], study_id: str
    ) -> list[dict[str, object]]:
        records: dict[tuple[str, str], dict[str, object]] = {}
        for df in domain_dataframes.values():
            self._collect_relspec_records(records, df, study_id)
        return list(records.values())

    def _collect_relspec_records(
        self,
        records: dict[tuple[str, str], dict[str, object]],
        df: pd.DataFrame | None,
        study_id: str,
    ) -> None:
        if df is None or df.empty:
            return
        if "USUBJID" not in df.columns:
            return
        refid_cols = self._find_refid_columns(df.columns)
        if not refid_cols:
            return
        spec_col = self._pick_first_matching_column(df.columns, suffix="SPEC")
        parent_col = self._pick_first_matching_column(df.columns, exact="PARENT")
        for refid_col in refid_cols:
            work = self._prepare_refid_frame(df, refid_col, spec_col, parent_col)
            if work is None:
                continue
            context = _RelspecColumnContext(
                refid_col=refid_col,
                spec_col=spec_col,
                parent_col=parent_col,
                study_id=study_id,
            )
            self._ingest_relspec_rows(records, work, context)

    def _prepare_refid_frame(
        self,
        df: pd.DataFrame,
        refid_col: str,
        spec_col: str | None,
        parent_col: str | None,
    ) -> pd.DataFrame | None:
        subset_cols: list[str] = ["USUBJID", refid_col]
        if spec_col:
            subset_cols.append(spec_col)
        if parent_col:
            subset_cols.append(parent_col)
        work = df[subset_cols].copy()
        work.loc[:, "USUBJID"] = work["USUBJID"].astype("string")
        work.loc[:, refid_col] = work[refid_col].astype("string")
        work.loc[:, "USUBJID"] = work["USUBJID"].fillna("").str.strip()
        work.loc[:, refid_col] = work[refid_col].fillna("").str.strip()
        work = work[(work["USUBJID"] != "") & (work[refid_col] != "")]
        if work.empty:
            return None
        return work

    def _ingest_relspec_rows(
        self,
        records: dict[tuple[str, str], dict[str, object]],
        work: pd.DataFrame,
        context: _RelspecColumnContext,
    ) -> None:
        for _, row in work.iterrows():
            usubjid = str(row.get("USUBJID", "")).strip()
            refid = str(row.get(context.refid_col, "")).strip()
            if not usubjid or not refid:
                continue
            key = (usubjid, refid)
            record = records.get(key)
            if record is None:
                record = cast(
                    "dict[str, object]",
                    {
                        "STUDYID": context.study_id,
                        "USUBJID": usubjid,
                        "REFID": refid,
                        "SPEC": "",
                        "PARENT": "",
                        "LEVEL": 1,
                    },
                )
                records[key] = record
            if context.spec_col and (not record.get("SPEC")):
                spec = str(row.get(context.spec_col, "") or "").strip()
                if spec:
                    record["SPEC"] = spec
            if context.parent_col and (not record.get("PARENT")):
                parent = str(row.get(context.parent_col, "") or "").strip()
                if parent:
                    record["PARENT"] = parent

    def _find_refid_columns(self, columns: Iterable[object]) -> list[str]:
        found: list[str] = []
        for col in columns:
            name = str(col)
            upper = name.upper()
            if upper == "REFID" or upper.endswith("REFID"):
                found.append(name)
        return found

    def _pick_first_matching_column(
        self,
        columns: Iterable[object],
        *,
        suffix: str | None = None,
        exact: str | None = None,
    ) -> str | None:
        cols = [str(c) for c in columns]
        if exact is not None:
            exact_u = exact.upper()
            for c in cols:
                if c.upper() == exact_u:
                    return c
            return None
        if suffix is not None:
            suffix_u = suffix.upper()
            for c in cols:
                if c.upper().endswith(suffix_u):
                    return c
        return None
