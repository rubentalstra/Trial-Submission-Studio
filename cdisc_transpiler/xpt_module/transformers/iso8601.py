"""ISO 8601 date/time normalization utilities.

This module provides functions for normalizing date/time values
to ISO 8601 format as required by SDTM standards.

Per SDTM IG v3.4:
- Date/time variables (--DTC) must conform to ISO 8601
- Duration variables (--DUR) must use ISO 8601 duration format
- Unknown date components should be omitted (not replaced with letters)
"""

from __future__ import annotations

import re

import pandas as pd
from pandas import isna


def normalize_iso8601(raw_value) -> str:
    """Normalize date/time-ish strings to ISO8601; return original if invalid.

    Uses :func:`pandas.isna` to safely handle ``pd.NA`` and other missing
    markers without triggering "boolean value of NA is ambiguous" errors
    when called from ``Series.apply``.

    SDTM supports partial dates with unknown components. This function handles:
    - Full dates: 2023-09-15 -> 2023-09-15
    - Partial dates: 2023-09-NK, 2023-NK-NK -> preserved as-is (invalid but kept)
    - Non-standard formats: cleaned up to ISO 8601

    Unknown date components should NOT contain letters like 'NK'.
    Per SDTM IG, unknown parts should be omitted (e.g., 2023-09 for unknown day).

    Args:
        raw_value: Value to normalize

    Returns:
        ISO 8601 formatted string or empty string if invalid
    """
    # Treat all missing-like values (None, NaN, pd.NA, empty string) as empty
    if isna(raw_value) or raw_value == "":
        return ""

    text = str(raw_value).strip()
    text_upper = text.upper()

    # Handle partial dates with "NK" (Not Known) - convert to proper ISO 8601 partial date
    # E.g., "2023-10-NK" -> "2023-10" (unknown day is omitted)
    # E.g., "2023-NK-NK" -> "2023" (unknown month and day)
    if "NK" in text_upper or "UN" in text_upper or "UNK" in text_upper:
        # Replace NK/UN/UNK patterns with empty
        cleaned = re.sub(r"-?(NK|UN|UNK)", "", text, flags=re.IGNORECASE)
        cleaned = cleaned.rstrip("-")  # Remove trailing dashes
        if cleaned:
            return cleaned
        return ""

    try:
        parsed = pd.to_datetime(raw_value, errors="coerce", utc=False)
        if pd.isna(parsed):
            # If parsing fails, still return the original if it looks like a partial date
            if re.match(r"^\d{4}(-\d{2})?(-\d{2})?", text):
                return text
            return str(raw_value)
        return parsed.isoformat()
    except Exception:
        return str(raw_value)


def normalize_iso8601_duration(raw_value) -> str:
    """Normalize elapsed time/duration values to ISO 8601 duration format.

    ISO 8601 durations: PnYnMnDTnHnMnS
    Examples: PT1H (1 hour), PT30M (30 minutes), P1D (1 day), PT1H30M (1.5 hours)

    Common input formats:
    - "1 hour", "30 minutes", "2 hours 30 minutes"
    - "1:30" (hours:minutes)
    - "PT1H30M" (already ISO 8601)
    - "30" or "30 min" (assumed minutes)

    Args:
        raw_value: Value to normalize

    Returns:
        ISO 8601 duration string (e.g., "PT1H30M") or empty string
    """
    # Treat all missing-like values (None, NaN, pd.NA, empty string) as empty
    if isna(raw_value) or raw_value == "":
        return ""

    text = str(raw_value).strip().upper()

    # Already ISO 8601 duration format
    if re.match(r"^P(\d+Y)?(\d+M)?(\d+D)?(T(\d+H)?(\d+M)?(\d+(\.\d+)?S)?)?$", text):
        return text

    # Clean up common variations
    text_clean = text.replace("HOURS", "H").replace("HOUR", "H")
    text_clean = (
        text_clean.replace("MINUTES", "M").replace("MINUTE", "M").replace("MIN", "M")
    )
    text_clean = (
        text_clean.replace("SECONDS", "S").replace("SECOND", "S").replace("SEC", "S")
    )
    text_clean = text_clean.replace("DAYS", "D").replace("DAY", "D")

    hours = 0
    minutes = 0
    seconds = 0
    days = 0

    # Match patterns like "1H 30M" or "1H30M"
    h_match = re.search(r"(\d+(?:\.\d+)?)\s*H", text_clean)
    m_match = re.search(r"(\d+(?:\.\d+)?)\s*M(?!O)", text_clean)  # M but not MONTH
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

    # If none matched, try HH:MM:SS or HH:MM format
    if not any([h_match, m_match, s_match, d_match]):
        time_match = re.match(r"^(\d{1,2}):(\d{2})(?::(\d{2}))?$", text)
        if time_match:
            hours = int(time_match.group(1))
            minutes = int(time_match.group(2))
            seconds = int(time_match.group(3) or 0)
        else:
            # Try plain number (assume minutes if small, hours if with decimal)
            num_match = re.match(r"^(\d+(?:\.\d+)?)$", text)
            if num_match:
                value = float(num_match.group(1))
                if value < 24:
                    # Could be hours, but most likely small values are minutes
                    minutes = value
                else:
                    minutes = value

    # If still nothing parsed, return empty
    if days == 0 and hours == 0 and minutes == 0 and seconds == 0:
        return ""

    # Build ISO 8601 duration string
    duration = "P"
    if days > 0:
        duration += f"{days}D"
    if hours > 0 or minutes > 0 or seconds > 0:
        duration += "T"
        if hours > 0:
            if hours == int(hours):
                duration += f"{int(hours)}H"
            else:
                duration += f"{hours}H"
        if minutes > 0:
            if minutes == int(minutes):
                duration += f"{int(minutes)}M"
            else:
                duration += f"{minutes}M"
        if seconds > 0:
            if seconds == int(seconds):
                duration += f"{int(seconds)}S"
            else:
                duration += f"{seconds}S"

    return duration if duration != "P" else ""
