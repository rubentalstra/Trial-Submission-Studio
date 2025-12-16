"""VS (Vital Signs) domain transformer.

This module provides a transformer for converting wide-format vital signs data
to SDTM VS (Vital Signs) long format using the generic WideToLongTransformer base class.

SDTM Reference:
    SDTMIG v3.4 Section 6.3.7 defines the VS domain structure:
    - Required variables: VSTESTCD, VSTEST, VSORRES, VSORRESU
    - Key qualifiers: VSSTAT (NOT DONE), VSREASND (reason not done)
    - Additional: VSPOS (position during measurement)
"""

from __future__ import annotations

from typing import Any

import pandas as pd

from .wide_to_long import TestColumnPattern, WideToLongTransformer


class VSTransformer(WideToLongTransformer):
    """Transformer for VS (Vital Signs) domain wide-to-long conversion.

    This transformer extends WideToLongTransformer to handle VS-specific patterns
    and output variables. It discovers test columns using patterns for:
    - ORRES_* : Original result values
    - ORRESU_* : Original units
    - POS_* : Position during measurement
    - TEST_* : Test labels

    Special handling:
    - VSSTAT: Set to "NOT DONE" when VSPERFCD = "N"
    - VSREASND: Reason not done (when VSSTAT = "NOT DONE")
    - VSPOS: Position during measurement

    Example:
        >>> # Terminology normalization is provided via the application's TerminologyPort.
        >>> transformer = VSTransformer(
        ...     test_code_normalizer=normalize_testcd,
        ...     test_label_getter=get_testcd_label
        ... )
        >>> context = TransformationContext(domain="VS", study_id="STUDY001")
        >>> result = transformer.transform(wide_df, context)
    """

    def __init__(
        self,
        test_code_normalizer=None,
        test_label_getter=None,
    ):
        """Initialize VS transformer with VS-specific patterns.

        Args:
            test_code_normalizer: Optional function to normalize test codes using CT
            test_label_getter: Optional function to get test labels from CT
        """
        # Define VS-specific column patterns
        patterns = [
            TestColumnPattern(
                pattern=r"^ORRES_([A-Za-z0-9]+)$",
                column_type="orres",
                description="Original result columns (ORRES_*)",
            ),
            TestColumnPattern(
                pattern=r"^ORRESU_([A-Za-z0-9]+)$",
                column_type="unit",
                description="Original unit columns (ORRESU_*)",
            ),
            TestColumnPattern(
                pattern=r"^POS_([A-Za-z0-9]+)$",
                column_type="position",
                description="Position columns (POS_*)",
            ),
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
            "Event name": "VISIT",
            "Event Name": "VISIT",
            "EventName": "VISIT",
            "Event sequence number": "VISITNUM",
            "Event Sequence Number": "VISITNUM",
            "EventSeq": "VISITNUM",
            "Event date": "VSDTC",
            "Event Date": "VSDTC",
            "EventDate": "VSDTC",
        }

        # Define output variable mapping
        output_mapping = {
            "TESTCD": "VSTESTCD",
            "TEST": "VSTEST",
            "ORRES": "VSORRES",
            "ORRESU": "VSORRESU",
        }

        super().__init__(
            domain="VS",
            patterns=patterns,
            column_renames=column_renames,
            output_mapping=output_mapping,
            test_code_normalizer=test_code_normalizer,
            test_label_getter=test_label_getter,
        )

    def _normalize_columns(self, df: pd.DataFrame) -> pd.DataFrame:
        """Normalize columns and add VS-specific computed columns.

        Extends base normalization to:
        - Convert VISITNUM to numeric
        - Generate VISIT from VISITNUM if missing

        Args:
            df: Input DataFrame

        Returns:
            DataFrame with normalized columns
        """
        df = super()._normalize_columns(df)

        # Normalize visit identifiers
        if "VISITNUM" in df.columns:
            visitnum = pd.to_numeric(df["VISITNUM"], errors="coerce")
            df = df.assign(VISITNUM=visitnum)

        if "VISIT" not in df.columns and "VISITNUM" in df.columns:
            df = df.assign(
                VISIT=df["VISITNUM"].apply(
                    lambda n: f"Visit {int(n)}" if pd.notna(n) else ""
                )
            )

        return df

    def _extract_row_identifiers(self, row: pd.Series) -> dict[str, Any]:
        """Extract VS-specific row identifiers.

        Extends base extraction to handle:
        - Visit number and visit name generation
        - VSDTC (date/time)
        - VSPERFCD (performance status)
        - VSREASND (reason not done)

        Args:
            row: Input row

        Returns:
            Dictionary of identifier values
        """
        identifiers = {}

        # Extract USUBJID
        usubjid = str(row.get("USUBJID", "") or "").strip()
        if usubjid and usubjid.lower() != "usubjid":
            identifiers["USUBJID"] = usubjid

        # Extract visit information
        visitnum_raw = row.get("VISITNUM", pd.NA)
        if not pd.isna(visitnum_raw) and visitnum_raw not in (None, ""):
            try:
                visitnum_float = float(visitnum_raw)
                if not pd.isna(visitnum_float):
                    identifiers["VISITNUM"] = visitnum_float

                    # Generate VISIT from VISITNUM if needed
                    visit = str(row.get("VISIT", "") or "").strip()
                    if not visit:
                        if visitnum_float.is_integer():
                            visit = f"Visit {int(visitnum_float)}"
                    if visit:
                        identifiers["VISIT"] = visit
            except (ValueError, TypeError):
                pass

        # Use VISIT if VISITNUM not available
        if "VISIT" not in identifiers:
            visit = str(row.get("VISIT", "") or "").strip()
            if visit:
                identifiers["VISIT"] = visit

        # Extract date/time
        vsdtc = str(row.get("VSDTC", "") or "").strip()
        if vsdtc:
            identifiers["VSDTC"] = vsdtc

        # Extract performance status (for VSSTAT)
        status_cd = str(row.get("VSPERFCD", "") or "").strip().upper()
        if status_cd:
            identifiers["_VSPERFCD"] = status_cd  # Internal use

        # Extract reason not done
        reason = str(row.get("VSREASND", "") or "").strip()
        if reason:
            identifiers["_VSREASND"] = reason  # Internal use

        return identifiers

    def _create_test_record(
        self,
        row: pd.Series,
        test_def,
        row_identifiers: dict[str, Any],
        study_id: str,
    ) -> dict[str, Any] | None:
        """Create a VS test record from a row.

        Extends base creation to handle:
        - VSSTAT (NOT DONE status)
        - VSREASND (reason not done)
        - VSPOS (position)
        - Special handling when test not performed

        Args:
            row: Input row
            test_def: Test definition
            row_identifiers: Extracted row identifiers
            study_id: Study ID

        Returns:
            Dictionary representing SDTM VS record, or None if invalid
        """
        # Get result value
        orres_col = test_def.get_column("orres")
        if not orres_col:
            return None

        value = self._extract_value(row, orres_col)

        # Get performance status
        status_cd = row_identifiers.get("_VSPERFCD", "")
        reason = row_identifiers.get("_VSREASND", "")

        # Handle "NOT DONE" case
        if status_cd == "N":
            # Test not performed - create record with VSSTAT
            stat_val = "NOT DONE"
            # Continue even if value is missing
        else:
            stat_val = ""
            # Skip if no value and test was supposed to be performed
            if value is None or (isinstance(value, str) and not value.strip()):
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

        # Build base record (without internal fields)
        clean_identifiers = {
            k: v for k, v in row_identifiers.items() if not k.startswith("_")
        }

        record = {
            "STUDYID": study_id,
            "DOMAIN": "VS",
            **clean_identifiers,
        }

        # Add test-specific values
        record.update(
            {
                "VSTESTCD": test_code[:8],  # Max 8 chars for SDTM
                "VSTEST": test_label,
                "VSORRES": "" if stat_val else (value if value is not None else ""),
                "VSSTAT": stat_val,
            }
        )

        # Add unit if present (empty when NOT DONE)
        unit_col = test_def.get_column("unit")
        if unit_col:
            unit_value = self._extract_value(row, unit_col)
            record["VSORRESU"] = "" if stat_val else (unit_value if unit_value else "")
        else:
            record["VSORRESU"] = ""

        # Add reason not done if applicable
        record["VSREASND"] = reason if stat_val else ""

        # Add position if present
        pos_col = test_def.get_column("position")
        if pos_col:
            pos_value = self._extract_value(row, pos_col)
            if pos_value:
                record["VSPOS"] = str(pos_value)

        return record
