"""ISO 8601 date/time normalization utilities."""

from __future__ import annotations

import re
from typing import Any

import pandas as pd
from pandas import isna


def normalize_iso8601(raw_value: Any) -> str:
    if isna(raw_value) or raw_value == "":
        return ""

    text = str(raw_value).strip()
    text_upper = text.upper()

    if "NK" in text_upper or "UN" in text_upper or "UNK" in text_upper:
        cleaned = re.sub(r"-?(NK|UN|UNK)", "", text, flags=re.IGNORECASE)
        cleaned = cleaned.rstrip("-")
        return cleaned or ""

    try:
        parsed = pd.to_datetime(raw_value, errors="coerce", utc=False)
        if pd.isna(parsed):
            if re.match(r"^\d{4}(-\d{2})?(-\d{2})?", text):
                return text
            return str(raw_value)
        return parsed.isoformat()
    except Exception:
        return str(raw_value)


def normalize_iso8601_duration(raw_value: Any) -> str:
    if isna(raw_value) or raw_value == "":
        return ""

    text = str(raw_value).strip().upper()

    if re.match(r"^P(\d+Y)?(\d+M)?(\d+D)?(T(\d+H)?(\d+M)?(\d+(\.\d+)?S)?)?$", text):
        return text

    text_clean = text.replace("HOURS", "H").replace("HOUR", "H")
    text_clean = (
        text_clean.replace("MINUTES", "M").replace("MINUTE", "M").replace("MIN", "M")
    )
    text_clean = (
        text_clean.replace("SECONDS", "S").replace("SECOND", "S").replace("SEC", "S")
    )
    text_clean = text_clean.replace("DAYS", "D").replace("DAY", "D")

    hours = 0.0
    minutes = 0.0
    seconds = 0.0
    days = 0

    h_match = re.search(r"(\d+(?:\.\d+)?)\s*H", text_clean)
    m_match = re.search(r"(\d+(?:\.\d+)?)\s*M(?!O)", text_clean)
    s_match = re.search(r"(\d+(?:\.\d+)?)\s*S", text_clean)
    d_match = re.search(r"(\d+)\s*D", text_clean)

    if h_match:
        hours = float(h_match.group(1))
    if m_match:
        minutes = float(m_match.group(1))
    if s_match:
        seconds = float(s_match.group(1))
    if d_match:
        days = int(d_match.group(1))

    if not any([h_match, m_match, s_match, d_match]):
        time_match = re.match(r"^(\d{1,2}):(\d{2})(?::(\d{2}))?$", text)
        if time_match:
            hours = int(time_match.group(1))
            minutes = int(time_match.group(2))
            seconds = int(time_match.group(3) or 0)
        else:
            num_match = re.match(r"^(\d+(?:\.\d+)?)$", text)
            if num_match:
                value = float(num_match.group(1))
                minutes = value

    if days == 0 and hours == 0 and minutes == 0 and seconds == 0:
        return ""

    duration = "P"
    if days > 0:
        duration += f"{days}D"
    if hours > 0 or minutes > 0 or seconds > 0:
        duration += "T"
        if hours > 0:
            duration += f"{int(hours) if hours == int(hours) else hours}H"
        if minutes > 0:
            duration += f"{int(minutes) if minutes == int(minutes) else minutes}M"
        if seconds > 0:
            duration += f"{int(seconds) if seconds == int(seconds) else seconds}S"

    return duration if duration != "P" else ""
