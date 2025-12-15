"""Generic wide-to-long transformer for SDTM Findings domains.

This module provides a base transformer class that converts wide-format source data
(one column per test) to SDTM long format (one row per test per subject per timepoint).
It extracts common logic shared by VS (Vital Signs) and LB (Laboratory) transformers.

The transformer uses configurable patterns to:
- Discover test-specific columns (results, units, normal ranges, etc.)
- Normalize test codes using CDISC Controlled Terminology
- Generate SDTM-compliant output with proper variable names

SDTM Reference:
    SDTMIG v3.4 Section 6.3 defines Findings domains which share common structure:
    - --TESTCD: Short code for the test
    - --TEST: Long test name/description
    - --ORRES: Result in original units
    - --ORRESU: Original units
    Additional domain-specific variables (e.g., --ORNRLO, --ORNRHI for LB)
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from typing import Any, Callable

import pandas as pd

from ..base import TransformationContext, TransformationResult, TransformerPort


@dataclass
class TestColumnPattern:
    """Pattern definition for discovering test-specific columns.
    
    This dataclass defines a regex pattern to match column names and extract
    the test code from them. Multiple patterns can be defined for different
    source data formats.
    
    Attributes:
        pattern: Regex pattern with a capture group for the test code
        column_type: Type of column (e.g., 'orres', 'unit', 'nrlo', 'nrhi', 'label')
        description: Human-readable description of what this pattern matches
        
    Example:
        >>> # Match "ORRES_HEIGHT", "ORRES_WEIGHT" -> extract "HEIGHT", "WEIGHT"
        >>> pattern = TestColumnPattern(
        ...     pattern=r"^ORRES_([A-Za-z0-9]+)$",
        ...     column_type="orres",
        ...     description="Original result columns (ORRES_*)"
        ... )
    """
    
    pattern: str
    column_type: str
    description: str = ""
    
    def match(self, column_name: str) -> str | None:
        """Try to match this pattern against a column name.
        
        Args:
            column_name: Column name to match
            
        Returns:
            Extracted test code if pattern matches, None otherwise
            
        Example:
            >>> pattern = TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")
            >>> pattern.match("ORRES_HEIGHT")
            'HEIGHT'
            >>> pattern.match("OTHER_COLUMN")
            None
        """
        match = re.match(self.pattern, column_name, re.IGNORECASE)
        if match:
            return match.group(1).upper()
        return None


@dataclass
class TestDefinition:
    """Definition of columns for a specific test.
    
    Tracks all the columns associated with a test code (result, unit, normal ranges, etc.)
    
    Attributes:
        test_code: The test code (e.g., 'HEIGHT', 'GLUC')
        columns: Dictionary mapping column type to actual column name
        
    Example:
        >>> test_def = TestDefinition(
        ...     test_code="HEIGHT",
        ...     columns={
        ...         "orres": "ORRES_HEIGHT",
        ...         "unit": "ORRESU_HEIGHT"
        ...     }
        ... )
    """
    
    test_code: str
    columns: dict[str, str] = field(default_factory=dict)
    
    def get_column(self, column_type: str) -> str | None:
        """Get the actual column name for a column type.
        
        Args:
            column_type: Type of column (e.g., 'orres', 'unit')
            
        Returns:
            Column name if exists, None otherwise
        """
        return self.columns.get(column_type)
    
    def has_result(self) -> bool:
        """Check if this test has a result column."""
        return "orres" in self.columns


class WideToLongTransformer:
    """Base transformer for converting wide-format Findings data to SDTM long format.
    
    This class provides the common logic for transforming wide-format source data
    to SDTM Findings domains (VS, LB, etc.). Domain-specific transformers should
    subclass this and provide domain-specific patterns and column mappings.
    
    The transformation process:
    1. Discover test-specific columns using regex patterns
    2. Normalize column names for common identifiers
    3. For each row, unpivot test columns into separate rows
    4. Normalize test codes using CDISC CT
    5. Map to SDTM variable names
    
    Attributes:
        domain: SDTM domain code (e.g., 'VS', 'LB')
        patterns: List of TestColumnPattern for column discovery
        column_renames: Dictionary mapping source column names to standard names
        output_mapping: Dictionary mapping generic names to domain-specific names
        test_code_normalizer: Optional function to normalize test codes
        test_label_getter: Optional function to get test labels from CT
        
    Example:
        >>> class VSTransformer(WideToLongTransformer):
        ...     def __init__(self):
        ...         super().__init__(
        ...             domain="VS",
        ...             patterns=[
        ...                 TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres"),
        ...                 TestColumnPattern(r"^ORRESU_([A-Z]+)$", "unit"),
        ...             ],
        ...             output_mapping={
        ...                 "TESTCD": "VSTESTCD",
        ...                 "TEST": "VSTEST",
        ...                 "ORRES": "VSORRES",
        ...             }
        ...         )
    """
    
    def __init__(
        self,
        domain: str,
        patterns: list[TestColumnPattern],
        column_renames: dict[str, str] | None = None,
        output_mapping: dict[str, str] | None = None,
        test_code_normalizer: Callable[[str, str], str | None] | None = None,
        test_label_getter: Callable[[str, str], str] | None = None,
    ):
        """Initialize the wide-to-long transformer.
        
        Args:
            domain: SDTM domain code (e.g., 'VS', 'LB')
            patterns: List of patterns for discovering test columns
            column_renames: Dictionary of column name mappings (default: {})
            output_mapping: Dictionary mapping generic to domain-specific names (default: {})
            test_code_normalizer: Function to normalize test codes using CT
            test_label_getter: Function to get test labels from CT
        """
        self.domain = domain.upper()
        self.patterns = patterns
        self.column_renames = column_renames or {}
        self.output_mapping = output_mapping or {}
        self.test_code_normalizer = test_code_normalizer
        self.test_label_getter = test_label_getter
    
    def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
        """Check if this transformer applies to the given data/domain.
        
        Args:
            df: Input DataFrame
            domain: Domain code
            
        Returns:
            True if this transformer should be applied
        """
        if domain.upper() != self.domain:
            return False
        
        # Check if we can find any test columns
        test_defs = self._discover_tests(df)
        return len(test_defs) > 0
    
    def transform(
        self, df: pd.DataFrame, context: TransformationContext
    ) -> TransformationResult:
        """Transform wide-format data to SDTM long format.
        
        Args:
            df: Input DataFrame with wide-format test data
            context: Transformation context
            
        Returns:
            TransformationResult with long-format SDTM data
        """
        if not self.can_transform(df, context.domain):
            return TransformationResult(
                data=df,
                applied=False,
                message=f"Transformer does not apply to domain {context.domain}",
            )
        
        result_df = df.copy()
        
        # Step 1: Normalize column names
        result_df = self._normalize_columns(result_df)
        
        # Step 2: Discover test definitions
        test_defs = self._discover_tests(result_df)
        
        if not test_defs:
            return TransformationResult(
                data=pd.DataFrame(),
                applied=True,
                message="No test columns found",
                warnings=["No columns matching test patterns"],
            )
        
        # Step 3: Unpivot rows
        records = self._unpivot_rows(result_df, test_defs, context)
        
        if not records:
            return TransformationResult(
                data=pd.DataFrame(),
                applied=True,
                message="No valid test data found after unpivoting",
                warnings=["All rows filtered out during transformation"],
            )
        
        # Step 4: Create output DataFrame
        long_df = pd.DataFrame(records)
        
        return TransformationResult(
            data=long_df,
            applied=True,
            message=f"Converted {len(df)} wide rows to {len(long_df)} long rows",
            metadata={
                "input_rows": len(df),
                "output_rows": len(long_df),
                "tests_found": len(test_defs),
                "test_codes": [td.test_code for td in test_defs],
            },
        )
    
    def _normalize_columns(self, df: pd.DataFrame) -> pd.DataFrame:
        """Normalize common column names using rename mappings.
        
        Args:
            df: Input DataFrame
            
        Returns:
            DataFrame with normalized column names
        """
        if not self.column_renames:
            return df
        
        return df.rename(columns=self.column_renames)
    
    def _discover_tests(self, df: pd.DataFrame) -> list[TestDefinition]:
        """Discover test definitions from DataFrame columns.
        
        Scans all columns against the defined patterns to build a list of
        test definitions with their associated columns.
        
        Args:
            df: DataFrame to scan
            
        Returns:
            List of TestDefinition objects
        """
        # Build dictionary of test_code -> {column_type: column_name}
        test_dict: dict[str, dict[str, str]] = {}
        
        for column in df.columns:
            for pattern in self.patterns:
                test_code = pattern.match(str(column))
                if test_code:
                    # Skip coded columns (ending with CD)
                    if test_code.endswith("CD"):
                        continue
                    
                    test_dict.setdefault(test_code, {})[pattern.column_type] = column
                    break  # Only match first pattern
        
        # Convert to TestDefinition objects
        test_defs = [
            TestDefinition(test_code=code, columns=cols)
            for code, cols in sorted(test_dict.items())
        ]
        
        # Filter to only tests with result columns
        return [td for td in test_defs if td.has_result()]
    
    def _unpivot_rows(
        self,
        df: pd.DataFrame,
        test_defs: list[TestDefinition],
        context: TransformationContext,
    ) -> list[dict[str, Any]]:
        """Unpivot wide rows into long format.
        
        For each row in the input, create one output row per test.
        
        Args:
            df: Input DataFrame
            test_defs: List of test definitions
            context: Transformation context
            
        Returns:
            List of dictionaries representing long-format rows
        """
        records: list[dict[str, Any]] = []
        study_id = context.study_id or ""
        
        for _, row in df.iterrows():
            # Extract common row identifiers
            row_identifiers = self._extract_row_identifiers(row)
            
            # Skip rows without subject ID
            if not row_identifiers.get("USUBJID"):
                continue
            
            # Process each test
            for test_def in test_defs:
                test_record = self._create_test_record(
                    row, test_def, row_identifiers, study_id
                )
                
                if test_record:
                    records.append(test_record)
        
        return records
    
    def _extract_row_identifiers(self, row: pd.Series) -> dict[str, Any]:
        """Extract common identifiers from a row.
        
        Subclasses can override to extract domain-specific identifiers.
        
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
        
        # Extract visit information if present
        if "VISITNUM" in row.index:
            visitnum_raw = row.get("VISITNUM", pd.NA)
            if not pd.isna(visitnum_raw) and visitnum_raw not in (None, ""):
                try:
                    identifiers["VISITNUM"] = float(visitnum_raw)
                except (ValueError, TypeError):
                    pass
        
        if "VISIT" in row.index:
            visit = str(row.get("VISIT", "") or "").strip()
            if visit:
                identifiers["VISIT"] = visit
        
        # Extract date/time information (domain-specific column name)
        dtc_col = f"{self.domain}DTC"
        if dtc_col in row.index:
            dtc = str(row.get(dtc_col, "") or "").strip()
            if dtc:
                identifiers[dtc_col] = dtc
        
        return identifiers
    
    def _create_test_record(
        self,
        row: pd.Series,
        test_def: TestDefinition,
        row_identifiers: dict[str, Any],
        study_id: str,
    ) -> dict[str, Any] | None:
        """Create a single test record from a row.
        
        Subclasses can override to add domain-specific logic.
        
        Args:
            row: Input row
            test_def: Test definition
            row_identifiers: Extracted row identifiers
            study_id: Study ID
            
        Returns:
            Dictionary representing SDTM test record, or None if invalid
        """
        # Get result value
        orres_col = test_def.get_column("orres")
        if not orres_col:
            return None
        
        value = self._extract_value(row, orres_col)
        if value is None or (isinstance(value, str) and not value.strip()):
            return None
        
        # Normalize test code
        test_code = self._normalize_test_code(test_def.test_code)
        if not test_code:
            return None
        
        # Get test label
        test_label = self._get_test_label(test_code)
        
        # Build base record
        record = {
            "STUDYID": study_id,
            "DOMAIN": self.domain,
            **row_identifiers,
        }
        
        # Add test-specific values with output mapping
        record.update(
            {
                self._map_output_name("TESTCD"): test_code[:8],  # Max 8 chars for SDTM
                self._map_output_name("TEST"): test_label,
                self._map_output_name("ORRES"): value,
            }
        )
        
        # Add unit if present
        unit_col = test_def.get_column("unit")
        if unit_col:
            unit_value = self._extract_value(row, unit_col)
            if unit_value:
                record[self._map_output_name("ORRESU")] = unit_value
        
        return record
    
    def _extract_value(self, row: pd.Series, column: str) -> Any:
        """Extract a value from a row, handling Series and Index types.
        
        Args:
            row: Input row
            column: Column name
            
        Returns:
            Extracted value
        """
        value = row.get(column, pd.NA)
        
        # Handle Series (can happen with duplicate columns)
        if isinstance(value, pd.Series):
            for v in value:
                if pd.notna(v):
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
        """Normalize test code using CT if normalizer is provided.
        
        Args:
            test_code: Raw test code from source
            
        Returns:
            Normalized test code, or None if invalid
        """
        if self.test_code_normalizer:
            normalized = self.test_code_normalizer(self.domain, test_code)
            if normalized:
                return normalized
        
        # Fall back to uppercased version
        return test_code.upper()
    
    def _get_test_label(self, test_code: str) -> str:
        """Get test label from CT if getter is provided.
        
        Args:
            test_code: Normalized test code
            
        Returns:
            Test label (defaults to test code if not found)
        """
        if self.test_label_getter:
            label = self.test_label_getter(self.domain, test_code)
            if label and label != test_code:
                return label
        
        # Fall back to test code as label
        return test_code
    
    def _map_output_name(self, generic_name: str) -> str:
        """Map generic variable name to domain-specific name.
        
        Args:
            generic_name: Generic name (e.g., 'TESTCD', 'TEST')
            
        Returns:
            Domain-specific name (e.g., 'VSTESTCD', 'VSTEST')
        """
        return self.output_mapping.get(generic_name, generic_name)
