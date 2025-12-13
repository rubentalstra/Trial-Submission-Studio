"""Domain processing service.

This service handles the core domain processing logic, including:
- Loading and transforming source data
- Applying mappings
- Generating domain DataFrames
- Handling supplemental qualifiers
- Managing domain variants and merging
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

import pandas as pd

if TYPE_CHECKING:
    from ..mapping_module import MappingConfig, ColumnMapping
    from ..metadata_module import StudyMetadata
    from ..domains_module import SDTMDomain

from ..io import load_input_dataset, build_column_hints
from ..mapping_module import build_config, create_mapper
from ..xpt_module import build_domain_dataframe
from ..submission import build_suppqual


@dataclass
class DomainProcessingResult:
    """Result of processing a domain."""

    domain_code: str
    dataframe: pd.DataFrame
    config: MappingConfig
    record_count: int
    supplementals: list[DomainProcessingResult] | None = None
    source_files: list[Path] | None = None


class DomainProcessingService:
    """Service for processing SDTM domains from source data."""

    def __init__(
        self,
        study_id: str,
        metadata: StudyMetadata | None = None,
        reference_starts: dict[str, str] | None = None,
        min_confidence: float = 0.5,
    ):
        """Initialize the domain processing service.

        Args:
            study_id: Study identifier
            metadata: Optional study metadata (Items.csv, CodeLists.csv)
            reference_starts: Optional USUBJID -> RFSTDTC mapping for study days
            min_confidence: Minimum confidence for fuzzy matching
        """
        self.study_id = study_id
        self.metadata = metadata
        self.reference_starts = reference_starts or {}
        self.min_confidence = min_confidence

    def process_domain(
        self,
        domain_code: str,
        source_file: Path,
        *,
        transform_long: bool = False,
        generate_suppqual: bool = True,
        common_column_counts: dict[str, int] | None = None,
        total_input_files: int | None = None,
    ) -> DomainProcessingResult:
        """Process a single domain from a source file.

        Args:
            domain_code: SDTM domain code (e.g., 'DM', 'AE', 'LB')
            source_file: Path to source data file
            transform_long: Whether to reshape wide data to long format
            generate_suppqual: Whether to generate SUPPQUAL dataset
            common_column_counts: Column frequency across all input files
            total_input_files: Total number of input files in study

        Returns:
            DomainProcessingResult with dataframe and metadata
        """
        # Load source data
        source_df = load_input_dataset(source_file)

        # Transform to long format if needed (VS, LB)
        if transform_long:
            source_df = self._transform_to_long(source_df, domain_code)

        # Build column hints for intelligent mapping
        column_hints = build_column_hints(source_df)

        # Create mapper and suggest mappings
        mapper = create_mapper(
            domain_code,
            metadata=self.metadata,
            min_confidence=self.min_confidence,
            column_hints=column_hints,
        )
        suggestions = mapper.suggest(source_df)

        # Build configuration
        config = build_config(domain_code, suggestions.mappings)
        config.study_id = self.study_id

        # Build domain dataframe
        domain_df = build_domain_dataframe(
            source_df,
            config,
            lenient=True,
            metadata=self.metadata,
            reference_starts=self.reference_starts,
        )

        # Generate supplemental qualifiers
        supplementals = []
        if generate_suppqual and domain_code.upper() != "LB":
            supp_result = self._generate_suppqual(
                domain_code,
                source_df,
                domain_df,
                config,
                common_column_counts,
                total_input_files,
            )
            if supp_result:
                supplementals.append(supp_result)

        return DomainProcessingResult(
            domain_code=domain_code,
            dataframe=domain_df,
            config=config,
            record_count=len(domain_df),
            supplementals=supplementals if supplementals else None,
            source_files=[source_file],
        )

    def merge_domain_variants(
        self,
        domain_code: str,
        variants: list[DomainProcessingResult],
    ) -> DomainProcessingResult:
        """Merge multiple domain variants into a single domain.

        Args:
            domain_code: Domain code
            variants: List of processing results for variants

        Returns:
            Merged DomainProcessingResult
        """
        if not variants:
            raise ValueError("No variants to merge")

        if len(variants) == 1:
            return variants[0]

        # Merge dataframes
        all_dfs = [v.dataframe for v in variants]
        merged_df = pd.concat(all_dfs, ignore_index=True)

        # Re-assign sequence numbers per subject
        seq_col = f"{domain_code}SEQ"
        if seq_col in merged_df.columns and "USUBJID" in merged_df.columns:
            merged_df[seq_col] = merged_df.groupby("USUBJID").cumcount() + 1

        # Merge supplementals
        all_supplementals = []
        for variant in variants:
            if variant.supplementals:
                all_supplementals.extend(variant.supplementals)

        # Merge supplemental dataframes if needed
        merged_supps = []
        if all_supplementals:
            supp_by_domain: dict[str, list[pd.DataFrame]] = {}
            for supp in all_supplementals:
                code = supp.domain_code
                if code not in supp_by_domain:
                    supp_by_domain[code] = []
                supp_by_domain[code].append(supp.dataframe)

            for supp_code, supp_dfs in supp_by_domain.items():
                if len(supp_dfs) > 1:
                    merged_supp_df = pd.concat(supp_dfs, ignore_index=True)
                else:
                    merged_supp_df = supp_dfs[0]

                # Get config from first supplemental
                supp_config = next(
                    s.config for s in all_supplementals if s.domain_code == supp_code
                )

                merged_supps.append(
                    DomainProcessingResult(
                        domain_code=supp_code,
                        dataframe=merged_supp_df,
                        config=supp_config,
                        record_count=len(merged_supp_df),
                    )
                )

        # Collect all source files
        all_sources = []
        for variant in variants:
            if variant.source_files:
                all_sources.extend(variant.source_files)

        return DomainProcessingResult(
            domain_code=domain_code,
            dataframe=merged_df,
            config=variants[0].config,  # Use first variant's config
            record_count=len(merged_df),
            supplementals=merged_supps if merged_supps else None,
            source_files=all_sources,
        )

    def _transform_to_long(
        self, source_df: pd.DataFrame, domain_code: str
    ) -> pd.DataFrame:
        """Transform wide-format data to long format (for VS, LB).

        Args:
            source_df: Source dataframe in wide format
            domain_code: Domain code

        Returns:
            Transformed dataframe in long format
        """
        # Import here to avoid circular dependency
        from ..cli import _reshape_vs_to_long, _reshape_lb_to_long

        if domain_code.upper() == "VS":
            return _reshape_vs_to_long(source_df, self.study_id)
        elif domain_code.upper() == "LB":
            return _reshape_lb_to_long(source_df, self.study_id)
        else:
            return source_df

    def _generate_suppqual(
        self,
        domain_code: str,
        source_df: pd.DataFrame,
        domain_df: pd.DataFrame,
        config: MappingConfig,
        common_column_counts: dict[str, int] | None,
        total_files: int | None,
    ) -> DomainProcessingResult | None:
        """Generate SUPPQUAL dataset for a domain.

        Args:
            domain_code: Domain code
            source_df: Source dataframe
            domain_df: Processed domain dataframe
            config: Mapping configuration
            common_column_counts: Column frequency across files
            total_files: Total number of files

        Returns:
            DomainProcessingResult for SUPPQUAL or None
        """
        # Get used source columns
        used_columns = set()
        if config and config.mappings:
            for mapping in config.mappings:
                used_columns.add(self._unquote_safe(mapping.source_column))
                if getattr(mapping, "use_code_column", None):
                    used_columns.add(self._unquote_safe(mapping.use_code_column))

        # Build SUPPQUAL
        supp_df, _ = build_suppqual(
            domain_code,
            source_df,
            domain_df,
            used_columns,
            study_id=self.study_id,
            common_column_counts=common_column_counts,
            total_files=total_files,
        )

        if supp_df is None or supp_df.empty:
            return None

        # Build config for SUPPQUAL
        supp_domain_code = f"SUPP{domain_code.upper()}"
        from ..mapping_module import ColumnMapping

        supp_mappings = [
            ColumnMapping(
                source_column=col,
                target_variable=col,
                transformation=None,
                confidence_score=1.0,
            )
            for col in supp_df.columns
        ]
        supp_config = build_config(supp_domain_code, supp_mappings)
        supp_config.study_id = self.study_id

        return DomainProcessingResult(
            domain_code=supp_domain_code,
            dataframe=supp_df,
            config=supp_config,
            record_count=len(supp_df),
        )

    @staticmethod
    def _unquote_safe(name: str | None) -> str:
        """Remove SAS name quoting."""
        if not name:
            return ""
        name = str(name)
        if len(name) >= 3 and name.startswith('"') and name.endswith("n"):
            inner = name[1:-1]
            if inner.endswith('"'):
                inner = inner[:-1]
            return inner.replace('""', '"')
        return name
