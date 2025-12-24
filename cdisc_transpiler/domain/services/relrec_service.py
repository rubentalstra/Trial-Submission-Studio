from dataclasses import dataclass
import math
from typing import Any, cast

import pandas as pd

from ..entities.mapping import ColumnMapping, MappingConfig


@dataclass(frozen=True, slots=True)
class _RelrecRecord:
    study_id: str
    rdomain: str
    usubjid: str
    idvar: str
    idvarval: str
    relid: str
    reltype: str | None = None


class RelrecService:
    pass

    def build_relrec(
        self, domain_dataframes: dict[str, pd.DataFrame], study_id: str
    ) -> tuple[pd.DataFrame, MappingConfig]:
        records = self._build_relrec_records(domain_dataframes, study_id)
        if not records:
            df = pd.DataFrame(
                columns=[
                    "STUDYID",
                    "RDOMAIN",
                    "USUBJID",
                    "IDVAR",
                    "IDVARVAL",
                    "RELTYPE",
                    "RELID",
                ]
            )
        else:
            df = pd.DataFrame(records)
        mappings = [
            ColumnMapping(
                source_column=col,
                target_variable=col,
                transformation=None,
                confidence_score=1.0,
            )
            for col in df.columns
        ]
        config = MappingConfig(domain="RELREC", study_id=study_id, mappings=mappings)
        return (df, config)

    def _build_relrec_records(
        self, domain_dataframes: dict[str, pd.DataFrame], study_id: str
    ) -> list[dict[str, Any]]:
        records: list[dict[str, Any]] = []
        eligible = self._get_eligible_domains(domain_dataframes)
        if not eligible:
            return records
        reference_domain = self._pick_reference_domain(eligible)
        ref_info = eligible[reference_domain]
        ref_df = ref_info["df"]
        ref_idvar = ref_info["idvar"]
        ref_seq_map = self._build_seq_map(ref_df, ref_idvar)
        for domain_code, info in eligible.items():
            if domain_code == reference_domain:
                continue
            df = info["df"]
            idvar = info["idvar"]
            for idx, (_, row) in enumerate(df.iterrows(), start=1):
                usubjid_val = row.get("USUBJID", "")
                if pd.isna(usubjid_val):
                    continue
                usubjid = str(usubjid_val).strip()
                if not usubjid:
                    continue
                idvarval = self._stringify(row.get(idvar), idx)
                relid = f"{domain_code}_{reference_domain}_{usubjid}_{idvarval}"
                self._add_record(
                    records,
                    _RelrecRecord(
                        study_id=study_id,
                        rdomain=domain_code,
                        usubjid=usubjid,
                        idvar=idvar,
                        idvarval=idvarval,
                        relid=relid,
                    ),
                )
                ref_seq = ref_seq_map.get(usubjid)
                if ref_seq is not None:
                    self._add_record(
                        records,
                        _RelrecRecord(
                            study_id=study_id,
                            rdomain=reference_domain,
                            usubjid=usubjid,
                            idvar=ref_idvar,
                            idvarval=self._stringify(ref_seq, 1),
                            relid=relid,
                        ),
                    )
        if not records:
            for usubjid, seq in ref_seq_map.items():
                relid = f"{reference_domain}_ONLY_{usubjid}"
                self._add_record(
                    records,
                    _RelrecRecord(
                        study_id=study_id,
                        rdomain=reference_domain,
                        usubjid=str(usubjid),
                        idvar=ref_idvar,
                        idvarval=self._stringify(seq, 1),
                        relid=relid,
                    ),
                )
        return records

    def _get_eligible_domains(
        self, domain_dataframes: dict[str, pd.DataFrame]
    ) -> dict[str, dict[str, Any]]:
        eligible: dict[str, dict[str, Any]] = {}
        for code, df in domain_dataframes.items():
            if df.empty:
                continue
            if "USUBJID" not in df.columns:
                continue
            domain_code = str(code).upper()
            idvar = self._infer_idvar(df, domain_code)
            if idvar is None:
                continue
            eligible[domain_code] = {"df": df, "idvar": idvar}
        return eligible

    def _infer_idvar(self, df: pd.DataFrame, domain_code: str) -> str | None:
        expected = f"{domain_code}SEQ"
        if expected in df.columns:
            return expected
        candidates = [
            str(col)
            for col in df.columns
            if str(col).upper().endswith("SEQ") and str(col).upper() != "SEQ"
        ]
        if candidates:
            return sorted(candidates, key=lambda c: c.upper())[0]
        grp_candidates = [
            str(col)
            for col in df.columns
            if str(col).upper().endswith("GRPID") and str(col).upper() != "GRPID"
        ]
        if grp_candidates:
            return sorted(grp_candidates, key=lambda c: c.upper())[0]
        return None

    def _pick_reference_domain(self, eligible: dict[str, dict[str, Any]]) -> str:
        scores: list[tuple[int, str]] = []
        for domain_code, info in eligible.items():
            df = info["df"]
            idvar = info["idvar"]
            subject_map = self._build_seq_map(df, idvar)
            scores.append((len(subject_map), domain_code))
        scores.sort(key=lambda item: (-item[0], item[1]))
        return scores[0][1]

    def _get_domain_df(
        self, domain_dataframes: dict[str, pd.DataFrame], domain_code: str
    ) -> pd.DataFrame | None:
        for code, df in domain_dataframes.items():
            if code.upper() == domain_code.upper() and (not df.empty):
                return df
        return None

    def _build_seq_map(self, df: pd.DataFrame, seq_col: str) -> dict[str, Any]:
        if seq_col not in df.columns or "USUBJID" not in df.columns:
            return {}
        numeric = pd.to_numeric(df[seq_col], errors="coerce")
        mapped = (
            pd.DataFrame({"USUBJID": df["USUBJID"], seq_col: numeric})
            .dropna(subset=["USUBJID", seq_col])
            .groupby("USUBJID")[seq_col]
            .min()
            .to_dict()
        )
        return {str(key): value for key, value in mapped.items()}

    def _stringify(self, val: object, fallback_index: int) -> str:
        if isinstance(val, pd.Series):
            series = cast("pd.Series[Any]", val)
            val = series.iloc[0] if not series.empty else None
        elif isinstance(val, pd.Index):
            index = cast("pd.Index[Any]", val)
            val = index[0] if len(index) else None
        if val is None:
            return str(fallback_index)
        try:
            num_f = float(str(val))
            if math.isnan(num_f):
                return str(fallback_index)
            return str(int(num_f)) if num_f.is_integer() else str(num_f)
        except Exception:
            return str(val)

    def _add_record(self, records: list[dict[str, Any]], record: _RelrecRecord) -> None:
        records.append(
            {
                "STUDYID": record.study_id,
                "RDOMAIN": record.rdomain,
                "USUBJID": record.usubjid,
                "IDVAR": record.idvar,
                "IDVARVAL": record.idvarval,
                "RELTYPE": record.reltype or "",
                "RELID": record.relid,
            }
        )
