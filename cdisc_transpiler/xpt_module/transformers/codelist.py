"""Codelist and controlled terminology transformation utilities for SDTM domains.

This module provides specialized transformation logic for applying and validating
controlled terminology (codelist) values according to CDISC standards.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Sequence, Any

import pandas as pd

from ...terminology_module import get_controlled_terminology
from ...domains_module import SDTMVariable
from ...pandas_utils import ensure_numeric_series, ensure_series

if TYPE_CHECKING:
    from ...metadata_module import StudyMetadata


class CodelistTransformer:
    """Transforms and validates controlled terminology values for SDTM compliance.

    This class provides methods for:
    - Applying codelist transformations to map codes to text values
    - Normalizing values to canonical CDISC Controlled Terminology forms
    - Validating paired terminology (e.g., TEST/TESTCD)
    - Populating MedDRA defaults for adverse events
    """

    def __init__(self, metadata: "StudyMetadata | None" = None):
        """Initialize the codelist transformer.

        Args:
            metadata: Optional study metadata containing codelist definitions
        """
        self.metadata = metadata

    def apply_codelist_transformation(
        self,
        source_data: Any,
        codelist_name: str,
        code_column: str | None = None,
        source_frame: pd.DataFrame | None = None,
        unquote_func=None,
    ) -> pd.Series:
        """Transform coded values to their text equivalents using codelist.

        Args:
            source_data: The source data series
            codelist_name: Name of the codelist to apply
            code_column: Optional column containing code values (for text columns)
            source_frame: Optional source DataFrame for code column lookup
            unquote_func: Optional function to unquote column names

        Returns:
            Transformed series with text values
        """
        if not self.metadata:
            return ensure_series(source_data)

        codelist = self.metadata.get_codelist(codelist_name)
        if not codelist:
            return ensure_series(source_data)

        # If we have a code column, use it for lookup
        if code_column and source_frame is not None:
            code_col = code_column
            if code_col not in source_frame.columns and unquote_func:
                alt = unquote_func(code_col)
                if alt in source_frame.columns:
                    code_col = alt

            if code_col in source_frame.columns:
                code_values = ensure_series(source_frame[code_col], index=source_data.index if hasattr(source_data, "index") else None)

                def transform(code_val):
                    if pd.isna(code_val):
                        return None
                    text = codelist.get_text(code_val)
                    return text if text is not None else str(code_val)

                return ensure_series(code_values.apply(transform), index=code_values.index)

        # Otherwise, try to transform the source data directly
        def transform_value(val):
            if pd.isna(val):
                return val
            # First check if it's a code that needs transformation
            text = codelist.get_text(val)
            if text is not None:
                return text
            # If not found in codelist, return as-is
            return val

        series = ensure_series(source_data)
        return ensure_series(series.apply(transform_value), index=series.index)

    @staticmethod
    def apply_codelist_validations(
        frame: pd.DataFrame,
        domain_variables: Sequence[SDTMVariable],
    ) -> None:
        """Apply codelist normalizations to the DataFrame.

        This normalizes raw values to their CDISC Controlled Terminology
        canonical forms using synonym mappings.

        Args:
            frame: DataFrame to modify in-place
            domain_variables: List of SDTMVariable objects defining domain structure
        """
        for var in domain_variables:
            if var.codelist_code and var.name in frame.columns:
                # Keep provided dictionary names (e.g., TSVCDREF) as-is to avoid
                # normalizer collapsing specific variants like ISO 3166-1 alpha-3.
                if var.name == "TSVCDREF":
                    continue
                ct_lookup = get_controlled_terminology(
                    codelist_code=var.codelist_code
                ) or get_controlled_terminology(variable=var.name)
                if ct_lookup is None:
                    continue
                normalizer = ct_lookup.normalize

                # Work in string dtype to avoid dtype-mismatch warnings
                series = frame[var.name].astype("string")
                # Identify values that are present (non-missing) after trimming
                trimmed = series.str.strip()
                mask = trimmed.notna() & (trimmed != "")

                def _normalize_ct_value(value: str) -> str:
                    normalized = normalizer(value)
                    return normalized or value

                normalized = series.copy()
                normalized.loc[mask] = trimmed.loc[mask].apply(_normalize_ct_value)
                # Write back as string to keep dtype consistent across assignments
                frame[var.name] = normalized.astype("string")

    @staticmethod
    def validate_controlled_terms(
        frame: pd.DataFrame,
        domain_variables: list,
    ) -> None:
        """Validate controlled terminology values and replace invalid ones.

        Args:
            frame: DataFrame to modify in-place
            domain_variables: List of SDTMVariable objects defining domain structure
        """
        for variable in domain_variables:
            if not variable.codelist_code:
                continue
            ct = get_controlled_terminology(codelist_code=variable.codelist_code)
            if not ct or variable.name not in frame.columns:
                continue
            invalid = ct.invalid_values(frame[variable.name])
            if invalid:
                canonical_default = sorted(ct.submission_values)[0]
                series = ensure_series(frame[variable.name]).astype(str)
                frame[variable.name] = series.where(
                    ~series.isin(list(invalid)), canonical_default
                )

    @staticmethod
    def validate_paired_terms(frame: pd.DataFrame) -> None:
        """Ensure paired TEST/TESTCD-style variables are both populated when present.

        Args:
            frame: DataFrame to validate

        Raises:
            ValueError: If paired terms are inconsistently populated
        """
        pairs = [
            ("AETEST", "AETESTCD"),
            ("LBTEST", "LBTESTCD"),
            ("VSTEST", "VSTESTCD"),
            ("QSTEST", "QSTESTCD"),
            ("MHDECOD", "MHTERM"),
        ]

        for left, right in pairs:
            if left not in frame.columns or right not in frame.columns:
                continue
            left_series = frame[left].astype(str).str.strip()
            right_series = frame[right].astype(str).str.strip()
            missing_right = (left_series != "") & (right_series == "")
            missing_left = (right_series != "") & (left_series == "")
            if missing_right.any() or missing_left.any():
                raise ValueError(
                    f"Paired terminology mismatch for {left}/{right}: both must be populated together"
                )

    @staticmethod
    def populate_meddra_defaults(frame: pd.DataFrame) -> None:
        """Populate MedDRA defaults for adverse event data.

        This method fills in default MedDRA hierarchy terms and codes when
        they are missing from the source data.

        Args:
            frame: DataFrame to modify in-place (typically AE domain)
        """
        # Derive MedDRA text from AETERM when missing
        aetext = frame.get("AETERM", pd.Series(["" for _ in frame.index]))
        if "AEDECOD" in frame.columns:
            decod = frame["AEDECOD"].astype("string")
            frame["AEDECOD"] = decod.where(decod.str.strip() != "", aetext)
        else:
            frame["AEDECOD"] = aetext

        # Fill SOC/group terms with a generic MedDRA bucket when absent
        for soc_var, term in {
            "AEBODSYS": "GENERAL DISORDERS",
            "AESOC": "GENERAL DISORDERS",
            "AEHLGT": "GENERAL DISORDERS",
            "AEHLT": "GENERAL DISORDERS",
            "AELLT": "GENERAL DISORDERS",
        }.items():
            if soc_var in frame.columns:
                frame[soc_var] = (
                    frame[soc_var]
                    .astype("string")
                    .fillna("")
                    .replace("", term)
                    .fillna(term)
                )
            else:
                frame[soc_var] = term

        # Fill code columns with numeric defaults when missing/empty
        for code_var in (
            "AEPTCD",
            "AEHLGTCD",
            "AEHLTCD",
            "AELLTCD",
            "AESOCCD",
            "AEBDSYCD",
        ):
            if code_var in frame.columns:
                code_series = ensure_numeric_series(frame[code_var], index=frame.index)
                frame[code_var] = code_series.fillna(999999).astype("Int64")
            else:
                frame[code_var] = pd.Series(
                    [999999 for _ in frame.index], dtype="Int64"
                )
