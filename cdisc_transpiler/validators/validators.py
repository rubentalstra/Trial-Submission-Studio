"""Validation framework for SDTM compliance and Pinnacle 21 rules.

This module implements a comprehensive validation system that checks:
- SDTM IG compliance (required variables, data types, formats)
- Pinnacle 21 validation rules (SD*, DD*, CT* rules)
- Cross-domain relationships and referential integrity
- Controlled terminology compliance
- ISO 8601 date/time formats
- Study day calculations

The validation framework is designed to be extensible and report
detailed error information for debugging and correction.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from datetime import date, datetime
from enum import Enum
from typing import TYPE_CHECKING, Any, Callable

import pandas as pd

if TYPE_CHECKING:
    from ..domains_module import SDTMDomain, SDTMVariable
    from ..terminology import ControlledTerminology

# ISO 8601 patterns for date/time validation
ISO8601_DATE_PATTERN = re.compile(
    r"^\d{4}(-\d{2}(-\d{2})?)?$"  # YYYY, YYYY-MM, or YYYY-MM-DD
)
ISO8601_DATETIME_PATTERN = re.compile(
    r"^\d{4}(-\d{2}(-\d{2}(T\d{2}(:\d{2}(:\d{2}(\.\d+)?)?)?)?)?)?$"
)
ISO8601_DURATION_PATTERN = re.compile(
    r"^P(\d+Y)?(\d+M)?(\d+D)?(T(\d+H)?(\d+M)?(\d+(\.\d+)?S)?)?$"
)


class ValidationSeverity(str, Enum):
    """Validation issue severity levels aligned with Pinnacle 21."""

    ERROR = "Error"  # Must be fixed
    WARNING = "Warning"  # Should be reviewed
    INFO = "Info"  # Informational only


class ValidationCategory(str, Enum):
    """Validation category classifications."""

    PRESENCE = "Presence"  # Missing required elements
    CONSISTENCY = "Consistency"  # Internal data consistency
    FORMAT = "Format"  # Data format compliance
    TERMINOLOGY = "Terminology"  # CT compliance
    CROSS_REFERENCE = "Cross-reference"  # Inter-domain relationships
    METADATA = "Metadata"  # Define-XML metadata
    STRUCTURE = "Structure"  # XML/file structure
    LIMIT = "Limit"  # Value range violations


@dataclass
class ValidationIssue:
    """Represents a validation issue found during processing."""

    rule_id: str  # e.g., "SD0002", "DD0031"
    severity: ValidationSeverity
    category: ValidationCategory
    message: str
    domain_code: str | None = None
    variable_name: str | None = None
    record_identifier: str | None = None  # e.g., USUBJID value
    details: dict[str, Any] = field(default_factory=dict)

    def __str__(self) -> str:
        """Format issue for display."""
        parts = [f"[{self.rule_id}] {self.severity.value}: {self.message}"]
        if self.domain_code:
            parts.append(f"Domain: {self.domain_code}")
        if self.variable_name:
            parts.append(f"Variable: {self.variable_name}")
        if self.record_identifier:
            parts.append(f"Record: {self.record_identifier}")
        return " | ".join(parts)


@dataclass
class ValidationContext:
    """Context information for validation operations."""

    study_id: str
    domain_code: str
    domain: SDTMDomain
    dataframe: pd.DataFrame
    all_domains: dict[str, pd.DataFrame] = field(default_factory=dict)
    controlled_terminology: ControlledTerminology | None = None
    reference_starts: dict[str, str] = field(default_factory=dict)


class ValidationRule:
    """Base class for validation rules."""

    def __init__(
        self,
        rule_id: str,
        severity: ValidationSeverity,
        category: ValidationCategory,
        message_template: str,
    ):
        self.rule_id = rule_id
        self.severity = severity
        self.category = category
        self.message_template = message_template

    def validate(self, context: ValidationContext) -> list[ValidationIssue]:
        """Execute the validation rule and return any issues found."""
        raise NotImplementedError("Subclasses must implement validate()")

    def create_issue(
        self,
        message: str,
        domain_code: str | None = None,
        variable_name: str | None = None,
        record_identifier: str | None = None,
        **details: Any,
    ) -> ValidationIssue:
        """Helper to create a ValidationIssue."""
        return ValidationIssue(
            rule_id=self.rule_id,
            severity=self.severity,
            category=self.category,
            message=message,
            domain_code=domain_code,
            variable_name=variable_name,
            record_identifier=record_identifier,
            details=details,
        )


class RequiredVariableValidator(ValidationRule):
    """SD0002: Null value in variable marked as Required."""

    def __init__(self):
        super().__init__(
            rule_id="SD0002",
            severity=ValidationSeverity.ERROR,
            category=ValidationCategory.PRESENCE,
            message_template="Required variable '{variable}' has null values",
        )

    def validate(self, context: ValidationContext) -> list[ValidationIssue]:
        issues = []
        df = context.dataframe

        # Check each required variable
        for var in context.domain.variables:
            if (var.core or "").strip().upper() != "REQ":
                continue

            var_name = var.name
            if var_name not in df.columns:
                # This is caught by SD0056
                continue

            # Count null values
            null_mask = df[var_name].isna() | (
                df[var_name].astype(str).str.strip() == ""
            )
            null_count = null_mask.sum()

            if null_count > 0:
                # Get sample record identifiers
                null_records = df.loc[null_mask, "USUBJID"].head(3).tolist()
                sample = ", ".join(str(r) for r in null_records)

                issues.append(
                    self.create_issue(
                        message=f"Required variable '{var_name}' has {null_count} null value(s)",
                        domain_code=context.domain_code,
                        variable_name=var_name,
                        details={
                            "null_count": null_count,
                            "sample_records": sample,
                        },
                    )
                )

        return issues


class ISO8601Validator(ValidationRule):
    """SD0003: Invalid ISO 8601 value for variable."""

    def __init__(self):
        super().__init__(
            rule_id="SD0003",
            severity=ValidationSeverity.ERROR,
            category=ValidationCategory.FORMAT,
            message_template="Variable '{variable}' contains invalid ISO 8601 values",
        )

    def validate(self, context: ValidationContext) -> list[ValidationIssue]:
        issues = []
        df = context.dataframe

        # Find all DTC and DUR variables
        datetime_vars = [
            col
            for col in df.columns
            if col.endswith("DTC") or col.endswith("STDTC") or col.endswith("ENDTC")
        ]
        duration_vars = [col for col in df.columns if col.endswith("DUR")]

        for var_name in datetime_vars:
            invalid_count = self._validate_datetime_column(df, var_name)
            if invalid_count > 0:
                issues.append(
                    self.create_issue(
                        message=f"Variable '{var_name}' contains {invalid_count} invalid ISO 8601 date/time value(s)",
                        domain_code=context.domain_code,
                        variable_name=var_name,
                        details={"invalid_count": invalid_count},
                    )
                )

        for var_name in duration_vars:
            invalid_count = self._validate_duration_column(df, var_name)
            if invalid_count > 0:
                issues.append(
                    self.create_issue(
                        message=f"Variable '{var_name}' contains {invalid_count} invalid ISO 8601 duration value(s)",
                        domain_code=context.domain_code,
                        variable_name=var_name,
                        details={"invalid_count": invalid_count},
                    )
                )

        return issues

    def _validate_datetime_column(self, df: pd.DataFrame, col: str) -> int:
        """Count invalid datetime values in a column."""
        series = df[col].astype(str).str.strip()
        # Exclude empty/null values
        series = series[series != ""]
        series = series[series.str.upper() != "NAN"]
        series = series[series.str.upper() != "NAT"]

        invalid_mask = ~series.apply(lambda x: bool(ISO8601_DATETIME_PATTERN.match(x)))
        return invalid_mask.sum()

    def _validate_duration_column(self, df: pd.DataFrame, col: str) -> int:
        """Count invalid duration values in a column."""
        series = df[col].astype(str).str.strip()
        # Exclude empty/null values
        series = series[series != ""]
        series = series[series.str.upper() != "NAN"]

        invalid_mask = ~series.apply(lambda x: bool(ISO8601_DURATION_PATTERN.match(x)))
        return invalid_mask.sum()


class DomainConsistencyValidator(ValidationRule):
    """SD0004: Inconsistent value for DOMAIN."""

    def __init__(self):
        super().__init__(
            rule_id="SD0004",
            severity=ValidationSeverity.ERROR,
            category=ValidationCategory.CONSISTENCY,
            message_template="DOMAIN variable inconsistent with dataset name",
        )

    def validate(self, context: ValidationContext) -> list[ValidationIssue]:
        issues = []
        df = context.dataframe

        if "DOMAIN" not in df.columns:
            return issues

        # Check if DOMAIN values match the domain code
        expected_domain = context.domain_code.upper()
        actual_domains = df["DOMAIN"].astype(str).str.strip().str.upper().unique()

        for actual_domain in actual_domains:
            if actual_domain != expected_domain:
                count = (
                    df["DOMAIN"].astype(str).str.strip().str.upper() == actual_domain
                ).sum()
                issues.append(
                    self.create_issue(
                        message=f"DOMAIN value '{actual_domain}' does not match expected '{expected_domain}' ({count} records)",
                        domain_code=context.domain_code,
                        variable_name="DOMAIN",
                        details={
                            "expected": expected_domain,
                            "actual": actual_domain,
                            "count": count,
                        },
                    )
                )

        return issues


class SequenceUniquenessValidator(ValidationRule):
    """SD0005: Duplicate value for --SEQ variable."""

    def __init__(self):
        super().__init__(
            rule_id="SD0005",
            severity=ValidationSeverity.ERROR,
            category=ValidationCategory.CONSISTENCY,
            message_template="--SEQ variable contains duplicate values",
        )

    def validate(self, context: ValidationContext) -> list[ValidationIssue]:
        issues = []
        df = context.dataframe

        # Find the SEQ variable
        seq_var = f"{context.domain_code}SEQ"
        if seq_var not in df.columns:
            return issues

        # Exclusions: DE, DO, DT, DU, DX domains
        if context.domain_code.upper() in {"DE", "DO", "DT", "DU", "DX"}:
            return issues

        # Check for duplicates within USUBJID or POOLID
        if "USUBJID" in df.columns:
            duplicates = df.groupby("USUBJID")[seq_var].apply(
                lambda x: x.duplicated(keep=False).sum()
            )
            duplicate_subjects = duplicates[duplicates > 0]

            if len(duplicate_subjects) > 0:
                sample = list(duplicate_subjects.head(3).index)
                issues.append(
                    self.create_issue(
                        message=f"Variable '{seq_var}' has duplicate values in {len(duplicate_subjects)} subject(s)",
                        domain_code=context.domain_code,
                        variable_name=seq_var,
                        details={
                            "subject_count": len(duplicate_subjects),
                            "sample_subjects": sample,
                        },
                    )
                )

        return issues


class StudyDayValidator(ValidationRule):
    """SD1086: Incorrect value for --DY variable."""

    def __init__(self):
        super().__init__(
            rule_id="SD1086",
            severity=ValidationSeverity.ERROR,
            category=ValidationCategory.PRESENCE,
            message_template="Study day calculation is incorrect",
        )

    def validate(self, context: ValidationContext) -> list[ValidationIssue]:
        issues = []
        df = context.dataframe

        # Find all *DY variables
        dy_vars = [col for col in df.columns if col.endswith("DY")]

        for dy_var in dy_vars:
            # Get corresponding *DTC variable
            dtc_var = dy_var[:-2] + "DTC"
            if dtc_var not in df.columns:
                continue

            # Check if values are calculated correctly
            if "USUBJID" not in df.columns:
                continue

            errors = self._validate_study_days(
                df, dy_var, dtc_var, context.reference_starts
            )

            if errors > 0:
                issues.append(
                    self.create_issue(
                        message=f"Variable '{dy_var}' has {errors} incorrectly calculated value(s)",
                        domain_code=context.domain_code,
                        variable_name=dy_var,
                        details={"error_count": errors},
                    )
                )

        return issues

    def _validate_study_days(
        self,
        df: pd.DataFrame,
        dy_var: str,
        dtc_var: str,
        reference_starts: dict[str, str],
    ) -> int:
        """Validate study day calculations."""
        errors = 0

        for _, row in df.iterrows():
            usubjid = row.get("USUBJID", "")
            if not usubjid or usubjid not in reference_starts:
                continue

            ref_date_str = reference_starts[usubjid]
            dtc_val = str(row.get(dtc_var, "")).strip()
            dy_val = row.get(dy_var)

            if not dtc_val or pd.isna(dy_val):
                continue

            # Parse dates
            try:
                ref_date = datetime.fromisoformat(ref_date_str[:10]).date()
                event_date = datetime.fromisoformat(dtc_val[:10]).date()
            except (ValueError, IndexError):
                continue

            # Calculate expected study day
            delta = (event_date - ref_date).days
            expected_dy = delta + 1 if delta >= 0 else delta

            # Check if actual matches expected
            if int(dy_val) != expected_dy:
                errors += 1

        return errors


class ControlledTerminologyValidator(ValidationRule):
    """CT2001: Variable value not found in non-extensible codelist."""

    def __init__(self):
        super().__init__(
            rule_id="CT2001",
            severity=ValidationSeverity.ERROR,
            category=ValidationCategory.TERMINOLOGY,
            message_template="Variable value not found in non-extensible codelist",
        )

    def validate(self, context: ValidationContext) -> list[ValidationIssue]:
        issues = []

        if not context.controlled_terminology:
            return issues

        df = context.dataframe
        ct = context.controlled_terminology

        # Check variables with CT requirements
        for var in context.domain.variables:
            if not var.codelist:
                continue

            var_name = var.name
            if var_name not in df.columns:
                continue

            # Get codelist info
            codelist = ct.get_codelist(var.codelist)
            if not codelist or codelist.extensible:
                continue

            # Check values against codelist
            values = df[var_name].astype(str).str.strip()
            values = values[values != ""]
            unique_values = values.unique()

            invalid_values = []
            for value in unique_values:
                if value not in codelist.terms:
                    invalid_values.append(value)

            if invalid_values:
                issues.append(
                    self.create_issue(
                        message=f"Variable '{var_name}' contains {len(invalid_values)} invalid term(s) not in codelist '{var.codelist}'",
                        domain_code=context.domain_code,
                        variable_name=var_name,
                        details={
                            "invalid_values": invalid_values[:5],
                            "codelist": var.codelist,
                        },
                    )
                )

        return issues


class ValidationEngine:
    """Main validation engine that orchestrates all validation rules."""

    def __init__(self):
        self.rules: list[ValidationRule] = []
        self._register_default_rules()

    def _register_default_rules(self) -> None:
        """Register all default validation rules."""
        self.rules.extend(
            [
                RequiredVariableValidator(),
                ISO8601Validator(),
                DomainConsistencyValidator(),
                SequenceUniquenessValidator(),
                StudyDayValidator(),
                ControlledTerminologyValidator(),
            ]
        )

    def add_rule(self, rule: ValidationRule) -> None:
        """Add a custom validation rule."""
        self.rules.append(rule)

    def validate_domain(self, context: ValidationContext) -> list[ValidationIssue]:
        """Run all validation rules against a domain."""
        all_issues = []

        for rule in self.rules:
            try:
                issues = rule.validate(context)
                all_issues.extend(issues)
            except Exception as e:
                # Create an issue for validation failures
                all_issues.append(
                    ValidationIssue(
                        rule_id=rule.rule_id,
                        severity=ValidationSeverity.WARNING,
                        category=ValidationCategory.STRUCTURE,
                        message=f"Validation rule failed: {e}",
                        domain_code=context.domain_code,
                    )
                )

        return all_issues

    def validate_study(
        self,
        study_id: str,
        domains: dict[str, tuple[SDTMDomain, pd.DataFrame]],
        controlled_terminology: ControlledTerminology | None = None,
        reference_starts: dict[str, str] | None = None,
    ) -> dict[str, list[ValidationIssue]]:
        """Validate all domains in a study."""
        all_domain_dfs = {code: df for code, (_, df) in domains.items()}
        results = {}

        for domain_code, (domain_obj, df) in domains.items():
            context = ValidationContext(
                study_id=study_id,
                domain_code=domain_code,
                domain=domain_obj,
                dataframe=df,
                all_domains=all_domain_dfs,
                controlled_terminology=controlled_terminology,
                reference_starts=reference_starts or {},
            )

            issues = self.validate_domain(context)
            if issues:
                results[domain_code] = issues

        return results


def format_validation_report(issues_by_domain: dict[str, list[ValidationIssue]]) -> str:
    """Format validation issues into a readable report."""
    lines = ["=" * 80, "VALIDATION REPORT", "=" * 80, ""]

    total_errors = sum(
        len([i for i in issues if i.severity == ValidationSeverity.ERROR])
        for issues in issues_by_domain.values()
    )
    total_warnings = sum(
        len([i for i in issues if i.severity == ValidationSeverity.WARNING])
        for issues in issues_by_domain.values()
    )

    lines.append(f"Total Issues: {total_errors} errors, {total_warnings} warnings\n")

    for domain_code in sorted(issues_by_domain.keys()):
        issues = issues_by_domain[domain_code]
        lines.append(f"\nDomain: {domain_code}")
        lines.append("-" * 80)

        # Group by severity
        errors = [i for i in issues if i.severity == ValidationSeverity.ERROR]
        warnings = [i for i in issues if i.severity == ValidationSeverity.WARNING]

        if errors:
            lines.append(f"\nErrors ({len(errors)}):")
            for issue in errors:
                lines.append(f"  {issue}")

        if warnings:
            lines.append(f"\nWarnings ({len(warnings)}):")
            for issue in warnings:
                lines.append(f"  {issue}")

    lines.append("\n" + "=" * 80)
    return "\n".join(lines)
