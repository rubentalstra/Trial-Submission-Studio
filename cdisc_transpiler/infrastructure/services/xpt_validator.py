"""XPT-oriented DataFrame validation/cleanup.

This module is infrastructure-layer because it encodes output-format-specific
constraints (SAS XPORT/XPT expectations).

It is designed to be injected into the domain frame builder (domain layer)
as a collaborator.
"""

from typing import TYPE_CHECKING

import pandas as pd

if TYPE_CHECKING:
    from collections.abc import Sequence

    from ...domain.entities.sdtm_domain import SDTMVariable


class XPTValidator:
    # Some variables are marked Permissible in SDTMIG but are treated as
    # regulatory-expected by common validators (e.g., Pinnacle 21) or are
    # required for conditional presence rules (e.g., EXCAT with EXSCAT).
    _KEEP_IF_EMPTY: set[str] = {
        "EPOCH",
        "EXCAT",
        "EXENDY",
    }

    def drop_empty_optional_columns(
        self, frame: pd.DataFrame, variables: Sequence[SDTMVariable]
    ) -> None:
        # Keep Required and Expected variables even when empty.
        # Only drop fully-empty Permissible (or unspecified core) variables.
        droppable = {
            v.name
            for v in variables
            if v.name in frame.columns
            and v.name not in self._KEEP_IF_EMPTY
            and (v.core or "").strip().lower() not in ("req", "exp")
        }

        to_drop: list[str] = []
        for name in droppable:
            series = frame[name]
            if pd.api.types.is_string_dtype(series) or series.dtype == object:
                empty = series.isna() | series.astype("string").fillna(
                    ""
                ).str.strip().eq("")
                if bool(empty.all()):
                    to_drop.append(name)
            elif bool(series.isna().all()):
                to_drop.append(name)

        if to_drop:
            frame.drop(columns=to_drop, inplace=True)

    def reorder_columns(
        self, frame: pd.DataFrame, variables: Sequence[SDTMVariable]
    ) -> None:
        ordered = [v.name for v in variables if v.name in frame.columns]

        # Many validators (including Pinnacle 21) expect the standard SDTM
        # identifier columns to appear first, regardless of spec CSV ordering.
        # Enforce a canonical prefix to prevent domain-spec anomalies from
        # producing surprising column orders.
        canonical_prefix = [
            c for c in ("STUDYID", "DOMAIN", "USUBJID") if c in frame.columns
        ]

        # Prefer domain sequence variables early (typically immediately after USUBJID).
        seq_cols = [
            c
            for c in ordered
            if c.upper().endswith("SEQ") and c not in canonical_prefix
        ]

        prefix = canonical_prefix + [c for c in seq_cols if c not in canonical_prefix]
        ordered = prefix + [c for c in ordered if c not in prefix]
        extras = [c for c in frame.columns if c not in ordered]
        desired = ordered + extras

        if list(frame.columns) == desired:
            return

        reordered = frame.reindex(columns=desired)
        frame.drop(columns=list(frame.columns), inplace=True)
        for col in reordered.columns:
            frame.loc[:, col] = reordered[col]

    def enforce_required_values(
        self,
        _frame: pd.DataFrame,
        _variables: Sequence[SDTMVariable],
        _lenient: bool,
    ) -> None:
        # Intentionally minimal for now: true SDTMIG “required populated”
        # enforcement is data-dependent and should be implemented as a
        # deterministic conformance report rather than hard-filling defaults.
        #
        # The pipeline controls strictness via `lenient`; we only keep the hook.
        return

    def enforce_lengths(
        self, frame: pd.DataFrame, variables: Sequence[SDTMVariable]
    ) -> None:
        for var in variables:
            if var.name not in frame.columns:
                continue
            if var.type != "Char":
                continue
            max_len = int(var.length) if var.length else None
            if not max_len or max_len <= 0:
                continue

            series = frame[var.name].astype("string")
            frame.loc[:, var.name] = series.str.slice(0, max_len)
