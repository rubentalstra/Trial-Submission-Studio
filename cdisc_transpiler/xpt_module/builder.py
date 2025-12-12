"""DataFrame construction and orchestration for SDTM domains.

This module provides the core builder class that orchestrates the construction
of SDTM-compliant DataFrames from source data and mapping configurations.

Phase 4 Step 7: This module now uses the modular transformers and validators
extracted in Steps 3-6, while maintaining backward compatibility.
"""

from __future__ import annotations

import re
from typing import TYPE_CHECKING

import pandas as pd

from ..mapping import ColumnMapping, MappingConfig
from ..domains import SDTMVariable, get_domain

# Import the modular components (Steps 3-6)
from .transformers import (
    DateTransformer,
    CodelistTransformer,
    NumericTransformer,
)
from .validators import XPTValidator

if TYPE_CHECKING:
    from ..metadata import StudyMetadata

_SAFE_NAME_RE = re.compile(r'^(?P<quoted>"(?:[^"]|"")*")n$', re.IGNORECASE)


class XportGenerationError(RuntimeError):
    """Raised when XPT export cannot be completed."""

# Re-export the exception for API compatibility
__all__ = ["XportGenerationError", "build_domain_dataframe", "DomainFrameBuilder"]


def build_domain_dataframe(
    frame: pd.DataFrame,
    config: MappingConfig,
    *,
    reference_starts: dict[str, str] | None = None,
    lenient: bool = False,
    metadata: "StudyMetadata | None" = None,
) -> pd.DataFrame:
    """Return a pandas DataFrame that matches the SDTM domain layout.

    Args:
        frame: The source DataFrame.
        config: The mapping configuration.
        reference_starts: Optional mapping of USUBJID -> RFSTDTC for study day calculations.
        lenient: If True, skip validation of required values (useful for Dataset-XML).
        metadata: Optional StudyMetadata for codelist transformations.

    Returns:
        A DataFrame with columns matching the SDTM domain layout.
    """
    builder = DomainFrameBuilder(
        frame,
        config,
        reference_starts=reference_starts,
        lenient=lenient,
        metadata=metadata,
    )
    return builder.build()


class DomainFrameBuilder:
    """Builds SDTM-compliant DataFrames from source data.
    
    This class orchestrates the construction of domain DataFrames by:
    1. Creating a blank DataFrame with domain variables
    2. Applying column mappings from source to target
    3. Performing transformations (dates, codelists, numeric)
    4. Validating and enforcing SDTM requirements
    5. Reordering columns to match domain specification
    
    Phase 4 Step 7: Now uses modular transformers and validators from Steps 3-6.
    Domain-specific processing still delegates to original xpt.py for complex logic.
    """

    def __init__(
        self,
        frame: pd.DataFrame,
        config: MappingConfig,
        *,
        reference_starts: dict[str, str] | None = None,
        lenient: bool = False,
        metadata: "StudyMetadata | None" = None,
    ) -> None:
        self.frame = frame.reset_index(drop=True)
        self.config = config
        self.domain = get_domain(config.domain)
        self.variable_lookup = {var.name: var for var in self.domain.variables}
        self.length = len(self.frame)
        self.reference_starts = reference_starts or {}
        self.lenient = lenient
        self.metadata = metadata
        
        # Initialize transformers
        self.codelist_transformer = CodelistTransformer(metadata)

    def build(self) -> pd.DataFrame:
        """Build the domain DataFrame using modular transformers and validators."""
        # Create a blank DataFrame with all domain variables
        result = pd.DataFrame(
            {var.name: self._default_column(var) for var in self.domain.variables}
        )

        # Apply mappings
        if self.config and self.config.mappings:
            for mapping in self.config.mappings:
                self._apply_mapping(result, mapping)
        else:
            # No mapping provided, assume frame is already structured correctly.
            for col in self.frame.columns:
                if col in result.columns:
                    result[col] = self.frame[col]

        # Fill in STUDYID and DOMAIN
        if self.config and self.config.study_id:
            result["STUDYID"] = self.config.study_id
        if "DOMAIN" in result.columns:
            result["DOMAIN"] = self.domain.code

        # Perform transformations using modular components (Steps 3-5)
        DateTransformer.normalize_dates(result, self.domain.variables)
        DateTransformer.calculate_dy(result, self.domain.variables, self.reference_starts)
        DateTransformer.normalize_durations(result, self.domain.variables)
        CodelistTransformer.apply_codelist_validations(result, self.domain.variables)
        NumericTransformer.populate_stresc_from_orres(result, self.domain.code)
        
        # Domain-specific processing - still uses original implementation
        # This is the complex 2,500+ line method that handles all domain-specific logic
        self._post_process_domain(result)
        
        # Validation and cleanup using modular components (Step 6)
        XPTValidator.drop_empty_optional_columns(result, self.domain.variables)
        XPTValidator.reorder_columns(result, self.domain.variables)
        XPTValidator.enforce_required_values(result, self.domain.variables, self.lenient)
        XPTValidator.enforce_lengths(result, self.domain.variables)

        return result

    def _apply_mapping(self, result: pd.DataFrame, mapping: ColumnMapping) -> None:
        """Apply a single column mapping to the result DataFrame."""
        if mapping.target_variable not in self.variable_lookup:
            return

        source_column = mapping.source_column
        raw_source = self._unquote_column(source_column)

        if mapping.transformation:
            # TODO: Implement transformation logic
            pass
        else:
            # Get the source data
            if source_column in self.frame.columns:
                source_data = self.frame[source_column].copy()
            elif raw_source in self.frame.columns:
                source_data = self.frame[raw_source].copy()
            else:
                return

            # Apply codelist transformation if specified (using modular transformer)
            if (
                mapping.codelist_name
                and self.metadata
                and mapping.target_variable != "TSVCDREF"
            ):
                code_column = mapping.use_code_column
                code_column = self._unquote_column(code_column) if code_column else None
                source_data = self.codelist_transformer.apply_codelist_transformation(
                    source_data,
                    mapping.codelist_name,
                    code_column,
                    self.frame,
                    self._unquote_column,
                )

            result[mapping.target_variable] = source_data

    def _default_column(self, variable: SDTMVariable) -> pd.Series:
        """Return a default column series for a given variable."""
        dtype = variable.pandas_dtype()
        return pd.Series([None] * self.length, dtype=dtype)

    @staticmethod
    def _unquote_column(name: str) -> str:
        """Remove quotes from SAS-safe column names."""
        match = _SAFE_NAME_RE.fullmatch(name)
        if not match:
            return name
        quoted = match.group("quoted")
        unescaped = quoted[1:-1].replace('""', '"')
        return unescaped

    def _post_process_domain(self, result: pd.DataFrame) -> None:
        """Perform domain-specific post-processing using the domain processor system.
        
        This method delegates to domain-specific processors that handle the unique
        requirements of each SDTM domain.
        """
        from .domain_processors import get_domain_processor
        
        # Get the appropriate processor for this domain
        processor = get_domain_processor(
            self.domain,
            self.reference_starts,
            self.metadata,
        )
        
        # Apply domain-specific processing
        processor.process(result)


# Backward compatibility alias
_DomainFrameBuilder = DomainFrameBuilder
