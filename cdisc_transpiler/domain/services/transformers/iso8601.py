"""ISO 8601 date/time normalization utilities."""

import re

import pandas as pd

from ....pandas_utils import is_missing_scalar

DURATION_PATTERN = re.compile(
    r"^P(\d+Y)?(\d+M)?(\d+D)?(T(\d+H)?(\d+M)?(\d+(\.\d+)?S)?)?$"
)


def normalize_iso8601(raw_value: object) -> str:
    if is_missing_scalar(raw_value) or raw_value == "":
        return ""

    text = str(raw_value).strip()
    text_upper = text.upper()

    if "NK" in text_upper or "UN" in text_upper or "UNK" in text_upper:
        cleaned = re.sub(r"-?(NK|UN|UNK)", "", text, flags=re.IGNORECASE)
        cleaned = cleaned.rstrip("-")
        return cleaned or ""

    try:
        parsed = pd.to_datetime(text, errors="coerce", utc=False)
        if is_missing_scalar(parsed):
            if re.match(r"^\d{4}(-\d{2})?(-\d{2})?", text):
                return text
            return text
        return parsed.isoformat()
    except Exception:
        return text


def normalize_iso8601_duration(raw_value: object) -> str:
    if is_missing_scalar(raw_value) or raw_value == "":
        return ""

    text = str(raw_value).strip().upper()

    if DURATION_PATTERN.match(text):
        return text

    text_clean = _normalize_duration_tokens(text)
    days, hours, minutes, seconds, has_units = _extract_duration_units(text_clean)
    if not has_units:
        hours, minutes, seconds, has_units = _parse_time_or_number(text)
        days = 0

    return _format_duration(days, hours, minutes, seconds) if has_units else ""


def _normalize_duration_tokens(text: str) -> str:
    cleaned = text.replace("HOURS", "H").replace("HOUR", "H")
    cleaned = cleaned.replace("MINUTES", "M").replace("MINUTE", "M").replace("MIN", "M")
    cleaned = cleaned.replace("SECONDS", "S").replace("SECOND", "S").replace("SEC", "S")
    return cleaned.replace("DAYS", "D").replace("DAY", "D")


def _extract_duration_units(text: str) -> tuple[int, float, float, float, bool]:
    h_match = re.search(r"(\d+(?:\.\d+)?)\s*H", text)
    m_match = re.search(r"(\d+(?:\.\d+)?)\s*M(?!O)", text)
    s_match = re.search(r"(\d+(?:\.\d+)?)\s*S", text)
    d_match = re.search(r"(\d+)\s*D", text)

    hours = float(h_match.group(1)) if h_match else 0.0
    minutes = float(m_match.group(1)) if m_match else 0.0
    seconds = float(s_match.group(1)) if s_match else 0.0
    days = int(d_match.group(1)) if d_match else 0

    has_units = any((h_match, m_match, s_match, d_match))
    return days, hours, minutes, seconds, has_units


def _parse_time_or_number(text: str) -> tuple[float, float, float, bool]:
    time_match = re.match(r"^(\d{1,2}):(\d{2})(?::(\d{2}))?$", text)
    if time_match:
        hours = int(time_match.group(1))
        minutes = int(time_match.group(2))
        seconds = int(time_match.group(3) or 0)
        return float(hours), float(minutes), float(seconds), True

    num_match = re.match(r"^(\d+(?:\.\d+)?)$", text)
    if num_match:
        value = float(num_match.group(1))
        return 0.0, value, 0.0, True

    return 0.0, 0.0, 0.0, False


def _format_duration(days: int, hours: float, minutes: float, seconds: float) -> str:
    if days == 0 and hours == 0 and minutes == 0 and seconds == 0:
        return ""

    duration = "P"
    if days > 0:
        duration += f"{days}D"
    if hours > 0 or minutes > 0 or seconds > 0:
        duration += "T"
        if hours > 0:
            duration += f"{_format_unit(hours)}H"
        if minutes > 0:
            duration += f"{_format_unit(minutes)}M"
        if seconds > 0:
            duration += f"{_format_unit(seconds)}S"
    return duration if duration != "P" else ""


def _format_unit(value: float) -> str:
    integer = int(value)
    return str(integer) if value == integer else str(value)
