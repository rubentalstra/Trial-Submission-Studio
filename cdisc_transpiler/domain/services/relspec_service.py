"""RELSPEC (Related Specimens) service.

SDTMIG v3.4 Section 8, Representing Relationships and Data, describes RELSPEC
as a standard way to represent relationships between specimens.

Unlike RELSUB, RELSPEC can often be partially inferred from specimen identifier
usage in other domains (e.g., any variables that implement --REFID).

This service:
- Scans the processed domain datasets for subject-level specimen identifiers
  (any column ending in REFID) alongside USUBJID.
- Produces one RELSPEC record per unique (USUBJID, REFID).

It does not attempt to infer full specimen genealogy (PARENT/LEVEL) unless
source datasets explicitly provide those concepts.
"""

from __future__ import annotations

from collections.abc import Iterable

import pandas as pd

from ..entities.mapping import ColumnMapping, MappingConfig


class RelspecService:
    """Service for building RELSPEC relationship records."""

    _REL_SPEC_COLUMNS: tuple[str, ...] = (
        "STUDYID",
        "USUBJID",
        "REFID",
        "SPEC",
        "PARENT",
        "LEVEL",
    )

    def build_relspec(
        self,
        *,
        domain_dataframes: dict[str, pd.DataFrame],
        study_id: str,
    ) -> tuple[pd.DataFrame, MappingConfig]:
        """Build RELSPEC dataframe and mapping config.

        Args:
            domain_dataframes: Dictionary of domain code -> dataframe
            study_id: Study identifier

        Returns:
            Tuple of (RELSPEC dataframe, mapping config)
        """
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
                # LEVEL is numeric per spec but allow stringy inputs; DomainFrameBuilder will coerce.
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

        return df, config

    def _build_relspec_records(
        self,
        domain_dataframes: dict[str, pd.DataFrame],
        study_id: str,
    ) -> list[dict[str, object]]:
        records: dict[tuple[str, str], dict[str, object]] = {}

        for _domain_code, df in domain_dataframes.items():
            if df is None or df.empty:
                continue
            if "USUBJID" not in df.columns:
                continue

            refid_cols = self._find_refid_columns(df.columns)
            if not refid_cols:
                continue

            spec_col = self._pick_first_matching_column(df.columns, suffix="SPEC")
            parent_col = self._pick_first_matching_column(df.columns, exact="PARENT")

            for refid_col in refid_cols:
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
                    continue

                for _, row in work.iterrows():
                    usubjid = str(row.get("USUBJID", "")).strip()
                    refid = str(row.get(refid_col, "")).strip()
                    if not usubjid or not refid:
                        continue

                    key = (usubjid, refid)
                    record = records.get(key)
                    if record is None:
                        record = {
                            "STUDYID": study_id,
                            "USUBJID": usubjid,
                            "REFID": refid,
                            "SPEC": "",
                            "PARENT": "",
                            "LEVEL": 1,
                        }
                        records[key] = record

                    if spec_col and not record.get("SPEC"):
                        spec = str(row.get(spec_col, "") or "").strip()
                        if spec:
                            record["SPEC"] = spec

                    if parent_col and not record.get("PARENT"):
                        parent = str(row.get(parent_col, "") or "").strip()
                        if parent:
                            record["PARENT"] = parent

        return list(records.values())

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
