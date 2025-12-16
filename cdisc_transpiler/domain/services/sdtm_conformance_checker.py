"""Deterministic SDTM/SDTMIG conformance checks.

This module is intentionally conservative:
- It only uses structured metadata already present in the in-memory domain
  definition (`SDTMDomain` / `SDTMVariable`) and optional controlled terminology
  lookups.
- It returns data (a report) and does not perform I/O.

These checks are meant to be *deterministic* and safe to run in strict output
modes (e.g., XPT/SAS). They do not try to infer "applicability" conditions for
Expected variables.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Callable, Literal

import pandas as pd

from ..entities.controlled_terminology import ControlledTerminology
from ..entities.sdtm_domain import SDTMDomain, SDTMVariable


Severity = Literal["error", "warning"]


@dataclass(frozen=True)
class ConformanceIssue:
    severity: Severity
    code: str
    domain: str
    variable: str | None
    message: str
    # Stable human-facing rule identifier (e.g., CT2001) for review/checklists.
    rule_id: str | None = None
    # Human-facing classification. Allowed values are enforced by convention.
    category: str | None = None
    count: int | None = None
    # For terminology issues, captures the NCI codelist code (e.g., C66768).
    codelist_code: str | None = None

    def to_dict(self) -> dict[str, object]:
        return {
            "severity": self.severity,
            "code": self.code,
            "rule_id": self.rule_id,
            "category": self.category,
            "domain": self.domain,
            "variable": self.variable,
            "message": self.message,
            "count": self.count,
            "codelist_code": self.codelist_code,
        }


@dataclass(frozen=True)
class ConformanceReport:
    domain: str
    issues: tuple[ConformanceIssue, ...]

    def has_errors(self) -> bool:
        return any(issue.severity == "error" for issue in self.issues)

    def error_count(self) -> int:
        return sum(1 for issue in self.issues if issue.severity == "error")

    def warning_count(self) -> int:
        return sum(1 for issue in self.issues if issue.severity == "warning")

    def to_dict(self) -> dict[str, object]:
        return {
            "domain": self.domain,
            "error_count": self.error_count(),
            "warning_count": self.warning_count(),
            "issues": [issue.to_dict() for issue in self.issues],
        }


CTResolver = Callable[[SDTMVariable], ControlledTerminology | None]


def _is_required(core: str | None) -> bool:
    return (core or "").strip().lower() == "req"


def _is_expected(core: str | None) -> bool:
    return (core or "").strip().lower() == "exp"


def _missing_count(series: pd.Series, variable: SDTMVariable) -> int:
    if variable.type == "Num":
        return int(pd.to_numeric(series, errors="coerce").isna().sum())

    text = series.astype("string")
    stripped = text.fillna("").str.strip()
    return int((stripped == "").sum())


def check_domain_dataframe(
    frame: pd.DataFrame,
    domain: SDTMDomain,
    *,
    ct_resolver: CTResolver | None = None,
) -> ConformanceReport:
    """Check a built domain DataFrame for basic SDTMIG conformance.

    Checks performed:
    - Required variables populated (Core == Req): non-empty / non-null per row.
    - Controlled terminology validity when `ct_resolver` is provided.

    Notes:
    - This does not attempt to infer conditional applicability for Expected vars.
    - This does not mutate the DataFrame.
    """

    issues: list[ConformanceIssue] = []

    for var in domain.variables:
        if var.name not in frame.columns:
            if _is_required(var.core):
                issues.append(
                    ConformanceIssue(
                        severity="error",
                        code="REQ_MISSING_COLUMN",
                        domain=domain.code,
                        variable=var.name,
                        message=f"Missing required column {var.name}",
                    )
                )
            elif _is_expected(var.core):
                issues.append(
                    ConformanceIssue(
                        severity="warning",
                        code="EXP_MISSING_COLUMN",
                        domain=domain.code,
                        variable=var.name,
                        message=f"Missing expected column {var.name}",
                    )
                )
            continue

        if _is_required(var.core):
            missing = _missing_count(frame[var.name], var)
            if missing:
                issues.append(
                    ConformanceIssue(
                        severity="error",
                        code="REQ_MISSING_VALUE",
                        domain=domain.code,
                        variable=var.name,
                        count=missing,
                        message=f"Required variable {var.name} has {missing} missing/blank values",
                    )
                )

        if ct_resolver is not None and (var.codelist_code or var.name):
            ct = ct_resolver(var)
            if ct is None:
                continue

            invalid = ct.invalid_values(frame[var.name])
            if invalid:
                example_items: list[str] = []
                for raw in sorted(list(invalid))[:5]:
                    suggestions = ct.suggest_submission_values(raw, limit=1)
                    if suggestions:
                        canonical = suggestions[0]
                        formatted = ct.format_submission_value_with_synonyms(canonical)
                        example_items.append(f"{raw} → {formatted}")
                    else:
                        example_items.append(raw)

                examples = ", ".join(example_items)
                severity: Severity = "warning" if ct.codelist_extensible else "error"
                issues.append(
                    ConformanceIssue(
                        severity=severity,
                        code="CT_INVALID",
                        domain=domain.code,
                        variable=var.name,
                        count=len(invalid),
                        codelist_code=ct.codelist_code,
                        message=(
                            f"{var.name} contains {len(invalid)} value(s) not found in CT for {ct.codelist_name} "
                            f"({ct.codelist_code}). Expected CDISC Submission Value (CDISC Synonym(s)); examples: {examples}"
                        ),
                    )
                )

    return ConformanceReport(domain=domain.code, issues=tuple(issues))
