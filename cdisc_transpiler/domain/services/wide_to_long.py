"""Wide-to-long reshaping helpers for SDTM Findings-style source extracts.

These helpers are *domain logic*:
- No filesystem/network I/O
- Pure pandas reshaping + light normalization
"""

from __future__ import annotations

from dataclasses import dataclass

import pandas as pd


@dataclass(frozen=True)
class WideToLongResult:
    frame: pd.DataFrame
    is_long: bool
    consumed_columns: set[str] | None = None


def _make_usubjid(study_id: str, subject_id: object) -> str:
    sid = "" if subject_id is None else str(subject_id).strip()
    if sid == "":
        return ""
    # Common SDTM convention: STUDYID-SUBJECTID
    return f"{study_id}-{sid}" if study_id else sid


def _try_parse_visitnum(event_name: object) -> int | None:
    if event_name is None:
        return None
    text = str(event_name).strip()
    if not text:
        return None
    # Accept "Visit 1" and variants.
    lowered = text.lower()
    if lowered.startswith("visit"):
        parts = lowered.replace("visit", "").strip().split()
        if parts:
            try:
                return int(parts[0])
            except ValueError:
                return None
    return None


def transform_wide_to_long(
    frame: pd.DataFrame, *, domain_code: str, study_id: str
) -> WideToLongResult:
    """Transform a wide extract (multiple tests per row) into SDTM long records.

    Generic transformation that looks for columns starting with 'ORRES_'.
    """
    domain = domain_code.upper()

    # Identify test suffixes from ORRES_ columns
    test_suffixes = set()
    for col in frame.columns:
        if col.startswith("ORRES_"):
            test_suffixes.add(col.replace("ORRES_", "", 1))

    if not test_suffixes:
        return WideToLongResult(frame=frame, is_long=False)

    consumed_columns = set()
    base = pd.DataFrame(index=frame.index)

    # Subject ID
    subject_id_col = "SubjectId" if "SubjectId" in frame.columns else "SUBJECTID"
    if subject_id_col in frame.columns:
        consumed_columns.add(subject_id_col)
        base.loc[:, "USUBJID"] = frame[subject_id_col].map(
            lambda v: _make_usubjid(study_id, v)
        )
    else:
        base.loc[:, "USUBJID"] = ""

    # Visit/Event
    event_name_col = "EventName" if "EventName" in frame.columns else "VISIT"
    if event_name_col in frame.columns:
        consumed_columns.add(event_name_col)
        base.loc[:, "VISIT"] = frame[event_name_col].astype("string").fillna("")
        base.loc[:, "VISITNUM"] = frame[event_name_col].map(_try_parse_visitnum)
    else:
        base.loc[:, "VISIT"] = ""
        base.loc[:, "VISITNUM"] = None

    # Date/Time (Generic search for *DAT and *TIM)
    date_col = None
    time_col = None

    # Try domain specific first
    if f"{domain}DAT" in frame.columns:
        date_col = f"{domain}DAT"
    elif "DAT" in frame.columns:
        date_col = "DAT"
    else:
        for col in frame.columns:
            if col.endswith("DAT") and (col.startswith(domain) or "DAT" in col):
                date_col = col
                break

    if f"{domain}TIM" in frame.columns:
        time_col = f"{domain}TIM"
    elif "TIM" in frame.columns:
        time_col = "TIM"

    if date_col:
        consumed_columns.add(date_col)
        dat_series = frame[date_col].astype("string").fillna("").str.strip()
    else:
        dat_series = pd.Series([""] * len(frame), index=frame.index, dtype="string")

    if time_col:
        consumed_columns.add(time_col)
        tim_series = frame[time_col].astype("string").fillna("").str.strip()
    else:
        tim_series = pd.Series([""] * len(frame), index=frame.index, dtype="string")

    # Construct DTC
    dtc = dat_series.where(dat_series != "", "")
    has_time = (tim_series != "") & (dat_series != "")
    dtc = dtc.where(~has_time, dat_series + "T" + tim_series)
    base.loc[:, f"{domain}DTC"] = dtc

    # Performance/Status
    perf_col = None
    if f"{domain}PERFCD" in frame.columns:
        perf_col = f"{domain}PERFCD"
    elif "PERFCD" in frame.columns:
        perf_col = "PERFCD"
    if not perf_col:
        for col in frame.columns:
            if col.endswith("PERFCD"):
                perf_col = col
                break

    if perf_col:
        consumed_columns.add(perf_col)
        perf = frame[perf_col].astype("string").fillna("").str.upper().str.strip()
        base.loc[:, f"{domain}STAT"] = ""
        base.loc[perf == "N", f"{domain}STAT"] = "NOT DONE"
    else:
        perf = pd.Series([""] * len(frame), index=frame.index, dtype="string")
        base.loc[:, f"{domain}STAT"] = ""

    out_frames: list[pd.DataFrame] = []

    for suffix in sorted(test_suffixes):
        orres_col = f"ORRES_{suffix}"
        if orres_col not in frame.columns:
            continue

        consumed_columns.add(orres_col)

        # Skip empty values
        values = frame[orres_col].astype("string").fillna("").str.strip()
        has_value = values != ""

        # Handle NOT DONE
        if (perf == "N").any():
            has_value = has_value & (perf != "N")

        if not has_value.any():
            continue

        # Related columns
        unit_col = f"ORRESU_{suffix}"
        test_col = f"TEST_{suffix}"
        pos_col = f"POS_{suffix}"

        part = pd.DataFrame(index=frame.index[has_value])
        part.loc[:, "STUDYID"] = study_id
        part.loc[:, "DOMAIN"] = domain
        part.loc[:, "USUBJID"] = base.loc[has_value, "USUBJID"]
        part.loc[:, "VISIT"] = base.loc[has_value, "VISIT"]
        part.loc[:, "VISITNUM"] = base.loc[has_value, "VISITNUM"]
        part.loc[:, f"{domain}DTC"] = base.loc[has_value, f"{domain}DTC"]
        part.loc[:, f"{domain}STAT"] = base.loc[has_value, f"{domain}STAT"]

        # Test Code and Name
        # Use suffix as code
        part.loc[:, f"{domain}TESTCD"] = suffix

        if test_col in frame.columns:
            consumed_columns.add(test_col)
            part.loc[:, f"{domain}TEST"] = (
                frame.loc[has_value, test_col].astype("string").fillna("").str.strip()
            )
        else:
            # Fallback: use suffix as name if no explicit name column
            part.loc[:, f"{domain}TEST"] = suffix

        # Result
        part.loc[:, f"{domain}ORRES"] = values.loc[has_value]

        # Unit
        if unit_col in frame.columns:
            consumed_columns.add(unit_col)
            part.loc[:, f"{domain}ORRESU"] = (
                frame.loc[has_value, unit_col].astype("string").fillna("").str.strip()
            )
        else:
            part.loc[:, f"{domain}ORRESU"] = ""

        # Position
        if pos_col in frame.columns:
            consumed_columns.add(pos_col)
            part.loc[:, f"{domain}POS"] = (
                frame.loc[has_value, pos_col].astype("string").fillna("").str.strip()
            )

        out_frames.append(part)

    if not out_frames:
        return WideToLongResult(frame=frame, is_long=False)

    long_df = pd.concat(out_frames, ignore_index=True)
    return WideToLongResult(
        frame=long_df, is_long=True, consumed_columns=consumed_columns
    )
