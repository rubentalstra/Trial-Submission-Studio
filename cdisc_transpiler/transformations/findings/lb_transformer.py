"""LB (Laboratory) domain transformer.

This module provides a transformer for converting wide-format laboratory data
to SDTM LB (Laboratory Test Results) long format using the generic WideToLongTransformer base class.

SDTM Reference:
    SDTMIG v3.4 Section 6.3.3 defines the LB domain structure:
    - Required variables: LBTESTCD, LBTEST, LBORRES, LBORRESU
    - Key qualifiers: LBORNRLO, LBORNRHI (normal ranges)
"""

from __future__ import annotations

from typing import Any

import pandas as pd

from ..base import TransformationContext, TransformationResult
from .wide_to_long import TestColumnPattern, WideToLongTransformer


class LBTransformer(WideToLongTransformer):
    """Transformer for LB (Laboratory Test Results) domain wide-to-long conversion.

    This transformer extends WideToLongTransformer to handle LB-specific patterns
    and output variables. It discovers test columns using multiple pattern formats for:
    - ORRES_* / *ORRES / "* result or finding in original units" : Original result values
    - ORRESU_* / "* unit" : Original units
    - ORNR_*_Lower / "* range (lower limit)" : Normal range lower limit
    - ORNR_*_Upper / "* range (upper limit)" : Normal range upper limit
    - TEST_* : Test labels

    Special handling:
    - LBORNRLO: Normal range lower limit
    - LBORNRHI: Normal range upper limit
    - LBDTC: Multiple date variations (blood/stool/urine sample dates)

    Example:
        >>> from cdisc_transpiler.terminology_module import normalize_testcd, get_testcd_label
        >>> transformer = LBTransformer(
        ...     test_code_normalizer=normalize_testcd,
        ...     test_label_getter=get_testcd_label
        ... )
        >>> context = TransformationContext(domain="LB", study_id="STUDY001")
        >>> result = transformer.transform(wide_df, context)
    """

    def __init__(
        self,
        test_code_normalizer=None,
        test_label_getter=None,
    ):
        """Initialize LB transformer with LB-specific patterns.

        Args:
            test_code_normalizer: Optional function to normalize test codes using CT
            test_label_getter: Optional function to get test labels from CT
        """
        # Define LB-specific column patterns (multiple formats)
        patterns = [
            # Original result patterns
            TestColumnPattern(
                pattern=r"^ORRES_([A-Za-z0-9]+)$",
                column_type="orres",
                description="Original result columns (ORRES_*)",
            ),
            TestColumnPattern(
                pattern=r"^([A-Za-z0-9]+)ORRES$",
                column_type="orres",
                description="Original result columns (*ORRES)",
            ),
            TestColumnPattern(
                pattern=r"^([A-Za-z0-9]+)\s+result or finding in original units$",
                column_type="orres",
                description="Original result columns (long format)",
            ),
            # Unit patterns
            TestColumnPattern(
                pattern=r"^ORRESU_([A-Za-z0-9]+)$",
                column_type="unit",
                description="Original unit columns (ORRESU_*)",
            ),
            TestColumnPattern(
                pattern=r"^([A-Za-z0-9]+)\s+unit(?:\s*-.*)?$",
                column_type="unit",
                description="Original unit columns (long format)",
            ),
            # Normal range patterns
            TestColumnPattern(
                pattern=r"^ORNR_([A-Za-z0-9]+)_Lower$",
                column_type="nrlo",
                description="Normal range lower limit (ORNR_*_Lower)",
            ),
            TestColumnPattern(
                pattern=r"^([A-Za-z0-9]+)\s+range \(lower limit\)$",
                column_type="nrlo",
                description="Normal range lower limit (long format)",
            ),
            TestColumnPattern(
                pattern=r"^ORNR_([A-Za-z0-9]+)_Upper$",
                column_type="nrhi",
                description="Normal range upper limit (ORNR_*_Upper)",
            ),
            TestColumnPattern(
                pattern=r"^([A-Za-z0-9]+)\s+range \(upper limit\)$",
                column_type="nrhi",
                description="Normal range upper limit (long format)",
            ),
            # Test label pattern
            TestColumnPattern(
                pattern=r"^TEST_([A-Za-z0-9]+)$",
                column_type="label",
                description="Test label columns (TEST_*)",
            ),
        ]

        # Define column name mappings for common variations
        column_renames = {
            "Subject Id": "USUBJID",
            "SubjectId": "USUBJID",
            "Event date": "LBDTC",
            "Event Date": "LBDTC",
            "EventDate": "LBDTC",
            "Date of blood sample": "LBDTC",
            "Date of stool sample": "LBDTC",
            "Date of urine sample": "LBDTC",
            "Date of pregnancy test": "LBDTC",
        }

        # Define output variable mapping
        output_mapping = {
            "TESTCD": "LBTESTCD",
            "TEST": "LBTEST",
            "ORRES": "LBORRES",
            "ORRESU": "LBORRESU",
        }

        super().__init__(
            domain="LB",
            patterns=patterns,
            column_renames=column_renames,
            output_mapping=output_mapping,
            test_code_normalizer=test_code_normalizer,
            test_label_getter=test_label_getter,
        )

    def _extract_row_identifiers(self, row: pd.Series) -> dict[str, Any]:
        """Extract LB-specific row identifiers.

        Extends base extraction to handle:
        - LBDTC with multiple date column candidates

        Args:
            row: Input row

        Returns:
            Dictionary of identifier values
        """
        identifiers = {}

        # Extract USUBJID
        usubjid = str(row.get("USUBJID", "") or "").strip()
        if usubjid and usubjid.lower() not in ("usubjid", "subjectid"):
            identifiers["USUBJID"] = usubjid

        # Extract date/time - try multiple candidate columns
        lbdtc = ""

        # First try LBDTC directly
        if "LBDTC" in row.index:
            val = self._extract_value(row, "LBDTC")
            if val:
                lbdtc = str(val).strip()

        # If not found, try columns ending with "DAT"
        if not lbdtc:
            for col in row.index:
                if str(col).upper().endswith("DAT"):
                    val = self._extract_value(row, col)
                    if val:
                        candidate = str(val).strip()
                        if candidate:
                            lbdtc = candidate
                            break

        if lbdtc:
            identifiers["LBDTC"] = lbdtc

        return identifiers

    def _extract_value(self, row: pd.Series, column: str) -> Any:
        """Extract a value from a row, with special handling for Series values.

        Extends base extraction to pick the first non-empty value from Series.

        Args:
            row: Input row
            column: Column name

        Returns:
            Extracted value
        """
        value = row.get(column, pd.NA)

        # Handle Series (can happen with duplicate columns)
        if isinstance(value, pd.Series):
            # Pick first non-NA, non-empty value
            for v in value:
                if pd.isna(v):
                    continue
                if str(v).strip():
                    return v
            return None

        # Handle Index
        if isinstance(value, pd.Index):
            if len(value) > 0:
                return value[0]
            return None

        # Handle NA values
        if pd.isna(value):
            return None

        return value

    def _normalize_test_code(self, test_code: str) -> str | None:
        """Normalize test code with LB-specific handling.

        Extends base normalization to handle special cases like GLUCU → GLUC.

        Args:
            test_code: Raw test code from source

        Returns:
            Normalized test code, or None if invalid
        """
        # Special case: GLUCU → GLUC
        if test_code.upper() == "GLUCU":
            test_code = "GLUC"

        # Use parent normalization
        if self.test_code_normalizer:
            normalized = self.test_code_normalizer(self.domain, test_code)
            if normalized:
                return normalized

        # Fall back to uppercased version
        return test_code.upper()

    def _create_test_record(
        self,
        row: pd.Series,
        test_def,
        row_identifiers: dict[str, Any],
        study_id: str,
    ) -> dict[str, Any] | None:
        """Create an LB test record from a row.

        Extends base creation to handle:
        - LBORNRLO (normal range lower limit)
        - LBORNRHI (normal range upper limit)
        - Value validation (skip headers and empty values)

        Args:
            row: Input row
            test_def: Test definition
            row_identifiers: Extracted row identifiers
            study_id: Study ID

        Returns:
            Dictionary representing SDTM LB record, or None if invalid
        """
        # Get result value
        orres_col = test_def.get_column("orres")
        if not orres_col:
            return None

        value = self._extract_value(row, orres_col)

        # Skip if no value
        if value is None:
            return None

        value_str = str(value).strip()

        # Skip empty values or header rows
        if not value_str or value_str.upper().startswith("ORRES"):
            return None

        # Normalize test code
        test_code = self._normalize_test_code(test_def.test_code)
        if not test_code:
            return None

        # Get test label (fallback to TEST_* column if CT doesn't have it)
        test_label = self._get_test_label(test_code)
        label_col = test_def.get_column("label")
        if label_col and test_label == test_code:
            label_val = self._extract_value(row, label_col)
            if label_val:
                test_label = str(label_val)

        # Build base record
        record = {
            "STUDYID": study_id,
            "DOMAIN": "LB",
            **row_identifiers,
        }

        # Add test-specific values
        record.update(
            {
                "LBTESTCD": test_code[:8],  # Max 8 chars for SDTM
                "LBTEST": test_label,
                "LBORRES": value_str,
            }
        )

        # Add unit if present
        unit_col = test_def.get_column("unit")
        if unit_col:
            unit_value = self._extract_value(row, unit_col)
            record["LBORRESU"] = str(unit_value) if unit_value else ""
        else:
            record["LBORRESU"] = ""

        # Add normal range lower limit if present
        nrlo_col = test_def.get_column("nrlo")
        if nrlo_col:
            nrlo_value = self._extract_value(row, nrlo_col)
            if nrlo_value:
                record["LBORNRLO"] = str(nrlo_value)

        # Add normal range upper limit if present
        nrhi_col = test_def.get_column("nrhi")
        if nrhi_col:
            nrhi_value = self._extract_value(row, nrhi_col)
            if nrhi_value:
                record["LBORNRHI"] = str(nrhi_value)

        return record
