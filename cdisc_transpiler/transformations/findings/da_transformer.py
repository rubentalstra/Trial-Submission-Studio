"""DA (Drug Accountability) domain transformer.

This transformer converts certain wide-format DA exports (where multiple DA tests
are represented by prefixed column groups such as RETAMT_* and DISAMT_*) into
SDTM long format (one row per DA test).

It is intentionally conservative and only emits rows for test groups that are
well-defined and have SDTM CT support:
- RETAMT -> Returned Amount
- DISAMT -> DISPAMT (Dispensed Amount)

SDTM Reference:
    DA is a Findings domain. The required test identifiers are DATESTCD/DATEST.

This transformer is not a full semantic DA mapper; it focuses on producing a
structurally valid long-format dataset so downstream mapping/conformance checks
operate on correct columns (e.g., dates stay in DADTC, not DATEST).
"""

from __future__ import annotations

import re

import pandas as pd

from ..base import TransformationContext, TransformationResult


class DATransformer:
    """Transformer for DA (Drug Accountability) wide-to-long conversion."""

    def __init__(
        self,
        test_code_normalizer=None,
        test_label_getter=None,
    ) -> None:
        self.domain = "DA"
        self.test_code_normalizer = test_code_normalizer
        self.test_label_getter = test_label_getter

        self._column_renames: dict[str, str] = {
            "Subject Id": "USUBJID",
            "SubjectId": "USUBJID",
            "Event name": "VISIT",
            "Event Name": "VISIT",
            "EventName": "VISIT",
            "Event sequence number": "VISITNUM",
            "Event Sequence Number": "VISITNUM",
            "EventSeq": "VISITNUM",
            # Base event date (used as fallback when group date missing)
            "Event date": "DADTC",
            "Event Date": "DADTC",
            "EventDate": "DADTC",
        }

        self._group_patterns: dict[str, re.Pattern[str]] = {
            "orres": re.compile(r"^([A-Za-z0-9]+)_DAORRES$", re.IGNORECASE),
            "date": re.compile(r"^([A-Za-z0-9]+)_DADAT$", re.IGNORECASE),
            "reasnd": re.compile(r"^([A-Za-z0-9]+)_DAREASND$", re.IGNORECASE),
            "refid": re.compile(r"^([A-Za-z0-9]+)_DAREFID(?:[1-3])?$", re.IGNORECASE),
        }

    def _discover_groups(self, columns: list[str]) -> dict[str, dict[str, list[str]]]:
        groups: dict[str, dict[str, list[str]]] = {}
        for col in columns:
            for key, pattern in self._group_patterns.items():
                match = pattern.match(col)
                if not match:
                    continue
                prefix = match.group(1).upper()
                groups.setdefault(
                    prefix, {"orres": [], "date": [], "reasnd": [], "refid": []}
                )
                groups[prefix][key].append(col)

        for prefix, parts in groups.items():
            # Stable selection order
            parts["orres"].sort()
            parts["date"].sort()
            parts["reasnd"].sort()
            parts["refid"].sort()
        return groups

    def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
        if domain.upper() != self.domain:
            return False

        groups = self._discover_groups(list(df.columns))
        return len(groups) > 0

    def transform(
        self, df: pd.DataFrame, context: TransformationContext
    ) -> TransformationResult:
        if not self.can_transform(df, context.domain):
            return TransformationResult(
                data=df,
                applied=False,
                message=f"Transformer does not apply to domain {context.domain}",
            )

        input_rows = len(df)

        frame = df.copy()
        frame = frame.rename(
            columns={
                k: v for k, v in self._column_renames.items() if k in frame.columns
            }
        )

        # Ensure numeric VISITNUM when possible.
        if "VISITNUM" in frame.columns:
            frame = frame.assign(
                VISITNUM=pd.to_numeric(frame["VISITNUM"], errors="coerce")
            )

        records: list[dict[str, object]] = []

        groups = self._discover_groups(list(frame.columns))
        cols = set(frame.columns)
        for _, row in frame.iterrows():
            for prefix, parts in groups.items():
                # Choose the first candidate column for each part (if any)
                orres_cols = parts.get("orres") or []
                date_cols = parts.get("date") or []
                reasnd_cols = parts.get("reasnd") or []
                refid_cols = parts.get("refid") or []

                orres_col = orres_cols[0] if orres_cols else None
                date_col = date_cols[0] if date_cols else None
                reasnd_col = reasnd_cols[0] if reasnd_cols else None

                testcd = prefix
                if self.test_code_normalizer:
                    normalized = self.test_code_normalizer(self.domain, testcd)
                    if normalized:
                        testcd = normalized

                test_label = testcd
                if self.test_label_getter:
                    label = self.test_label_getter(self.domain, testcd)
                    if label:
                        test_label = label

                # Pull values (keeping empty -> None)
                def _get(vname: str) -> object | None:
                    if vname not in cols:
                        return None
                    value = row.get(vname)
                    if pd.isna(value):
                        return None
                    return value

                daorres = _get(orres_col) if orres_col else None
                dadtc = (_get(date_col) if date_col else None) or _get("DADTC")
                dareasnd = _get(reasnd_col) if reasnd_col else None

                darefid = None
                for c in refid_cols:
                    val = _get(c)
                    if val is not None and str(val).strip() != "":
                        darefid = val
                        break

                # Emit only when at least one meaningful measurement value is present.
                if all(
                    v is None or str(v).strip() == ""
                    for v in (daorres, dadtc, dareasnd, darefid)
                ):
                    continue

                records.append(
                    {
                        "USUBJID": _get("USUBJID") or "",
                        "VISIT": _get("VISIT") or "",
                        "VISITNUM": _get("VISITNUM"),
                        "DATESTCD": testcd,
                        "DATEST": test_label,
                        "DAORRES": daorres,
                        "DAREASND": dareasnd,
                        "DAREFID": darefid,
                        "DADTC": dadtc,
                    }
                )

        long_df = pd.DataFrame.from_records(records)

        return TransformationResult(
            data=long_df,
            applied=True,
            message=f"Converted {input_rows} wide rows to {len(long_df)} long rows",
            metadata={
                "input_rows": input_rows,
                "output_rows": len(long_df),
                "tests_found": len(groups),
                "test_codes": sorted(groups.keys()),
            },
        )
