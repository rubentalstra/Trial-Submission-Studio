"""RELREC (Related Records) Service.

This service builds RELREC domain records that link observations across
different domains (e.g., linking AE records to DS disposition events).

SDTM Reference:
    SDTMIG v3.4 Section 6.4 describes RELREC (Related Records) for linking
    observations across domains.
"""

from __future__ import annotations

import math
from typing import Any

import pandas as pd

from ..entities.mapping import MappingConfig, ColumnMapping


class RelrecService:
    """Service for building RELREC relationship records.

    This service contains the logic for creating relationship records that
    link observations across SDTM domains. It implements the following rules:

    1. Links AE records to DS records by subject
    2. Links EX records to DS records by subject
    3. Creates fallback DS-only relationships if no other linkages exist

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
        """Build RELREC records linking AE/EX records to DS by subject.

        Args:
            domain_dataframes: Dictionary mapping domain codes to their dataframes
            study_id: Study identifier

        Returns:
            List of RELREC record dictionaries
        """
        # Extract relevant domain dataframes
        ae_df = self._get_domain_df(domain_dataframes, "AE")
        ds_df = self._get_domain_df(domain_dataframes, "DS")
        ex_df = self._get_domain_df(domain_dataframes, "EX")

        records: list[dict[str, Any]] = []

        # Build DS sequence map (subject -> min DS sequence number)
        ds_seq_map = self._build_seq_map(ds_df, "DSSEQ") if ds_df is not None else {}

        # Link AE records to DS records
        if ae_df is not None and ds_seq_map:
            for idx, (_, row) in enumerate(ae_df.iterrows(), start=1):
                usubjid = str(row.get("USUBJID", "") or "").strip()
                if not usubjid:
                    continue

                aeseq = self._stringify(row.get("AESEQ"), idx)
                relid = f"AE_DS_{usubjid}_{aeseq}"

                # Add AE record
                self._add_record(
                    records, study_id, "AE", usubjid, "AESEQ", aeseq, relid
                )

                # Add linked DS record if available
                ds_seq = ds_seq_map.get(usubjid)
                if ds_seq is not None:
                    self._add_record(
                        records,
                        study_id,
                        "DS",
                        usubjid,
                        "DSSEQ",
                        self._stringify(ds_seq, 1),
                        relid,
                    )

        # Link EX records to DS records
        if ex_df is not None and ds_seq_map:
            for idx, (_, row) in enumerate(ex_df.iterrows(), start=1):
                usubjid = str(row.get("USUBJID", "") or "").strip()
                if not usubjid:
                    continue

                exseq = self._stringify(row.get("EXSEQ"), idx)
                relid = f"EX_DS_{usubjid}_{exseq}"

                # Add EX record
                self._add_record(
                    records, study_id, "EX", usubjid, "EXSEQ", exseq, relid
                )

                # Add linked DS record if available
                ds_seq = ds_seq_map.get(usubjid)
                if ds_seq is not None:
                    self._add_record(
                        records,
                        study_id,
                        "DS",
                        usubjid,
                        "DSSEQ",
                        self._stringify(ds_seq, 1),
                        relid,
                    )

        # Fallback: if no relationships were created but DS exists,
        # create DS-only relationships
        if not records and ds_df is not None:
            ds_seq_map = self._build_seq_map(ds_df, "DSSEQ")
            for usubjid, ds_seq in ds_seq_map.items():
                relid = f"DS_ONLY_{usubjid}"
                self._add_record(
                    records,
                    study_id,
                    "DS",
                    str(usubjid),
                    "DSSEQ",
                    self._stringify(ds_seq, 1),
                    relid,
                )

        return records

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
                if isinstance(df, pd.DataFrame) and not df.empty:
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
