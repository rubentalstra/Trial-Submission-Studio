from contextlib import suppress
from dataclasses import dataclass
from typing import TYPE_CHECKING

import pandas as pd

from ...constants import Constraints, Defaults
from ...pandas_utils import ensure_numeric_series, ensure_series
from .mapping.utils import unquote_column_name

if TYPE_CHECKING:
    from ...domain.entities.sdtm_domain import SDTMDomain
    from ..entities.mapping import MappingConfig


@dataclass(slots=True)
class SuppqualBuildRequest:
    domain_code: str
    source_df: pd.DataFrame
    mapped_df: pd.DataFrame | None
    domain_def: SDTMDomain
    used_source_columns: set[str] | None = None
    study_id: str | None = None
    common_column_counts: dict[str, int] | None = None
    total_files: int | None = None


@dataclass(slots=True)
class _SuppqualRecordContext:
    request: SuppqualBuildRequest
    aligned_source: pd.DataFrame
    mapped_df: pd.DataFrame
    idvar: str | None
    idvals: pd.Series
    id_is_seq: bool


def _drop_missing_usubjid(frame: pd.DataFrame) -> pd.DataFrame:
    if "USUBJID" not in frame.columns:
        return frame
    series = frame["USUBJID"].astype("string")
    mask = series.isna() | series.str.strip().eq("")
    return frame.loc[~mask].reset_index(drop=True)


def _clean_idvarval(values: pd.Series, is_seq: bool) -> pd.Series:
    series: pd.Series = ensure_series(values)
    if not is_seq:
        return series.astype("string")
    numeric = ensure_numeric_series(series).astype("Int64")
    return numeric.astype(str)


def _is_operational_column(
    name: str,
    *,
    common_counts: dict[str, int] | None = None,
    total_files: int | None = None,
) -> bool:
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
        except (TypeError, ValueError):
            length_int = 0
        if length_int > 0:
            lengths[str(var_name).upper()] = length_int
    return lengths


def _select_idvar(
    mapped_df: pd.DataFrame, *, domain: str
) -> tuple[str | None, pd.Series, bool] | None:
    mapped_cols = set(mapped_df.columns)
    seq_var = f"{domain}SEQ"
    if seq_var in mapped_cols:
        return (seq_var, mapped_df[seq_var], True)
    if "USUBJID" in mapped_cols:
        return (None, mapped_df["USUBJID"], False)
    return None


def _align_by_length(
    source: pd.DataFrame, idvals: pd.Series, mapped_df: pd.DataFrame
) -> tuple[pd.DataFrame, pd.Series]:
    max_len = min(len(source), len(mapped_df))
    return (source.iloc[:max_len], idvals.iloc[:max_len])


def _extra_suppqual_columns(
    request: SuppqualBuildRequest, aligned_source: pd.DataFrame
) -> list[str]:
    used_source_columns = request.used_source_columns or set()
    core_vars = set(request.domain_def.variable_names())
    return [
        col
        for col in aligned_source.columns
        if col not in used_source_columns
        and col.upper() not in core_vars
        and (
            not _is_operational_column(
                str(col),
                common_counts=request.common_column_counts,
                total_files=request.total_files,
            )
        )
    ]


def _build_suppqual_records(
    context: _SuppqualRecordContext, *, extra_cols: list[str]
) -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    aligned_source = context.aligned_source
    for col in extra_cols:
        series = aligned_source[col].astype("string").fillna("").str.strip()
        if series.eq("").all():
            continue
        records.extend(
            _build_column_records(context, col=col, series_values=series.to_list())
        )
    return records


def _build_column_records(
    context: _SuppqualRecordContext, *, col: str, series_values: list[str]
) -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    request = context.request
    domain = request.domain_code.upper()
    aligned_source = context.aligned_source
    mapped_df = context.mapped_df
    idvar = context.idvar
    idvals = context.idvals
    id_is_seq = context.id_is_seq
    for pos, val in enumerate(series_values):
        if val == "":
            continue
        idval = idvals.iloc[pos]
        idvar_val = (
            str(_clean_idvarval(pd.Series([idval]), id_is_seq).iloc[0]) if idvar else ""
        )
        usubjid = (
            str(aligned_source.iloc[pos]["USUBJID"])
            if "USUBJID" in aligned_source.columns
            else ""
        )
        if (not usubjid or usubjid.strip() == "") and "USUBJID" in mapped_df.columns:
            usubjid = str(mapped_df.iloc[pos]["USUBJID"])
        studyid = ""
        if "STUDYID" in aligned_source.columns:
            studyid = str(aligned_source.iloc[pos]["STUDYID"]).strip()
        if not studyid:
            studyid = request.study_id or ""
        records.append(
            {
                "STUDYID": studyid,
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
    return records


def build_suppqual(
    request: SuppqualBuildRequest,
) -> tuple[pd.DataFrame | None, set[str]]:
    supp_df: pd.DataFrame | None = None
    used_columns: set[str] = set()
    if not request.source_df.empty:
        aligned_source = _drop_missing_usubjid(request.source_df.copy())
        if not aligned_source.empty:
            extra_cols = _extra_suppqual_columns(request, aligned_source)
            if extra_cols and request.mapped_df is not None:
                selection = _select_idvar(
                    request.mapped_df, domain=request.domain_code.upper()
                )
                if selection is not None:
                    idvar, idvals, id_is_seq = selection
                    aligned_source, idvals = _align_by_length(
                        aligned_source, idvals, request.mapped_df
                    )
                    context = _SuppqualRecordContext(
                        request=request,
                        aligned_source=aligned_source,
                        mapped_df=request.mapped_df,
                        idvar=idvar,
                        idvals=idvals,
                        id_is_seq=id_is_seq,
                    )
                    records = _build_suppqual_records(context, extra_cols=extra_cols)
                    if records:
                        supp_df = pd.DataFrame(records)
                        used_columns = set(extra_cols)
    return (supp_df, used_columns)


def finalize_suppqual(
    supp_df: pd.DataFrame, supp_domain_def: SDTMDomain | None = None
) -> pd.DataFrame:
    result = supp_df.copy()
    if supp_domain_def is not None:
        with suppress(Exception):
            ordering = list(supp_domain_def.variable_names())
            result = result.reindex(columns=ordering)
        lengths = _get_variable_lengths(supp_domain_def)
        for col in result.columns:
            max_len = lengths.get(str(col).upper())
            if not max_len:
                continue
            result.loc[:, col] = result[col].astype("string").str.slice(0, max_len)
    if "QLABEL" in result.columns:
        result.loc[:, "QLABEL"] = (
            result["QLABEL"]
            .astype("string")
            .str.slice(0, Constraints.XPT_MAX_LABEL_LENGTH)
        )
    result.drop_duplicates(
        subset=["STUDYID", "USUBJID", "IDVAR", "IDVARVAL", "QNAM"], inplace=True
    )
    result.sort_values(by=["USUBJID", "IDVARVAL"], inplace=True)
    return result


def extract_used_columns(config: MappingConfig | None) -> set[str]:
    used_columns: set[str] = set()
    if config and config.mappings:
        for mapping in config.mappings:
            used_columns.add(unquote_column_name(mapping.source_column))
            if getattr(mapping, "use_code_column", None):
                used_columns.add(unquote_column_name(mapping.use_code_column))
    return used_columns
