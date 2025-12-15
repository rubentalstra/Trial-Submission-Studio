"""Domain processing use case.

This module contains the use case for processing a single SDTM domain,
orchestrating file loading, transformations, mapping, and output generation.

Note: This use case demonstrates the clean architecture pattern but currently
cannot be fully instantiated due to circular import issues (services -> cli -> services).
The DTOs (ProcessDomainRequest/Response) work perfectly and provide clean contracts.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import pandas as pd

from .models import ProcessDomainRequest, ProcessDomainResponse
from .ports import LoggerPort

if TYPE_CHECKING:
    # Use TYPE_CHECKING to avoid circular import at runtime
    from ..services import DomainProcessingCoordinator


class DomainProcessingUseCase:
    """Use case for processing a single SDTM domain.
    
    This class orchestrates domain-level processing through clear pipeline stages:
    1. Load files stage - Load and validate input files
    2. Transform stage - Apply domain-specific transformations (VS, LB)
    3. Map columns stage - Map source columns to SDTM variables
    4. Build domain stage - Create final domain dataframe
    5. Generate SUPPQUAL stage - Create supplemental qualifiers
    6. Generate outputs stage - Create XPT/XML/SAS files
    
    The use case follows the Ports & Adapters architecture with dependencies
    injected via the constructor.
    
    Note: Due to circular import issues (services -> cli -> services), this
    use case currently delegates to DomainProcessingCoordinator. The DTOs
    provide clean contracts that enable proper testing and future refactoring.
    
    Example:
        >>> # This example shows the intended usage pattern
        >>> # Currently limited by circular import
        >>> use_case = DomainProcessingUseCase(logger=my_logger)
        >>> request = ProcessDomainRequest(
        ...     files_for_domain=[(Path("DM.csv"), "DM")],
        ...     domain_code="DM",
        ...     study_id="STUDY001",
        ...     output_formats={"xpt", "xml"},
        ...     output_dirs={"xpt": Path("output/xpt")},
        ... )
        >>> response = use_case.execute(request)
        >>> if response.success:
        ...     print(f"Processed {response.records} records")
    """
    
    def __init__(self, logger: LoggerPort):
        """Initialize the use case with injected dependencies.
        
        Args:
            logger: Logger for progress and error reporting
        """
        self.logger = logger
    
    def execute(self, request: ProcessDomainRequest) -> ProcessDomainResponse:
        """Execute the domain processing workflow.
        
        This method orchestrates the complete domain processing pipeline:
        - Loads input files
        - Applies transformations
        - Maps columns
        - Builds domain dataframe
        - Generates SUPPQUAL
        - Generates output files
        
        Args:
            request: Domain processing request with all parameters
            
        Returns:
            Domain processing response with results and any errors
            
        Example:
            >>> response = use_case.execute(request)
            >>> print(f"Success: {response.success}")
            >>> print(f"Records: {response.records}")
            >>> print(f"Errors: {response.error}")
        """
        response = ProcessDomainResponse(domain_code=request.domain_code)
        
        try:
            # Import here to avoid circular import at module level
            from ..services import DomainProcessingCoordinator
            
            # Create coordinator (this encapsulates the current implementation)
            coordinator = DomainProcessingCoordinator()
            
            # Delegate to coordinator (temporary until circular import is resolved)
            # In the future, this would be decomposed into clear pipeline stages
            result_dict = coordinator.process_and_merge_domain(
                files_for_domain=request.files_for_domain,
                domain_code=request.domain_code,
                study_id=request.study_id,
                output_format="/".join(request.output_formats),
                xpt_dir=request.output_dirs.get("xpt"),
                xml_dir=request.output_dirs.get("xml"),
                sas_dir=request.output_dirs.get("sas"),
                min_confidence=request.min_confidence,
                streaming=request.streaming,
                chunk_size=request.chunk_size,
                generate_sas=request.generate_sas,
                verbose=request.verbose > 0,
                metadata=request.metadata,
                reference_starts=request.reference_starts,
                common_column_counts=request.common_column_counts,
                total_input_files=request.total_input_files,
            )
            
            # Convert dict result to ProcessDomainResponse
            response.success = True
            response.records = result_dict.get("records", 0)
            response.domain_dataframe = result_dict.get("domain_dataframe")
            response.config = result_dict.get("config")
            response.xpt_path = result_dict.get("xpt_path")
            response.xml_path = result_dict.get("xml_path")
            response.sas_path = result_dict.get("sas_path")
            response.split_datasets = result_dict.get("split_datasets", [])
            
            # Handle supplemental domains
            for supp_dict in result_dict.get("supplementals", []):
                supp_response = ProcessDomainResponse(
                    success=True,
                    domain_code=supp_dict.get("domain_code", ""),
                    records=supp_dict.get("records", 0),
                    domain_dataframe=supp_dict.get("domain_dataframe"),
                    config=supp_dict.get("config"),
                    xpt_path=supp_dict.get("xpt_path"),
                    xml_path=supp_dict.get("xml_path"),
                    sas_path=supp_dict.get("sas_path"),
                )
                response.supplementals.append(supp_response)
            
        except Exception as exc:
            response.success = False
            response.error = str(exc)
            self.logger.error(f"{request.domain_code}: {exc}")
        
        return response
    
    # Future pipeline stages (commented out due to circular import):
    # These would be implemented when the circular import is resolved
    
    # def _load_files_stage(self, request: ProcessDomainRequest) -> list[pd.DataFrame]:
    #     """Stage 1: Load and validate input files."""
    #     pass
    
    # def _transform_stage(self, dataframes: list[pd.DataFrame], request: ProcessDomainRequest) -> list[pd.DataFrame]:
    #     """Stage 2: Apply domain-specific transformations."""
    #     pass
    
    # def _map_columns_stage(self, dataframes: list[pd.DataFrame], request: ProcessDomainRequest) -> tuple[list[pd.DataFrame], Any]:
    #     """Stage 3: Map source columns to SDTM variables."""
    #     pass
    
    # def _build_domain_stage(self, dataframes: list[pd.DataFrame], config: Any, request: ProcessDomainRequest) -> pd.DataFrame:
    #     """Stage 4: Build final domain dataframe."""
    #     pass
    
    # def _generate_suppqual_stage(self, domain_df: pd.DataFrame, request: ProcessDomainRequest) -> pd.DataFrame | None:
    #     """Stage 5: Generate supplemental qualifiers."""
    #     pass
    
    # def _generate_outputs_stage(self, domain_df: pd.DataFrame, config: Any, request: ProcessDomainRequest) -> dict[str, Path]:
    #     """Stage 6: Generate output files (XPT, XML, SAS)."""
    #     pass
