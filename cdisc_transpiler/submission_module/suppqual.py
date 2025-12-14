"""SUPPQUAL (Supplemental Qualifiers) building utilities.

This module provides functions for building SUPPQUAL DataFrames
for non-model columns in parent SDTM domains.
"""

from __future__ import annotations

import pandas as pd

from ..pandas_utils import ensure_numeric_series, ensure_series

from ..domains_module import get_domain

# Markers that indicate missing/null values
_MISSING_MARKERS = {"", "NAN", "<NA>", "NONE", "NULL"}


def _drop_missing_usubjid(frame: pd.DataFrame) -> pd.DataFrame:
    """Drop rows with missing/blank USUBJID to align with domain processing.

    Args:
        frame: DataFrame to filter

    Returns:
        DataFrame with missing USUBJID rows removed
    """
    if "USUBJID" not in frame.columns:
        return frame
    mask = (
        frame["USUBJID"].isna()
        | frame["USUBJID"].astype("string").str.strip().str.upper().isin(_MISSING_MARKERS)
    )
    return frame.loc[~mask].reset_index(drop=True)


def _clean_idvarval(values: pd.Series, is_seq: bool) -> pd.Series:
    """Clean IDVARVAL values based on whether they are sequence numbers.

    Args:
        values: Series of IDVARVAL values
        is_seq: True if values are sequence numbers

    Returns:
        Cleaned series
    """
    series: pd.Series = ensure_series(values)
    if not is_seq:
        return series.astype("string")
    numeric = ensure_numeric_series(series).astype("Int64")
    return (
        numeric.astype(str)
        .str.slice(0, 8)
    )


def _is_operational_column(
    name: str,
    *,
    common_counts: dict[str, int] | None = None,
    total_files: int | None = None,
) -> bool:
    """Heuristic: treat columns that appear in many input files as operational.

    Args:
        name: Column name
        common_counts: Dictionary of column name counts across files
        total_files: Total number of input files

    Returns:
        True if column appears to be operational (not domain-specific data)
    """
    if not common_counts or not total_files:
        return False
    norm = name.strip().lower()
    count = common_counts.get(norm, 0)
    threshold = max(3, total_files // 2)
    return count >= threshold


def _sanitize_qnam(name: str) -> str:
    """Convert a source column into a SAS-safe QNAM (<=8 chars, alnum/underscore).

    Per SDTM, QNAM must be:
    - Maximum 8 characters
    - Alphanumeric or underscore only
    - Cannot start with a number

    Args:
        name: Source column name

    Returns:
        SAS-safe QNAM string
    """
    safe = "".join(ch if ch.isalnum() else "_" for ch in name.upper())
    while "__" in safe:
        safe = safe.replace("__", "_")
    safe = safe.strip("_")
    if not safe:
        safe = "QVAL"
    if safe[0].isdigit():
        safe = f"Q{safe}"
    return safe[:8]


def build_suppqual(
    domain_code: str,
    source_df: pd.DataFrame,
    mapped_df: pd.DataFrame | None,
    used_source_columns: set[str] | None = None,
    *,
    study_id: str | None = None,
    common_column_counts: dict[str, int] | None = None,
    total_files: int | None = None,
) -> tuple[pd.DataFrame | None, set[str]]:
    """Build a SUPP-- DataFrame for non-model columns in a parent domain.

    Creates a SUPPQUAL domain containing supplemental qualifier data for
    columns that exist in the source data but are not part of the SDTM
    domain model.

    Args:
        domain_code: Two-character domain code (e.g., "DM", "AE")
        source_df: Source DataFrame with raw data
        mapped_df: Mapped DataFrame with processed domain data
        used_source_columns: Set of source columns already mapped
        study_id: Study identifier for STUDYID column
        common_column_counts: Count of column appearances across files
        total_files: Total number of input files

    Returns:
        Tuple of (supp_df, used_columns):
        - supp_df: SUPPQUAL DataFrame or None if no qualifiers found
        - used_columns: Set of source columns included in SUPPQUAL
    """
    if source_df is None or source_df.empty:
        return None, set()

    used_source_columns = used_source_columns or set()
    domain = domain_code.upper()
    domain_def = get_domain(domain)
    core_vars = set(domain_def.variable_names())

    # Drop rows with missing USUBJID to mirror domain processing
    aligned_source = _drop_missing_usubjid(source_df.copy())
    if aligned_source.empty:
        return None, set()

    # Identify non-model, non-mapped columns
    extra_cols = [
        col
        for col in aligned_source.columns
        if col not in used_source_columns
        and col.upper() not in core_vars
        and not _is_operational_column(
            str(col),
            common_counts=common_column_counts,
            total_files=total_files,
        )
    ]
    if not extra_cols:
        return None, set()

    if mapped_df is None:
        return None, set()
    mapped_cols = set(mapped_df.columns)
    seq_var = f"{domain}SEQ"
    if seq_var in mapped_cols:
        idvar: str | None = seq_var
        idvals = mapped_df[seq_var]
        id_is_seq = True
    elif "USUBJID" in mapped_cols:
        # For domains without a sequence variable, leave IDVAR/IDVARVAL blank
        idvar = None
        idvals = mapped_df["USUBJID"]
        id_is_seq = False
    else:
        return None, set()

    # Ensure lengths align; if not, align by index up to min length
    max_len = min(len(aligned_source), len(mapped_df)) if mapped_df is not None else len(aligned_source)
    aligned_source = aligned_source.iloc[:max_len]
    if mapped_df is not None:
        idvals = idvals.iloc[:max_len]

    records: list[dict] = []
    for col in extra_cols:
        series = aligned_source[col].astype("string").fillna("").str.strip()
        if series.eq("").all():
            continue
        for idx, val in series.items():
            if val == "":
                continue
            idval = idvals.iloc[idx] if mapped_df is not None else ""
            idvar_val = (
                str(_clean_idvarval(pd.Series([idval]), id_is_seq).iloc[0])
                if idvar
                else ""
            )
            usubjid = (
                str(aligned_source.loc[idx, "USUBJID"])
                if "USUBJID" in aligned_source.columns
                else ""
            )
            if (not usubjid or usubjid.strip() == "") and mapped_df is not None and "USUBJID" in mapped_df.columns:
                usubjid = str(mapped_df.iloc[idx]["USUBJID"])
            records.append(
                {
                    "STUDYID": (
                        str(aligned_source.loc[idx, "STUDYID"])
                        if "STUDYID" in aligned_source.columns
                        and str(aligned_source.loc[idx, "STUDYID"]).strip() != ""
                        else (study_id or "")
                    ),
                    "RDOMAIN": domain,
                    "USUBJID": usubjid,
                    "IDVAR": idvar or "",
                    "IDVARVAL": idvar_val,
                    "QNAM": _sanitize_qnam(col),
                    "QLABEL": _sanitize_qnam(col),
                    "QVAL": str(val)[:200],
                    "QORIG": "CRF",
                    "QEVAL": "",
                }
            )

    if not records:
        return None, set()

    supp_df = pd.DataFrame(records)
    
    # Shrink QVAL width to actual max to avoid SD1082; keep reasonable cap
    if "QVAL" in supp_df.columns:
        supp_df["QVAL"] = supp_df["QVAL"].astype(str)
        max_qval_len = supp_df["QVAL"].str.len().max() if not supp_df.empty else 1
        max_qval_len = max(1, min(int(max_qval_len or 1), 50))
        supp_df["QVAL"] = supp_df["QVAL"].str.slice(0, max_qval_len)
    
    supp_domain_code = f"SUPP{domain}"
    try:
        ordering = list(get_domain(supp_domain_code).variable_names())
        supp_df = supp_df.reindex(columns=ordering)
    except Exception:
        pass

    supp_df.drop_duplicates(
        subset=["STUDYID", "USUBJID", "IDVAR", "IDVARVAL", "QNAM"], inplace=True
    )
    supp_df.sort_values(by=["USUBJID", "IDVARVAL"], inplace=True)
    
    # Keep QVAL within metadata length for SUPPDM to avoid SD1082
    if domain.upper() == "DM" and "QVAL" in supp_df.columns:
        try:
            qval_len = next(
                var.length
                for var in get_domain("SUPPDM").variables
                if var.name.upper() == "QVAL"
            )
        except Exception:
            qval_len = 200
        supp_df["QVAL"] = (
            supp_df["QVAL"].astype(str).str.slice(0, qval_len)
        )

    return supp_df, set(extra_cols)
