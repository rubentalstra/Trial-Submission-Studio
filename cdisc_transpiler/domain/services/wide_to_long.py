"""Wide-to-long reshaping helpers for SDTM Findings-style source extracts.

These helpers are *domain logic*:
- No filesystem/network I/O
- Pure pandas reshaping + light normalization

They intentionally implement only the minimal mappings needed for the demo
study inputs bundled with this repository.
"""

from __future__ import annotations

from dataclasses import dataclass

import pandas as pd


@dataclass(frozen=True)
class WideToLongResult:
    frame: pd.DataFrame
    is_long: bool


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


def transform_vs_wide_to_long(
    frame: pd.DataFrame, *, study_id: str
) -> WideToLongResult:
    """Transform a VS wide extract (one row per visit) into SDTM VS long records.

    Expected source columns (from demo inputs):
    - SubjectId, EventName
    - VSDAT, VSTIM (optional)
    - VSPERFCD (Y/N)
    - ORRES_HEIGHT/WEIGHT/BMI/TEMP/PLS/SYSBP/DIABP
    - ORRESU_* unit columns
    - POS_* position columns (optional)

    Returns:
        WideToLongResult(frame=<long frame>, is_long=True) when the shape matches,
        otherwise returns the original frame (is_long=False).
    """

    candidate_cols = {
        "ORRES_HEIGHT",
        "ORRES_WEIGHT",
        "ORRES_BMI",
        "ORRES_TEMP",
        "ORRES_PLS",
        "ORRES_SYSBP",
        "ORRES_DIABP",
    }
    if not (candidate_cols & set(frame.columns)):
        return WideToLongResult(frame=frame, is_long=False)

    base = pd.DataFrame(index=frame.index)

    subject_id_col = "SubjectId" if "SubjectId" in frame.columns else "SUBJECTID"
    if subject_id_col in frame.columns:
        base.loc[:, "USUBJID"] = frame[subject_id_col].map(
            lambda v: _make_usubjid(study_id, v)
        )
    else:
        base.loc[:, "USUBJID"] = ""

    # Visit/date/time
    event_name_col = "EventName" if "EventName" in frame.columns else "VISIT"
    if event_name_col in frame.columns:
        base.loc[:, "VISIT"] = frame[event_name_col].astype("string").fillna("")
        base.loc[:, "VISITNUM"] = frame[event_name_col].map(_try_parse_visitnum)
    else:
        base.loc[:, "VISIT"] = ""
        base.loc[:, "VISITNUM"] = None

    if "VSDAT" in frame.columns:
        vsdat = frame["VSDAT"].astype("string").fillna("").str.strip()
    else:
        vsdat = pd.Series([""] * len(frame), index=frame.index, dtype="string")

    if "VSTIM" in frame.columns:
        vstim = frame["VSTIM"].astype("string").fillna("").str.strip()
    else:
        vstim = pd.Series([""] * len(frame), index=frame.index, dtype="string")

    # ISO-ish: YYYY-MM-DD or YYYY-MM-DDThh:mm
    vsdtc = vsdat.where(vsdat != "", "")
    has_time = (vstim != "") & (vsdat != "")
    vsdtc = vsdtc.where(~has_time, vsdat + "T" + vstim)
    base.loc[:, "VSDTC"] = vsdtc

    perf = frame.get(
        "VSPERFCD", pd.Series([""] * len(frame), index=frame.index)
    ).astype("string")
    perf = perf.fillna("").str.upper().str.strip()
    base.loc[:, "VSSTAT"] = ""
    base.loc[perf == "N", "VSSTAT"] = "NOT DONE"

    measures: list[tuple[str, str, str, str | None]] = [
        ("HEIGHT", "Height", "ORRES_HEIGHT", "ORRESU_HEIGHT"),
        ("WEIGHT", "Weight", "ORRES_WEIGHT", "ORRESU_WEIGHT"),
        ("BMI", "Body Mass Index", "ORRES_BMI", "ORRESU_BMI"),
        ("TEMP", "Temperature", "ORRES_TEMP", "ORRESU_TEMP"),
        # CT for VSTEST expects "Pulse Rate" rather than "Pulse".
        ("PULSE", "Pulse Rate", "ORRES_PLS", "ORRESU_PLS"),
        ("SYSBP", "Systolic Blood Pressure", "ORRES_SYSBP", "ORRESU_BP"),
        ("DIABP", "Diastolic Blood Pressure", "ORRES_DIABP", "ORRESU_BP"),
    ]

    out_frames: list[pd.DataFrame] = []
    for testcd, test, value_col, unit_col in measures:
        if value_col not in frame.columns:
            continue

        values = frame[value_col]
        values_str = values.astype("string").fillna("").str.strip()
        has_value = values_str != ""

        # If visit marked NOT DONE, skip creating records entirely.
        if (perf == "N").any():
            has_value = has_value & (perf != "N")

        if not has_value.any():
            continue

        part = pd.DataFrame(index=frame.index[has_value])
        part.loc[:, "STUDYID"] = study_id
        part.loc[:, "DOMAIN"] = "VS"
        part.loc[:, "USUBJID"] = base.loc[has_value, "USUBJID"]
        part.loc[:, "VISIT"] = base.loc[has_value, "VISIT"]
        part.loc[:, "VISITNUM"] = base.loc[has_value, "VISITNUM"]
        part.loc[:, "VSDTC"] = base.loc[has_value, "VSDTC"]
        part.loc[:, "VSSTAT"] = base.loc[has_value, "VSSTAT"]
        part.loc[:, "VSTESTCD"] = testcd
        part.loc[:, "VSTEST"] = test
        part.loc[:, "VSORRES"] = values_str.loc[has_value]
        if unit_col and unit_col in frame.columns:
            part.loc[:, "VSORRESU"] = (
                frame.loc[has_value, unit_col].astype("string").fillna("").str.strip()
            )
        else:
            part.loc[:, "VSORRESU"] = ""

        if testcd in {"PULSE"} and "POS_PLS" in frame.columns:
            part.loc[:, "VSPOS"] = (
                frame.loc[has_value, "POS_PLS"].astype("string").fillna("").str.strip()
            )
        elif testcd in {"SYSBP", "DIABP"} and "POS_BP" in frame.columns:
            part.loc[:, "VSPOS"] = (
                frame.loc[has_value, "POS_BP"].astype("string").fillna("").str.strip()
            )

        out_frames.append(part)

    if not out_frames:
        return WideToLongResult(frame=frame, is_long=False)

    long_df = pd.concat(out_frames, ignore_index=True)
    return WideToLongResult(frame=long_df, is_long=True)


def transform_lb_wide_to_long(
    frame: pd.DataFrame, *, study_id: str
) -> WideToLongResult:
    """Transform LB wide extracts (multiple tests per row) into SDTM LB long records."""

    if not any(col.startswith("ORRES_") for col in frame.columns):
        return WideToLongResult(frame=frame, is_long=False)

    # Only reshape well-known test suffixes that are likely to exist in SDTM LBTESTCD CT.
    # This avoids incorrectly turning operational/helper columns (e.g., ORRES_CULTURE0)
    # into LBTESTCD values, which would create new CT_INVALID issues.
    allowed_suffixes = {
        "ALT",
        "AST",
        "CHOL",
        "GLUC",
        "HGB",
        "HCT",
        "RBC",
        "WBC",
        "PLAT",
        "PROT",
        "OCCBLD",
        "GLUCU",
        "PH",
    }

    # Determine which tests are present by scanning ORRES_ columns.
    test_suffixes: list[str] = []
    for col in frame.columns:
        if not col.startswith("ORRES_"):
            continue
        suffix = col.replace("ORRES_", "", 1)
        if suffix in allowed_suffixes:
            test_suffixes.append(suffix)

    if not test_suffixes:
        return WideToLongResult(frame=frame, is_long=False)

    subject_id_col = "SubjectId" if "SubjectId" in frame.columns else "SUBJECTID"
    usubjid = (
        frame[subject_id_col].map(lambda v: _make_usubjid(study_id, v))
        if subject_id_col in frame.columns
        else pd.Series([""] * len(frame), index=frame.index, dtype="string")
    )

    event_name_col = "EventName" if "EventName" in frame.columns else "VISIT"
    visit = (
        frame[event_name_col].astype("string").fillna("")
        if event_name_col in frame.columns
        else pd.Series([""] * len(frame), index=frame.index, dtype="string")
    )
    visitnum = (
        frame[event_name_col].map(_try_parse_visitnum)
        if event_name_col in frame.columns
        else None
    )

    # Best-effort collection date: prefer *DAT columns
    date_col = None
    for candidate in (
        "LBCCDAT",
        "LBHMDAT",
        "LBURDAT",
        "LBSADAT",
        "PREGDAT",
        "LBDAT",
    ):
        if candidate in frame.columns:
            date_col = candidate
            break

    if date_col is not None:
        lbdtc = frame[date_col].astype("string").fillna("").str.strip()
    else:
        lbdtc = pd.Series([""] * len(frame), index=frame.index, dtype="string")

    # Basic perf handling
    perf_col = None
    for candidate in (
        "LBCCPERFCD",
        "LBHMPERFCD",
        "LBURPERFCD",
        "LBSAPERFCD",
        "PREGPERFCD",
    ):
        if candidate in frame.columns:
            perf_col = candidate
            break

    perf = (
        frame[perf_col].astype("string").fillna("").str.upper().str.strip()
        if perf_col is not None
        else pd.Series([""] * len(frame), index=frame.index, dtype="string")
    )

    out_frames: list[pd.DataFrame] = []

    for suffix in sorted(set(test_suffixes)):
        orres_col = f"ORRES_{suffix}"
        if orres_col not in frame.columns:
            continue

        # Skip empty columns.
        orres = frame[orres_col].astype("string").fillna("").str.strip()
        has_value = orres != ""
        if perf_col is not None:
            has_value = has_value & (perf != "N")

        if not has_value.any():
            continue

        # Labels and units (when present)
        test_label_col = f"TEST_{suffix}"
        unit_col = f"ORRESU_{suffix}"

        lbtest = (
            frame.loc[has_value, test_label_col].astype("string").fillna("").str.strip()
            if test_label_col in frame.columns
            else pd.Series(
                [suffix] * int(has_value.sum()),
                index=frame.index[has_value],
                dtype="string",
            )
        )

        lborresu = (
            frame.loc[has_value, unit_col].astype("string").fillna("").str.strip()
            if unit_col in frame.columns
            else pd.Series(
                [""] * int(has_value.sum()),
                index=frame.index[has_value],
                dtype="string",
            )
        )

        part = pd.DataFrame(index=frame.index[has_value])
        part.loc[:, "STUDYID"] = study_id
        part.loc[:, "DOMAIN"] = "LB"
        part.loc[:, "USUBJID"] = usubjid.loc[has_value]
        part.loc[:, "VISIT"] = visit.loc[has_value]
        part.loc[:, "VISITNUM"] = (
            visitnum[has_value] if isinstance(visitnum, pd.Series) else visitnum
        )
        part.loc[:, "LBDTC"] = lbdtc.loc[has_value]
        # Some source exports use GLUCU for urine glucose; standardize to GLUC
        # to satisfy CT expectations for LBTESTCD.
        testcd = "GLUC" if suffix == "GLUCU" else suffix
        part.loc[:, "LBTESTCD"] = testcd
        part.loc[:, "LBTEST"] = lbtest
        part.loc[:, "LBORRES"] = orres.loc[has_value]
        part.loc[:, "LBORRESU"] = lborresu

        # Standardize results: mirror to STRES* (numeric parsed later in processor)
        part.loc[:, "LBSTRESC"] = part["LBORRES"].astype("string").fillna("")
        part.loc[:, "LBSTRESU"] = part["LBORRESU"].astype("string").fillna("")

        out_frames.append(part)

    if not out_frames:
        return WideToLongResult(frame=frame, is_long=False)

    long_df = pd.concat(out_frames, ignore_index=True)
    return WideToLongResult(frame=long_df, is_long=True)
