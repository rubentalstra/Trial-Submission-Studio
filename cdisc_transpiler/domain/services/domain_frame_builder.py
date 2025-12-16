"""Domain DataFrame builder service.

This module provides the core business logic for constructing SDTM-compliant
DataFrames from source data and mapping configurations. This is domain logic
that belongs in the domain layer, not in the output-focused xpt_module.

SDTM Reference:
    SDTMIG v3.4 Section 4.1 defines the general structure of SDTM datasets.
    Variables follow the General Observation Classes (Interventions, Events,
    Findings) and include Identifier, Topic, Timing, and Qualifier roles.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Callable

import pandas as pd

from ..entities.sdtm_domain import SDTMDomain, SDTMVariable

if TYPE_CHECKING:
    from ...mapping_module import ColumnMapping, MappingConfig
    from ...metadata_module import StudyMetadata


class DomainFrameBuildError(RuntimeError):
    """Raised when domain DataFrame construction fails."""


def build_domain_dataframe(
    frame: pd.DataFrame,
    config: "MappingConfig",
    domain: SDTMDomain,
    *,
    reference_starts: dict[str, str] | None = None,
    lenient: bool = False,
    metadata: "StudyMetadata | None" = None,
    domain_processor_factory: Callable | None = None,
    transformers: dict | None = None,
    validators: dict | None = None,
) -> pd.DataFrame:
    """Build an SDTM-compliant domain DataFrame from source data.

    This is the main entry point for domain DataFrame construction. It orchestrates
    the full pipeline of creating, mapping, transforming, and validating domain data.

    Args:
        frame: Source DataFrame with raw data
        config: Mapping configuration specifying column mappings
        domain: SDTMDomain definition for the target domain
        reference_starts: Optional USUBJID -> RFSTDTC mapping for study day calculations
        lenient: If True, skip validation of required values (useful for Dataset-XML)
        metadata: Optional StudyMetadata for codelist transformations
        domain_processor_factory: Optional factory to get domain-specific processors
        transformers: Optional dict of transformer classes (DateTransformer, etc.)
        validators: Optional dict of validator classes (XPTValidator)

    Returns:
        DataFrame with columns matching the SDTM domain layout

    Example:
        >>> from cdisc_transpiler.domains_module import get_domain
        >>> domain = get_domain("DM")
        >>> result = build_domain_dataframe(source_df, config, domain)
    """
    builder = DomainFrameBuilder(
        frame,
        config,
        domain,
        reference_starts=reference_starts,
        lenient=lenient,
        metadata=metadata,
        domain_processor_factory=domain_processor_factory,
        transformers=transformers,
        validators=validators,
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

    This is core SDTM domain logic that belongs in the domain layer.
    """

    def __init__(
        self,
        frame: pd.DataFrame,
        config: "MappingConfig",
        domain: SDTMDomain,
        *,
        reference_starts: dict[str, str] | None = None,
        lenient: bool = False,
        metadata: "StudyMetadata | None" = None,
        domain_processor_factory: Callable | None = None,
        transformers: dict | None = None,
        validators: dict | None = None,
    ) -> None:
        """Initialize the builder.

        Args:
            frame: Source DataFrame
            config: Mapping configuration
            domain: SDTMDomain definition (injected, not looked up)
            reference_starts: USUBJID -> RFSTDTC mapping
            lenient: Skip required value validation
            metadata: Study metadata for transformations
            domain_processor_factory: Factory to get domain processors
            transformers: Dict with 'date', 'codelist', 'numeric' transformer classes
            validators: Dict with 'xpt' validator class
        """
        self.frame = frame.reset_index(drop=True)
        self.config = config
        self.domain = domain
        self.variable_lookup = {var.name: var for var in domain.variables}
        self.length = len(frame)
        self.reference_starts = reference_starts or {}
        self.lenient = lenient
        self.metadata = metadata
        self._domain_processor_factory = domain_processor_factory
        self._transformers = transformers or {}
        self._validators = validators or {}

        # Initialize codelist transformer if provided
        codelist_transformer_cls = self._transformers.get("codelist")
        self.codelist_transformer = (
            codelist_transformer_cls(metadata) if codelist_transformer_cls else None
        )

    def build(self) -> pd.DataFrame:
        """Build the domain DataFrame using transformers and validators.

        Returns:
            Complete SDTM-compliant DataFrame
        """
        # Create a blank DataFrame with all domain variables
        result = pd.DataFrame(
            {var.name: self._default_column(var) for var in self.domain.variables}
        )

        # Apply mappings
        if self.config and self.config.mappings:
            for mapping in self.config.mappings:
                self._apply_mapping(result, mapping)
        else:
            # No mapping provided, assume frame is already structured correctly
            for col in self.frame.columns:
                if col in result.columns:
                    result[col] = self.frame[col]

        # Fill in STUDYID and DOMAIN
        if self.config and self.config.study_id:
            result["STUDYID"] = self.config.study_id
        if "DOMAIN" in result.columns:
            result["DOMAIN"] = self.domain.code

        # Apply transformations
        self._apply_transformations(result)

        # Apply lightweight, domain-agnostic normalizations that are required
        # for downstream processing (e.g., SUPPQUAL joins) even when no
        # domain-specific processor factory is provided.
        self._apply_common_normalizations(result)

        # Domain-specific processing
        self._post_process_domain(result)

        # Validation and cleanup
        self._validate_and_cleanup(result)

        return result

    def _apply_common_normalizations(self, result: pd.DataFrame) -> None:
        """Apply minimal normalizations needed for SDTM compliance.

        This intentionally stays small and non-invasive:
        - Normalizes DM.SEX to common SDTM CT tokens (M/F/U/UNDIFFERENTIATED)
        - Populates *SEQ variables when the entire column is missing
          (common when mappings don't provide sequence values)
        """
        # DM.SEX controlled terminology normalization
        if self.domain.code.upper() == "DM" and "SEX" in result.columns:
            normalized = (
                result["SEX"]
                .astype("string")
                .fillna("")
                .str.strip()
                .str.upper()
            )
            result["SEX"] = normalized.replace(
                {
                    "F": "F",
                    "FEMALE": "F",
                    "M": "M",
                    "MALE": "M",
                    "U": "U",
                    "UNKNOWN": "U",
                    "UNK": "U",
                    "": "",
                    "INTERSEX": "UNDIFFERENTIATED",
                    "UNDIFFERENTIATED": "UNDIFFERENTIATED",
                }
            )

        # Populate sequence columns when missing entirely
        if "USUBJID" not in result.columns:
            return

        usubjid = result["USUBJID"].astype("string").fillna("").str.strip()
        for col in result.columns:
            if not col.upper().endswith("SEQ"):
                continue

            series = result[col]

            # Only populate when sequences are effectively absent.
            # We treat "all missing" and "constant/near-constant" as absent.
            numeric = pd.to_numeric(series, errors="coerce")
            if numeric.isna().all() or numeric.nunique(dropna=True) <= 1:
                result[col] = result.groupby(usubjid).cumcount() + 1

    def _apply_mapping(self, result: pd.DataFrame, mapping: "ColumnMapping") -> None:
        """Apply a single column mapping to the result DataFrame."""
        if mapping.target_variable not in self.variable_lookup:
            return

        # Lazy import to avoid circular dependencies
        from ...mapping_module import unquote_column_name

        source_column = mapping.source_column
        raw_source = unquote_column_name(source_column)

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

            # Apply codelist transformation if specified
            if (
                mapping.codelist_name
                and self.metadata
                and mapping.target_variable != "TSVCDREF"
                and self.codelist_transformer
            ):
                code_column = mapping.use_code_column
                code_column = unquote_column_name(code_column) if code_column else None
                source_data = self.codelist_transformer.apply_codelist_transformation(
                    source_data,
                    mapping.codelist_name,
                    code_column,
                    self.frame,
                    unquote_column_name,
                )

            result[mapping.target_variable] = source_data

    def _default_column(self, variable: SDTMVariable) -> pd.Series:
        """Return a default column series for a given variable."""
        dtype = variable.pandas_dtype()
        return pd.Series([None] * self.length, dtype=dtype)

    def _apply_transformations(self, result: pd.DataFrame) -> None:
        """Apply date, codelist, and numeric transformations."""
        date_transformer = self._transformers.get("date")
        codelist_transformer = self._transformers.get("codelist")
        numeric_transformer = self._transformers.get("numeric")

        if date_transformer:
            date_transformer.normalize_dates(result, self.domain.variables)
            date_transformer.calculate_dy(
                result, self.domain.variables, self.reference_starts
            )
            date_transformer.normalize_durations(result, self.domain.variables)

        if codelist_transformer:
            codelist_transformer.apply_codelist_validations(
                result, self.domain.variables
            )

        if numeric_transformer:
            numeric_transformer.populate_stresc_from_orres(result, self.domain.code)

    def _post_process_domain(self, result: pd.DataFrame) -> None:
        """Perform domain-specific post-processing.

        Uses the domain processor system for domain-specific logic.
        """
        if not self._domain_processor_factory:
            return

        processor = self._domain_processor_factory(
            self.domain,
            self.reference_starts,
            self.metadata,
        )
        processor.config = self.config
        processor.process(result)

    def _validate_and_cleanup(self, result: pd.DataFrame) -> None:
        """Apply validation and cleanup using validators."""
        xpt_validator = self._validators.get("xpt")

        if xpt_validator:
            xpt_validator.drop_empty_optional_columns(result, self.domain.variables)
            xpt_validator.reorder_columns(result, self.domain.variables)
            xpt_validator.enforce_required_values(
                result, self.domain.variables, self.lenient
            )
            xpt_validator.enforce_lengths(result, self.domain.variables)
