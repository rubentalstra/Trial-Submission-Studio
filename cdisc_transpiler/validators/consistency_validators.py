"""Consistency and limit validators for SDTM compliance.

These validators check data consistency and value limits:
- SD0012-SD0013: Start/end date consistency (--STDY <= --ENDY, --STDTC <= --ENDTC)
- SD0014-SD0015: Non-negative values (--DOSE, --DUR >= 0)
- SD0024-SD0025: Collection date consistency
- SD0028: Reference range consistency (--STNRLO <= --STNRHI)
- SD0038: Study day variables != 0
- SD0084: AGE > 0
- SD1002: RFSTDTC <= RFENDTC
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd

if TYPE_CHECKING:
    from .validators import ValidationContext, ValidationIssue


class DateTimeConsistencyValidator:
    """SD0012, SD0013: Start date/time must be before or equal to end date/time."""

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate temporal consistency across all date/time pairs."""
        issues = []
        df = context.dataframe

        # Find all STDY/ENDY pairs
        dy_pairs = []
        for col in df.columns:
            if col.endswith("STDY"):
                prefix = col[:-4]  # Remove 'STDY'
                endy_col = f"{prefix}ENDY"
                if endy_col in df.columns:
                    dy_pairs.append((col, endy_col))

        # Find all STDTC/ENDTC pairs
        dtc_pairs = []
        for col in df.columns:
            if col.endswith("STDTC"):
                prefix = col[:-5]  # Remove 'STDTC'
                endtc_col = f"{prefix}ENDTC"
                if endtc_col in df.columns:
                    dtc_pairs.append((col, endtc_col))

        # Special case: --DTC with --ENDTC
        for col in df.columns:
            if (
                col.endswith("DTC")
                and not col.endswith("STDTC")
                and not col.endswith("ENDTC")
            ):
                prefix = col[:-3]
                endtc_col = f"{prefix}ENDTC"
                if endtc_col in df.columns:
                    dtc_pairs.append((col, endtc_col))

        # Validate study day pairs (SD0012)
        for stdy_col, endy_col in dy_pairs:
            violations = DateTimeConsistencyValidator._check_numeric_order(
                df, stdy_col, endy_col
            )
            if violations > 0:
                from .validators import (
                    ValidationIssue,
                    ValidationSeverity,
                    ValidationCategory,
                )

                issues.append(
                    ValidationIssue(
                        rule_id="SD0012",
                        severity=ValidationSeverity.ERROR,
                        category=ValidationCategory.LIMIT,
                        message=f"{stdy_col} is after {endy_col} in {violations} record(s)",
                        domain_code=context.domain_code,
                        variable_name=f"{stdy_col}/{endy_col}",
                        details={"violation_count": violations},
                    )
                )

        # Validate datetime pairs (SD0013, SD0025)
        for std_col, end_col in dtc_pairs:
            violations = DateTimeConsistencyValidator._check_datetime_order(
                df, std_col, end_col
            )
            if violations > 0:
                from .validators import (
                    ValidationIssue,
                    ValidationSeverity,
                    ValidationCategory,
                )

                rule_id = (
                    "SD0025"
                    if std_col.endswith("DTC") and not std_col.endswith("STDTC")
                    else "SD0013"
                )

                issues.append(
                    ValidationIssue(
                        rule_id=rule_id,
                        severity=ValidationSeverity.ERROR,
                        category=ValidationCategory.LIMIT,
                        message=f"{std_col} is after {end_col} in {violations} record(s)",
                        domain_code=context.domain_code,
                        variable_name=f"{std_col}/{end_col}",
                        details={"violation_count": violations},
                    )
                )

        return issues

    @staticmethod
    def _check_numeric_order(df: pd.DataFrame, start_col: str, end_col: str) -> int:
        """Check if start <= end for numeric columns."""
        start_vals = pd.to_numeric(df[start_col], errors="coerce")
        end_vals = pd.to_numeric(df[end_col], errors="coerce")

        # Only check where both are non-null
        mask = start_vals.notna() & end_vals.notna()
        violations = (start_vals[mask] > end_vals[mask]).sum()

        return violations

    @staticmethod
    def _check_datetime_order(df: pd.DataFrame, start_col: str, end_col: str) -> int:
        """Check if start <= end for datetime columns."""
        violations = 0

        for _, row in df.iterrows():
            start_val = str(row.get(start_col, "")).strip()
            end_val = str(row.get(end_col, "")).strip()

            if not start_val or not end_val:
                continue

            # Extract comparable date portion (YYYY-MM-DD)
            start_date = start_val[:10] if len(start_val) >= 10 else start_val
            end_date = end_val[:10] if len(end_val) >= 10 else end_val

            # Simple string comparison for ISO 8601 dates
            if start_date > end_date:
                violations += 1

        return violations


class NonNegativeValueValidator:
    """SD0014, SD0015: Values must be >= 0 for dose and duration variables."""

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate non-negative values for dose and duration variables."""
        issues = []
        df = context.dataframe

        # Find --DOSE variables
        dose_vars = [
            col for col in df.columns if col.endswith("DOSE") and col != "DOSE"
        ]

        # Find --DUR variables
        dur_vars = [col for col in df.columns if col.endswith("DUR")]

        # Check dose variables (SD0014)
        for dose_var in dose_vars:
            negative_count = NonNegativeValueValidator._count_negative(df, dose_var)
            if negative_count > 0:
                from .validators import (
                    ValidationIssue,
                    ValidationSeverity,
                    ValidationCategory,
                )

                issues.append(
                    ValidationIssue(
                        rule_id="SD0014",
                        severity=ValidationSeverity.ERROR,
                        category=ValidationCategory.LIMIT,
                        message=f"{dose_var} has {negative_count} negative value(s)",
                        domain_code=context.domain_code,
                        variable_name=dose_var,
                        details={"negative_count": negative_count},
                    )
                )

        # Check duration variables (SD0015)
        for dur_var in dur_vars:
            negative_count = NonNegativeValueValidator._count_negative(df, dur_var)
            if negative_count > 0:
                from .validators import (
                    ValidationIssue,
                    ValidationSeverity,
                    ValidationCategory,
                )

                issues.append(
                    ValidationIssue(
                        rule_id="SD0015",
                        severity=ValidationSeverity.ERROR,
                        category=ValidationCategory.LIMIT,
                        message=f"{dur_var} has {negative_count} negative value(s)",
                        domain_code=context.domain_code,
                        variable_name=dur_var,
                        details={"negative_count": negative_count},
                    )
                )

        return issues

    @staticmethod
    def _count_negative(df: pd.DataFrame, col: str) -> int:
        """Count negative values in a numeric column."""
        values = pd.to_numeric(df[col], errors="coerce")
        # Count non-null negative values
        return (values.notna() & (values < 0)).sum()


class ReferenceRangeValidator:
    """SD0028: Reference range upper limit must be >= lower limit."""

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate reference range consistency."""
        issues = []
        df = context.dataframe

        # Find all STNRLO/STNRHI pairs
        range_pairs = []
        for col in df.columns:
            if col.endswith("STNRLO"):
                prefix = col[:-7]  # Remove 'STNRLO'
                hi_col = f"{prefix}STNRHI"
                if hi_col in df.columns:
                    range_pairs.append((col, hi_col))

        # Also check ORNRLO/ORNRHI pairs
        for col in df.columns:
            if col.endswith("ORNRLO"):
                prefix = col[:-7]  # Remove 'ORNRLO'
                hi_col = f"{prefix}ORNRHI"
                if hi_col in df.columns:
                    range_pairs.append((col, hi_col))

        for lo_col, hi_col in range_pairs:
            violations = ReferenceRangeValidator._check_range_order(df, lo_col, hi_col)
            if violations > 0:
                from .validators import (
                    ValidationIssue,
                    ValidationSeverity,
                    ValidationCategory,
                )

                issues.append(
                    ValidationIssue(
                        rule_id="SD0028",
                        severity=ValidationSeverity.ERROR,
                        category=ValidationCategory.LIMIT,
                        message=f"{hi_col} is less than {lo_col} in {violations} record(s)",
                        domain_code=context.domain_code,
                        variable_name=f"{lo_col}/{hi_col}",
                        details={"violation_count": violations},
                    )
                )

        return issues

    @staticmethod
    def _check_range_order(df: pd.DataFrame, lo_col: str, hi_col: str) -> int:
        """Check if lower <= upper for range values."""
        lo_vals = pd.to_numeric(df[lo_col], errors="coerce")
        hi_vals = pd.to_numeric(df[hi_col], errors="coerce")

        # Only check where both are non-null
        mask = lo_vals.notna() & hi_vals.notna()
        violations = (lo_vals[mask] > hi_vals[mask]).sum()

        return violations


class StudyDayZeroValidator:
    """SD0038: Study day variables should not equal 0."""

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate study day variables don't have zero values."""
        issues = []
        df = context.dataframe

        # Find all *DY variables
        dy_vars = [col for col in df.columns if col.endswith("DY")]

        for dy_var in dy_vars:
            zero_count = (pd.to_numeric(df[dy_var], errors="coerce") == 0).sum()

            if zero_count > 0:
                from .validators import (
                    ValidationIssue,
                    ValidationSeverity,
                    ValidationCategory,
                )

                issues.append(
                    ValidationIssue(
                        rule_id="SD0038",
                        severity=ValidationSeverity.ERROR,
                        category=ValidationCategory.LIMIT,
                        message=f"{dy_var} has {zero_count} value(s) equal to 0",
                        domain_code=context.domain_code,
                        variable_name=dy_var,
                        details={"zero_count": zero_count},
                    )
                )

        return issues


class AgeValidator:
    """SD0084: Age must be greater than 0."""

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate AGE is positive."""
        issues = []
        df = context.dataframe

        if "AGE" not in df.columns:
            return issues

        age_vals = pd.to_numeric(df["AGE"], errors="coerce")
        # Count non-null values <= 0
        invalid_count = (age_vals.notna() & (age_vals <= 0)).sum()

        if invalid_count > 0:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            issues.append(
                ValidationIssue(
                    rule_id="SD0084",
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.LIMIT,
                    message=f"AGE has {invalid_count} value(s) <= 0",
                    domain_code=context.domain_code,
                    variable_name="AGE",
                    details={"invalid_count": invalid_count},
                )
            )

        return issues


class ReferenceStartEndValidator:
    """SD1002: RFSTDTC must be before or equal to RFENDTC."""

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate RFSTDTC <= RFENDTC."""
        issues = []

        # Only apply to DM domain
        if context.domain_code.upper() != "DM":
            return issues

        df = context.dataframe

        if "RFSTDTC" not in df.columns or "RFENDTC" not in df.columns:
            return issues

        violations = 0
        for _, row in df.iterrows():
            rfstdtc = str(row.get("RFSTDTC", "")).strip()
            rfendtc = str(row.get("RFENDTC", "")).strip()

            if not rfstdtc or not rfendtc:
                continue

            # Extract comparable date portion
            start_date = rfstdtc[:10] if len(rfstdtc) >= 10 else rfstdtc
            end_date = rfendtc[:10] if len(rfendtc) >= 10 else rfendtc

            if start_date > end_date:
                violations += 1

        if violations > 0:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            issues.append(
                ValidationIssue(
                    rule_id="SD1002",
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.LIMIT,
                    message=f"RFSTDTC is after RFENDTC in {violations} record(s)",
                    domain_code="DM",
                    variable_name="RFSTDTC/RFENDTC",
                    details={"violation_count": violations},
                )
            )

        return issues


class PairedVariableConsistencyValidator:
    """SD0040, SD0051, SD0052: Consistency checks for paired variables."""

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate consistency of paired variables."""
        issues = []
        df = context.dataframe

        # SD0040: --TEST should be consistent within --TESTCD
        issues.extend(
            PairedVariableConsistencyValidator._check_testcd_test_consistency(
                df, context.domain_code
            )
        )

        # SD0051: VISIT should be consistent within VISITNUM
        issues.extend(
            PairedVariableConsistencyValidator._check_visit_consistency(
                df, context.domain_code
            )
        )

        # SD0052: VISITNUM should be consistent within VISIT
        issues.extend(
            PairedVariableConsistencyValidator._check_visitnum_consistency(
                df, context.domain_code
            )
        )

        return issues

    @staticmethod
    def _check_testcd_test_consistency(
        df: pd.DataFrame, domain_code: str
    ) -> list[ValidationIssue]:
        """Check if --TEST is consistent for each --TESTCD."""
        issues = []

        # Find TESTCD and TEST variables
        testcd_var = f"{domain_code}TESTCD"
        test_var = f"{domain_code}TEST"

        if testcd_var not in df.columns or test_var not in df.columns:
            return issues

        # Group by TESTCD and check TEST uniqueness
        grouped = df.groupby(testcd_var)[test_var].nunique()
        inconsistent = grouped[grouped > 1]

        if len(inconsistent) > 0:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            sample_codes = list(inconsistent.index[:5])
            issues.append(
                ValidationIssue(
                    rule_id="SD0040",
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.CONSISTENCY,
                    message=f"{test_var} has inconsistent values for {len(inconsistent)} {testcd_var} value(s)",
                    domain_code=domain_code,
                    variable_name=f"{testcd_var}/{test_var}",
                    details={
                        "inconsistent_count": len(inconsistent),
                        "sample_testcds": sample_codes,
                    },
                )
            )

        return issues

    @staticmethod
    def _check_visit_consistency(
        df: pd.DataFrame, domain_code: str
    ) -> list[ValidationIssue]:
        """Check if VISIT is consistent for each VISITNUM."""
        issues = []

        if "VISITNUM" not in df.columns or "VISIT" not in df.columns:
            return issues

        # Group by VISITNUM and check VISIT uniqueness
        grouped = df.groupby("VISITNUM")["VISIT"].nunique()
        inconsistent = grouped[grouped > 1]

        if len(inconsistent) > 0:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            sample_nums = list(inconsistent.index[:5])
            issues.append(
                ValidationIssue(
                    rule_id="SD0051",
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.CONSISTENCY,
                    message=f"VISIT has inconsistent values for {len(inconsistent)} VISITNUM value(s)",
                    domain_code=domain_code,
                    variable_name="VISITNUM/VISIT",
                    details={
                        "inconsistent_count": len(inconsistent),
                        "sample_visitnums": sample_nums,
                    },
                )
            )

        return issues

    @staticmethod
    def _check_visitnum_consistency(
        df: pd.DataFrame, domain_code: str
    ) -> list[ValidationIssue]:
        """Check if VISITNUM is consistent for each VISIT."""
        issues = []

        if "VISIT" not in df.columns or "VISITNUM" not in df.columns:
            return issues

        # Group by VISIT and check VISITNUM uniqueness
        grouped = df.groupby("VISIT")["VISITNUM"].nunique()
        inconsistent = grouped[grouped > 1]

        if len(inconsistent) > 0:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            sample_visits = list(inconsistent.index[:5])
            issues.append(
                ValidationIssue(
                    rule_id="SD0052",
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.CONSISTENCY,
                    message=f"VISITNUM has inconsistent values for {len(inconsistent)} VISIT value(s)",
                    domain_code=domain_code,
                    variable_name="VISIT/VISITNUM",
                    details={
                        "inconsistent_count": len(inconsistent),
                        "sample_visits": sample_visits,
                    },
                )
            )

        return issues


def register_consistency_validators(engine) -> None:
    """Register all consistency and limit validators with the validation engine."""
    from .validators import ValidationRule

    # Create wrapper classes for each validator
    class DateTimeConsistencyRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id="SD0012",
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return DateTimeConsistencyValidator.validate(context)

    class NonNegativeValueRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id="SD0014",
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return NonNegativeValueValidator.validate(context)

    class ReferenceRangeRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id="SD0028",
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return ReferenceRangeValidator.validate(context)

    class StudyDayZeroRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id="SD0038",
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return StudyDayZeroValidator.validate(context)

    class AgeRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id="SD0084",
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return AgeValidator.validate(context)

    class ReferenceStartEndRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id="SD1002",
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return ReferenceStartEndValidator.validate(context)

    class PairedVariableConsistencyRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id="SD0040",
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return PairedVariableConsistencyValidator.validate(context)

    # Add all rules
    engine.add_rule(DateTimeConsistencyRule())
    engine.add_rule(NonNegativeValueRule())
    engine.add_rule(ReferenceRangeRule())
    engine.add_rule(StudyDayZeroRule())
    engine.add_rule(AgeRule())
    engine.add_rule(ReferenceStartEndRule())
    engine.add_rule(PairedVariableConsistencyRule())
