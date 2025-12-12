"""Cross-domain reference validators for SDTM compliance.

These validators check referential integrity across domains:
- SD0064: Subject presence in DM
- SD0065: USUBJID/VISIT/VISITNUM in SV
- SD0066-SD0071: ARM/ARMCD validation
- SD0072: RDOMAIN validation
- SD0077: Referenced record validation
- SD0083: Duplicate USUBJID in DM
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd

if TYPE_CHECKING:
    from .validators import ValidationContext, ValidationIssue, ValidationRule


class SubjectInDMValidator:
    """SD0064: Subject is not present in DM domain."""

    rule_id = "SD0064"

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate all subjects exist in DM domain."""
        issues = []

        # Skip if this IS the DM domain
        if context.domain_code.upper() == "DM":
            return issues

        # Get DM domain
        dm_df = context.all_domains.get("DM")
        if dm_df is None or dm_df.empty:
            # Can't validate without DM
            return issues

        if "USUBJID" not in context.dataframe.columns:
            return issues

        if "USUBJID" not in dm_df.columns:
            return issues

        # Get unique subjects in current domain
        domain_subjects = set(
            context.dataframe["USUBJID"].dropna().astype(str).str.strip()
        )
        domain_subjects = {s for s in domain_subjects if s}

        # Get subjects in DM
        dm_subjects = set(dm_df["USUBJID"].dropna().astype(str).str.strip())
        dm_subjects = {s for s in dm_subjects if s}

        # Find subjects not in DM
        missing_subjects = domain_subjects - dm_subjects

        if missing_subjects:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            sample = list(missing_subjects)[:10]
            issues.append(
                ValidationIssue(
                    rule_id=SubjectInDMValidator.rule_id,
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.CROSS_REFERENCE,
                    message=f"{len(missing_subjects)} subject(s) in {context.domain_code} not found in DM domain",
                    domain_code=context.domain_code,
                    variable_name="USUBJID",
                    details={
                        "missing_count": len(missing_subjects),
                        "sample_subjects": sample,
                    },
                )
            )

        return issues


class VisitInSVValidator:
    """SD0065: USUBJID/VISIT/VISITNUM values do not match SV domain data."""

    rule_id = "SD0065"

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate visit combinations exist in SV domain."""
        issues = []

        # Skip for domains that don't need SV validation
        if context.domain_code.upper() in {"DM", "SV", "TA", "TE", "TI", "TS", "TV"}:
            return issues

        # Get SV domain
        sv_df = context.all_domains.get("SV")
        if sv_df is None or sv_df.empty:
            # Can't validate without SV
            return issues

        # Need USUBJID, VISIT, or VISITNUM
        if "USUBJID" not in context.dataframe.columns:
            return issues

        # Check if we have visit info
        has_visit = "VISIT" in context.dataframe.columns
        has_visitnum = "VISITNUM" in context.dataframe.columns

        if not (has_visit or has_visitnum):
            return issues

        # Build SV lookup
        sv_keys = set()
        for _, row in sv_df.iterrows():
            usubjid = str(row.get("USUBJID", "")).strip()
            if not usubjid:
                continue

            visit = str(row.get("VISIT", "")).strip()
            visitnum = row.get("VISITNUM")

            if has_visit and has_visitnum:
                key = (usubjid, visit, visitnum)
            elif has_visit:
                key = (usubjid, visit)
            else:
                key = (usubjid, visitnum)

            sv_keys.add(key)

        # Check domain records
        missing_count = 0
        for _, row in context.dataframe.iterrows():
            # Skip records with --STAT = 'NOT DONE'
            stat_col = f"{context.domain_code}STAT"
            if stat_col in context.dataframe.columns:
                if str(row.get(stat_col, "")).strip().upper() == "NOT DONE":
                    continue

            # Skip records with OCCUR = 'N' (for EC domain)
            if context.domain_code.upper() == "EC":
                if str(row.get("ECOCCUR", "")).strip().upper() == "N":
                    continue

            usubjid = str(row.get("USUBJID", "")).strip()
            if not usubjid:
                continue

            visit = str(row.get("VISIT", "")).strip() if has_visit else None
            visitnum = row.get("VISITNUM") if has_visitnum else None

            # Build key for this record
            if has_visit and has_visitnum:
                key = (usubjid, visit, visitnum)
            elif has_visit:
                key = (usubjid, visit)
            else:
                key = (usubjid, visitnum)

            if key not in sv_keys:
                missing_count += 1

        if missing_count > 0:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            issues.append(
                ValidationIssue(
                    rule_id=VisitInSVValidator.rule_id,
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.CROSS_REFERENCE,
                    message=f"{missing_count} USUBJID/VISIT/VISITNUM combination(s) not found in SV domain",
                    domain_code=context.domain_code,
                    details={"missing_count": missing_count},
                )
            )

        return issues


class ARMCDValidator:
    """SD0066: Invalid ARMCD.

    ARMCD values should match entries in TA dataset, except for
    SCRNFAIL, NOTASSGN.
    """

    rule_id = "SD0066"

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate ARMCD against TA domain."""
        issues = []

        if "ARMCD" not in context.dataframe.columns:
            return issues

        # Get TA domain
        ta_df = context.all_domains.get("TA")
        if ta_df is None or ta_df.empty:
            # Can't validate without TA
            return issues

        if "ARMCD" not in ta_df.columns:
            return issues

        # Get valid ARMCD values from TA
        valid_armcds = set(ta_df["ARMCD"].dropna().astype(str).str.strip().str.upper())

        # Add special exemptions
        exemptions = {"SCRNFAIL", "NOTASSGN", ""}
        valid_armcds.update(exemptions)

        # Check ARMCD values in current domain
        domain_armcds = context.dataframe["ARMCD"].dropna().astype(str).str.strip().str.upper().unique()

        invalid_armcds = [a for a in domain_armcds if a and a not in valid_armcds]

        if invalid_armcds:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            issues.append(
                ValidationIssue(
                    rule_id=ARMCDValidator.rule_id,
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.CROSS_REFERENCE,
                    message=f"ARMCD contains {len(invalid_armcds)} invalid value(s) not found in TA",
                    domain_code=context.domain_code,
                    variable_name="ARMCD",
                    details={
                        "invalid_armcds": invalid_armcds[:10],
                    },
                )
            )

        return issues


class RDOMAINValidator:
    """SD0072: Invalid RDOMAIN.

    RDOMAIN must have valid values of domains included in study data.
    """

    rule_id = "SD0072"

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate RDOMAIN references valid domains."""
        issues = []

        if "RDOMAIN" not in context.dataframe.columns:
            return issues

        # Get all available domain codes
        valid_domains = set(context.all_domains.keys())
        valid_domains = {d.upper() for d in valid_domains}

        # Check RDOMAIN values
        rdomain_values = context.dataframe["RDOMAIN"].dropna().astype(str).str.strip().str.upper().unique()
        rdomain_values = [r for r in rdomain_values if r]

        invalid_rdomains = [r for r in rdomain_values if r not in valid_domains]

        if invalid_rdomains:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            issues.append(
                ValidationIssue(
                    rule_id=RDOMAINValidator.rule_id,
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.CONSISTENCY,
                    message=f"RDOMAIN contains {len(invalid_rdomains)} invalid domain reference(s)",
                    domain_code=context.domain_code,
                    variable_name="RDOMAIN",
                    details={
                        "invalid_domains": invalid_rdomains,
                        "valid_domains": sorted(valid_domains),
                    },
                )
            )

        return issues


class IDVARValidator:
    """SD0075: Invalid IDVAR.

    IDVAR must reference valid variables from the referenced domain.
    """

    rule_id = "SD0075"

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate IDVAR references valid variables."""
        issues = []

        if "IDVAR" not in context.dataframe.columns:
            return issues

        if "RDOMAIN" not in context.dataframe.columns:
            return issues

        # Check each RDOMAIN/IDVAR pair
        invalid_count = 0
        for _, row in context.dataframe.iterrows():
            rdomain = str(row.get("RDOMAIN", "")).strip().upper()
            idvar = str(row.get("IDVAR", "")).strip().upper()

            if not rdomain or not idvar:
                continue

            # Get referenced domain
            ref_domain_df = context.all_domains.get(rdomain)
            if ref_domain_df is None:
                # RDOMAIN invalid - caught by SD0072
                continue

            # Check if IDVAR exists in referenced domain
            ref_columns = {str(c).upper() for c in ref_domain_df.columns}
            if idvar not in ref_columns:
                invalid_count += 1

        if invalid_count > 0:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            issues.append(
                ValidationIssue(
                    rule_id=IDVARValidator.rule_id,
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.CONSISTENCY,
                    message=f"IDVAR contains {invalid_count} invalid variable reference(s)",
                    domain_code=context.domain_code,
                    variable_name="IDVAR",
                    details={"invalid_count": invalid_count},
                )
            )

        return issues


class ReferencedRecordValidator:
    """SD0077: Invalid referenced record.

    Reference record defined by RDOMAIN, USUBJID, IDVAR, IDVARVAL
    must exist in target domain.
    """

    rule_id = "SD0077"

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate referenced records exist."""
        issues = []

        required_cols = {"RDOMAIN", "USUBJID", "IDVAR", "IDVARVAL"}
        if not required_cols.issubset(context.dataframe.columns):
            return issues

        # Check each reference
        missing_count = 0
        for _, row in context.dataframe.iterrows():
            rdomain = str(row.get("RDOMAIN", "")).strip().upper()
            usubjid = str(row.get("USUBJID", "")).strip()
            idvar = str(row.get("IDVAR", "")).strip().upper()
            idvarval = str(row.get("IDVARVAL", "")).strip()

            if not all([rdomain, usubjid, idvar, idvarval]):
                continue

            # Get referenced domain
            ref_domain_df = context.all_domains.get(rdomain)
            if ref_domain_df is None:
                # RDOMAIN invalid - caught by SD0072
                continue

            # Check if IDVAR exists
            if idvar not in {str(c).upper() for c in ref_domain_df.columns}:
                # IDVAR invalid - caught by SD0075
                continue

            # Find matching record
            if "USUBJID" not in ref_domain_df.columns:
                continue

            # Build mask for matching record
            usubjid_match = ref_domain_df["USUBJID"].astype(str).str.strip() == usubjid

            # Get the IDVAR column (case-insensitive)
            idvar_col = None
            for col in ref_domain_df.columns:
                if str(col).upper() == idvar:
                    idvar_col = col
                    break

            if idvar_col is None:
                continue

            idvarval_match = ref_domain_df[idvar_col].astype(str).str.strip() == idvarval

            # Check if record exists
            if not (usubjid_match & idvarval_match).any():
                missing_count += 1

        if missing_count > 0:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            issues.append(
                ValidationIssue(
                    rule_id=ReferencedRecordValidator.rule_id,
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.CROSS_REFERENCE,
                    message=f"{missing_count} referenced record(s) not found in target domain",
                    domain_code=context.domain_code,
                    details={"missing_count": missing_count},
                )
            )

        return issues


class DuplicateUSUBJIDValidator:
    """SD0083: Duplicate USUBJID in DM within STUDYID.

    USUBJID must be unique for each subject in DM domain.
    """

    rule_id = "SD0083"

    @staticmethod
    def validate(context: ValidationContext) -> list[ValidationIssue]:
        """Validate USUBJID uniqueness in DM."""
        issues = []

        # Only apply to DM domain
        if context.domain_code.upper() != "DM":
            return issues

        if "USUBJID" not in context.dataframe.columns:
            return issues

        # Check for duplicates
        usubjid_counts = context.dataframe["USUBJID"].value_counts()
        duplicates = usubjid_counts[usubjid_counts > 1]

        if len(duplicates) > 0:
            from .validators import (
                ValidationIssue,
                ValidationSeverity,
                ValidationCategory,
            )

            duplicate_ids = list(duplicates.index[:10])

            issues.append(
                ValidationIssue(
                    rule_id=DuplicateUSUBJIDValidator.rule_id,
                    severity=ValidationSeverity.ERROR,
                    category=ValidationCategory.CONSISTENCY,
                    message=f"USUBJID has {len(duplicates)} duplicate value(s) in DM domain",
                    domain_code="DM",
                    variable_name="USUBJID",
                    details={
                        "duplicate_count": len(duplicates),
                        "sample_duplicates": duplicate_ids,
                    },
                )
            )

        return issues


def register_cross_domain_validators(engine: ValidationRule) -> None:
    """Register all cross-domain validators with the validation engine."""
    from .validators import ValidationRule

    # Create wrapper classes for each validator
    class SubjectInDMRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id=SubjectInDMValidator.rule_id,
                severity=None,  # Set by validator
                category=None,
                message_template="",
            )

        def validate(self, context):
            return SubjectInDMValidator.validate(context)

    class VisitInSVRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id=VisitInSVValidator.rule_id,
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return VisitInSVValidator.validate(context)

    class ARMCDRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id=ARMCDValidator.rule_id,
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return ARMCDValidator.validate(context)

    class RDOMAINRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id=RDOMAINValidator.rule_id,
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return RDOMAINValidator.validate(context)

    class IDVARRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id=IDVARValidator.rule_id,
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return IDVARValidator.validate(context)

    class ReferencedRecordRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id=ReferencedRecordValidator.rule_id,
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return ReferencedRecordValidator.validate(context)

    class DuplicateUSUBJIDRule(ValidationRule):
        def __init__(self):
            super().__init__(
                rule_id=DuplicateUSUBJIDValidator.rule_id,
                severity=None,
                category=None,
                message_template="",
            )

        def validate(self, context):
            return DuplicateUSUBJIDValidator.validate(context)

    # Add all rules
    engine.add_rule(SubjectInDMRule())
    engine.add_rule(VisitInSVRule())
    engine.add_rule(ARMCDRule())
    engine.add_rule(RDOMAINRule())
    engine.add_rule(IDVARRule())
    engine.add_rule(ReferencedRecordRule())
    engine.add_rule(DuplicateUSUBJIDRule())
