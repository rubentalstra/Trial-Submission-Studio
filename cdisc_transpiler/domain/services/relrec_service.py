"""RELREC (Related Records) Service.

This service builds RELREC domain records that describe relationships between
records for a subject within or across domains.

SDTMIG Reference:
    SDTMIG v3.4 Section 8.2.1, Related Records (RELREC).

Notes:
    Per SDTMIG, relationships are expressed using STUDYID, RDOMAIN, USUBJID and
    an identifying key (IDVAR/IDVARVAL) plus a relationship identifier (RELID).
    RELTYPE is used only for dataset-to-dataset relationships (Section 8.3).
"""

import math
from typing import Any

import pandas as pd

from ..entities.mapping import ColumnMapping, MappingConfig


class RelrecService:
    """Service for building RELREC relationship records.

        This service contains the logic for creating relationship records that
        link observations across SDTM domains.

                To avoid domain-specific hardcoding while remaining spec-aligned, the
                service:

                - Treats a domain as *eligible* only if it has ``USUBJID`` and at least
                    one identifying variable suitable for IDVAR/IDVARVAL (typically
                    ``{DOMAIN}SEQ``; otherwise any ``*SEQ``).
                - Selects a reference domain dynamically (the one covering most subjects).
                - Emits one relationship (RELID) per eligible record in a non-reference
                    domain, relating that record to the reference domain's per-subject
                    minimum sequence record.

    The service is pure domain logic with no dependencies on infrastructure.
    """

    def build_relrec(
        self,
        domain_dataframes: dict[str, pd.DataFrame],
        study_id: str,
    ) -> tuple[pd.DataFrame, MappingConfig]:
        """Build RELREC dataframe and config from processed domain data.

        Args:
            domain_dataframes: Dictionary mapping domain codes to their dataframes
            study_id: Study identifier

        Returns:
            Tuple of (RELREC dataframe, mapping config)
        """
        records = self._build_relrec_records(domain_dataframes, study_id)

        # Create dataframe
        if not records:
            # Return empty dataframe with proper structure
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

        # Create mapping config
        mappings = [
            ColumnMapping(
                source_column=col,
                target_variable=col,
                transformation=None,
                confidence_score=1.0,
            )
            for col in df.columns
        ]

        config = MappingConfig(
            domain="RELREC",
            study_id=study_id,
            mappings=mappings,
        )

        return df, config

    def _build_relrec_records(
        self,
        domain_dataframes: dict[str, pd.DataFrame],
        study_id: str,
    ) -> list[dict[str, Any]]:
        """Build RELREC records linking eligible domains by subject.

        Args:
            domain_dataframes: Dictionary mapping domain codes to their dataframes
            study_id: Study identifier

        Returns:
            List of RELREC record dictionaries
        """
        records: list[dict[str, Any]] = []

        eligible = self._get_eligible_domains(domain_dataframes)
        if not eligible:
            return records

        reference_domain = self._pick_reference_domain(eligible)
        ref_info = eligible[reference_domain]
        ref_df = ref_info["df"]
        ref_idvar = ref_info["idvar"]
        ref_seq_map = self._build_seq_map(ref_df, ref_idvar)

        # Link each non-reference domain to the reference domain by USUBJID.
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
                    study_id,
                    domain_code,
                    usubjid,
                    idvar,
                    idvarval,
                    relid,
                )

                ref_seq = ref_seq_map.get(usubjid)
                if ref_seq is not None:
                    self._add_record(
                        records,
                        study_id,
                        reference_domain,
                        usubjid,
                        ref_idvar,
                        self._stringify(ref_seq, 1),
                        relid,
                    )

        # Fallback: if only one eligible domain exists, create self-only
        # relationships for the reference domain.
        if not records:
            for usubjid, seq in ref_seq_map.items():
                relid = f"{reference_domain}_ONLY_{usubjid}"
                self._add_record(
                    records,
                    study_id,
                    reference_domain,
                    str(usubjid),
                    ref_idvar,
                    self._stringify(seq, 1),
                    relid,
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
        """Infer an identifying variable suitable for RELREC IDVAR.

        SDTMIG 3.4 Section 8.2.1 describes using a unique record identifier
        such as --SEQ, or a grouping identifier such as --GRPID.

        Prefer the domain's canonical sequence variable (e.g. AESEQ). If absent,
        fall back to any other *SEQ (excluding plain SEQ). If still absent,
        fall back to any *GRPID.
        """
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
        # Pick the domain that covers the most distinct subjects.
        scores: list[tuple[int, str]] = []
        for domain_code, info in eligible.items():
            df = info["df"]
            idvar = info["idvar"]
            subject_map = self._build_seq_map(df, idvar)
            scores.append((len(subject_map), domain_code))

        # Deterministic: most subjects, then lexicographic.
        scores.sort(key=lambda item: (-item[0], item[1]))
        return scores[0][1]

    def _get_domain_df(
        self,
        domain_dataframes: dict[str, pd.DataFrame],
        domain_code: str,
    ) -> pd.DataFrame | None:
        """Get domain dataframe by code.

        Args:
            domain_dataframes: Dictionary of domain dataframes
            domain_code: Domain code to retrieve

        Returns:
            Domain dataframe or None if not found or empty
        """
        for code, df in domain_dataframes.items():
            if code.upper() == domain_code.upper():
                if not df.empty:
                    return df
        return None

    def _build_seq_map(
        self,
        df: pd.DataFrame,
        seq_col: str,
    ) -> dict[str, Any]:
        """Build mapping from USUBJID to minimum sequence number.

        Args:
            df: Domain dataframe
            seq_col: Sequence column name (e.g., "DSSEQ", "AESEQ")

        Returns:
            Dictionary mapping USUBJID to minimum sequence number
        """
        if seq_col not in df.columns or "USUBJID" not in df.columns:
            return {}

        numeric = pd.to_numeric(df[seq_col], errors="coerce")
        return (
            pd.DataFrame({"USUBJID": df["USUBJID"], seq_col: numeric})
            .dropna(subset=["USUBJID", seq_col])
            .groupby("USUBJID")[seq_col]
            .min()
            .to_dict()
        )

    def _stringify(self, val: Any, fallback_index: int) -> str:
        """Convert value to string, using fallback index if invalid.

        Args:
            val: Value to stringify
            fallback_index: Fallback integer to use if value is invalid

        Returns:
            String representation of value
        """
        # Handle pandas Series/Index
        if isinstance(val, pd.Series):
            val = val.iloc[0] if not val.empty else None
        elif isinstance(val, pd.Index):
            val = val[0] if len(val) else None

        # Handle None/NaN
        if val is None:
            return str(fallback_index)

        # Try to convert to number
        try:
            num_f = float(str(val))
            if math.isnan(num_f):
                return str(fallback_index)
            return str(int(num_f)) if num_f.is_integer() else str(num_f)
        except Exception:
            return str(val)

    def _add_record(
        self,
        records: list[dict[str, Any]],
        study_id: str,
        rdomain: str,
        usubjid: str,
        idvar: str,
        idvarval: str,
        relid: str,
        reltype: str | None = None,
    ) -> None:
        """Add a relationship record to the list.

        Args:
            records: List to append record to
            study_id: Study identifier
            rdomain: Related domain code
            usubjid: Unique subject identifier
            idvar: ID variable name
            idvarval: ID variable value
            relid: Relationship identifier
            reltype: Relationship type (optional)
        """
        records.append(
            {
                "STUDYID": study_id,
                "RDOMAIN": rdomain,
                "USUBJID": usubjid,
                "IDVAR": idvar,
                "IDVARVAL": idvarval,
                "RELTYPE": reltype or "",
                "RELID": relid,
            }
        )
