"""SUPPQUAL (Supplemental Qualifiers) service.

This module provides domain services for building SUPPQUAL DataFrames
for non-model columns in parent SDTM domains.

SDTM Reference:
    Supplemental Qualifiers (SUPPQUAL) are used to capture additional
    information that cannot fit into the standard SDTM domain structure.
    Each SUPPQUAL record provides:
    - RDOMAIN: The parent domain code
    - IDVAR/IDVARVAL: Reference to the parent record
    - QNAM: Qualifier variable name (8 chars max)
    - QVAL: Qualifier value
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

import pandas as pd

from ...constants import Constraints, Defaults
from ...pandas_utils import ensure_numeric_series, ensure_series

if TYPE_CHECKING:
    from ...domain.entities.sdtm_domain import SDTMDomain
    from ..entities.mapping import MappingConfig


def _drop_missing_usubjid(frame: pd.DataFrame) -> pd.DataFrame:
    """Drop rows with missing/blank USUBJID to align with domain processing.

    Args:
        frame: DataFrame to filter

    Returns:
        DataFrame with missing USUBJID rows removed
    """
    if "USUBJID" not in frame.columns:
        return frame
    series = frame["USUBJID"].astype("string")
    mask = series.isna() | series.str.strip().eq("")
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
    # Keep formatting stable; enforce final max-length using SUPP domain metadata
    # during `finalize_suppqual`.
    return numeric.astype(str)


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
    threshold = max(
        Defaults.OPERATIONAL_COLUMN_MIN_COUNT,
        int(total_files * Defaults.OPERATIONAL_COLUMN_COMMON_FRACTION),
    )
    return count >= threshold


def sanitize_qnam(name: str) -> str:
    """Convert a source column into a SAS-safe QNAM (<=8 chars, alnum/underscore).

    Per SDTM, QNAM must be:
    - Maximum 8 characters
    - Alphanumeric or underscore only
    - Cannot start with a number

    Args:
        name: Source column name

    Returns:
        SAS-safe QNAM string

    Example:
        >>> sanitize_qnam("PatientAge")
        'PATIENTA'
        >>> sanitize_qnam("123Value")
        'Q123VALU'
    """
    safe = "".join(ch if ch.isalnum() else "_" for ch in name.upper())
    while "__" in safe:
        safe = safe.replace("__", "_")
    safe = safe.strip("_")
    if not safe:
        safe = "QVAL"
    if safe[0].isdigit():
        safe = f"Q{safe}"
    return safe[: Constraints.QNAM_MAX_LENGTH]


def _get_variable_lengths(domain_def: SDTMDomain) -> dict[str, int]:
    lengths: dict[str, int] = {}
    for var in getattr(domain_def, "variables", []) or []:
        var_name = getattr(var, "name", None)
        var_length = getattr(var, "length", None)
        if not var_name or var_length is None:
            continue
        try:
            length_int = int(var_length)
        except Exception:
            continue
        if length_int > 0:
            lengths[str(var_name).upper()] = length_int
    return lengths


def build_suppqual(
    domain_code: str,
    source_df: pd.DataFrame,
    mapped_df: pd.DataFrame | None,
    domain_def: SDTMDomain,
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
        domain_def: SDTMDomain definition for the parent domain
        used_source_columns: Set of source columns already mapped
        study_id: Study identifier for STUDYID column
        common_column_counts: Count of column appearances across files
        total_files: Total number of input files

    Returns:
        Tuple of (supp_df, used_columns):
        - supp_df: SUPPQUAL DataFrame or None if no qualifiers found
        - used_columns: Set of source columns included in SUPPQUAL
    """
    if source_df.empty:
        return None, set()

    used_source_columns = used_source_columns or set()
    domain = domain_code.upper()
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
    max_len = min(len(aligned_source), len(mapped_df))
    aligned_source = aligned_source.iloc[:max_len]
    idvals = idvals.iloc[:max_len]

    records: list[dict[str, Any]] = []
    for col in extra_cols:
        series = aligned_source[col].astype("string").fillna("").str.strip()
        if series.eq("").all():
            continue
        for pos, val in enumerate(series.to_list()):
            if val == "":
                continue
            idval = idvals.iloc[pos]
            idvar_val = (
                str(_clean_idvarval(pd.Series([idval]), id_is_seq).iloc[0])
                if idvar
                else ""
            )
            usubjid = (
                str(aligned_source.iloc[pos]["USUBJID"])
                if "USUBJID" in aligned_source.columns
                else ""
            )
            if (
                not usubjid or usubjid.strip() == ""
            ) and "USUBJID" in mapped_df.columns:
                usubjid = str(mapped_df.iloc[pos]["USUBJID"])
            records.append(
                {
                    "STUDYID": (
                        str(aligned_source.iloc[pos]["STUDYID"])
                        if "STUDYID" in aligned_source.columns
                        and str(aligned_source.iloc[pos]["STUDYID"]).strip() != ""
                        else (study_id or "")
                    ),
                    "RDOMAIN": domain,
                    "USUBJID": usubjid,
                    "IDVAR": idvar or "",
                    "IDVARVAL": idvar_val,
                    "QNAM": sanitize_qnam(col),
                    "QLABEL": str(col),
                    "QVAL": str(val),
                    "QORIG": "CRF",
                    "QEVAL": "",
                }
            )

    if not records:
        return None, set()

    supp_df = pd.DataFrame(records)

    return supp_df, set(extra_cols)


def finalize_suppqual(
    supp_df: pd.DataFrame,
    supp_domain_def: SDTMDomain | None = None,
    parent_domain_code: str = "DM",
) -> pd.DataFrame:
    """Finalize a SUPPQUAL DataFrame with proper ordering and deduplication.

    Args:
        supp_df: SUPPQUAL DataFrame to finalize
        supp_domain_def: SDTMDomain definition for the SUPP domain (optional)
        parent_domain_code: Parent domain code for QVAL length handling

    Returns:
        Finalized SUPPQUAL DataFrame
    """
    result = supp_df.copy()

    # Reorder columns based on domain definition if available
    if supp_domain_def is not None:
        try:
            ordering = list(supp_domain_def.variable_names())
            result = result.reindex(columns=ordering)
        except Exception:
            pass

        # Enforce SDTM/SAS lengths based on the SUPP domain definition.
        # This keeps length constraints centralized in metadata rather than
        # hard-coded slices in the builder.
        lengths = _get_variable_lengths(supp_domain_def)
        for col in result.columns:
            max_len = lengths.get(str(col).upper())
            if not max_len:
                continue
            result.loc[:, col] = result[col].astype("string").str.slice(0, max_len)

    # As a safety net, ensure QLABEL does not exceed the XPT label constraint.
    if "QLABEL" in result.columns:
        result.loc[:, "QLABEL"] = (
            result["QLABEL"]
            .astype("string")
            .str.slice(0, Constraints.XPT_MAX_LABEL_LENGTH)
        )

    # Deduplicate
    result.drop_duplicates(
        subset=["STUDYID", "USUBJID", "IDVAR", "IDVARVAL", "QNAM"], inplace=True
    )
    result.sort_values(by=["USUBJID", "IDVARVAL"], inplace=True)

    return result


def extract_used_columns(config: MappingConfig | None) -> set[str]:
    """Extract the set of source columns used in a mapping configuration.

    This helper extracts column names from mapping configurations to identify
    which source columns have already been mapped to SDTM variables.

    Args:
        config: Mapping configuration with column mappings

    Returns:
        Set of source column names that are used in the configuration
    """
    from .mapping.utils import unquote_column_name

    used_columns: set[str] = set()
    if config and config.mappings:
        for mapping in config.mappings:
            used_columns.add(unquote_column_name(mapping.source_column))
            if getattr(mapping, "use_code_column", None):
                used_columns.add(unquote_column_name(mapping.use_code_column))
    return used_columns
