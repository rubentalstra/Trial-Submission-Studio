from collections.abc import Callable
from dataclasses import dataclass
from typing import TYPE_CHECKING, Literal

import pandas as pd

from ..entities.controlled_terminology import ControlledTerminology
from ..entities.sdtm_domain import SDTMVariable

if TYPE_CHECKING:
    from ..entities.sdtm_domain import SDTMDomain
Severity = Literal["error", "warning"]


@dataclass(frozen=True, slots=True)
class ConformanceIssue:
    severity: Severity
    code: str
    domain: str
    variable: str | None
    message: str
    rule_id: str | None = None
    category: str | None = None
    count: int | None = None
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


@dataclass(frozen=True, slots=True)
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
    frame: pd.DataFrame, domain: SDTMDomain, *, ct_resolver: CTResolver | None = None
) -> ConformanceReport:
    issues: list[ConformanceIssue] = []
    for var in domain.variables:
        if var.name not in frame.columns:
            issues.extend(_missing_column_issues(domain, var))
            continue
        issues.extend(_missing_value_issues(frame, domain, var))
        if ct_resolver is not None and (var.codelist_code or var.name):
            issues.extend(_ct_issues(frame, domain, var, ct_resolver))
    return ConformanceReport(domain=domain.code, issues=tuple(issues))


def _missing_column_issues(
    domain: SDTMDomain, var: SDTMVariable
) -> list[ConformanceIssue]:
    if _is_required(var.core):
        return [
            ConformanceIssue(
                severity="error",
                code="REQ_MISSING_COLUMN",
                domain=domain.code,
                variable=var.name,
                message=f"Missing required column {var.name}",
            )
        ]
    if _is_expected(var.core):
        return [
            ConformanceIssue(
                severity="warning",
                code="EXP_MISSING_COLUMN",
                domain=domain.code,
                variable=var.name,
                message=f"Missing expected column {var.name}",
            )
        ]
    return []


def _missing_value_issues(
    frame: pd.DataFrame, domain: SDTMDomain, var: SDTMVariable
) -> list[ConformanceIssue]:
    if not _is_required(var.core):
        return []
    missing = _missing_count(frame[var.name], var)
    if not missing:
        return []
    return [
        ConformanceIssue(
            severity="error",
            code="REQ_MISSING_VALUE",
            domain=domain.code,
            variable=var.name,
            count=missing,
            message=f"Required variable {var.name} has {missing} missing/blank values",
        )
    ]


def _ct_issues(
    frame: pd.DataFrame, domain: SDTMDomain, var: SDTMVariable, ct_resolver: CTResolver
) -> list[ConformanceIssue]:
    ct = ct_resolver(var)
    if ct is None:
        return []
    series_for_ct = _select_ct_series(frame, domain, var)
    invalid = ct.invalid_values(series_for_ct)
    if not invalid:
        return []
    example_items: list[str] = []
    for raw in sorted(invalid)[:5]:
        suggestions = ct.suggest_submission_values(raw, limit=1)
        if suggestions:
            canonical = suggestions[0]
            formatted = ct.format_submission_value_with_synonyms(canonical)
            example_items.append(f"{raw} → {formatted}")
        else:
            example_items.append(raw)
    examples = ", ".join(example_items)
    severity: Severity = "warning" if ct.codelist_extensible else "error"
    return [
        ConformanceIssue(
            severity=severity,
            code="CT_INVALID",
            domain=domain.code,
            variable=var.name,
            count=len(invalid),
            codelist_code=ct.codelist_code,
            message=f"{var.name} contains {len(invalid)} value(s) not found in CT for {ct.codelist_name} ({ct.codelist_code}). examples: {examples}",
        )
    ]


def _select_ct_series(
    frame: pd.DataFrame, domain: SDTMDomain, var: SDTMVariable
) -> pd.Series:
    series_for_ct = frame[var.name]
    if (
        domain.code.upper() == "DS"
        and var.name == "DSDECOD"
        and ("DSCAT" in frame.columns)
    ):
        dscat_upper = frame["DSCAT"].astype("string").fillna("").str.upper().str.strip()
        disposition_mask = (dscat_upper == "DISPOSITION EVENT") | (dscat_upper == "")
        series_for_ct = series_for_ct.loc[disposition_mask]
    if domain.code.upper() == "LB" and var.name == "LBSTRESC":
        mask_numeric = _is_numeric_like_text(series_for_ct)
        series_for_ct = series_for_ct.loc[~mask_numeric]
    return series_for_ct


def _is_numeric_like_text(value: pd.Series) -> pd.Series:
    text = value.astype("string").fillna("").str.strip()
    return text.str.match("^(?:[<>]=?\\s*)?\\d+(?:\\.\\d+)?$")
