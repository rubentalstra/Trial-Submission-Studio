"""SAS program generation.

This module provides the core functionality for generating SAS programs
from SDTM mapping configurations.
"""

from __future__ import annotations

from datetime import UTC, datetime
from typing import TYPE_CHECKING

from jinja2 import Environment, StrictUndefined

if TYPE_CHECKING:
    from ..mapping_module import MappingConfig

from ..domains_module import get_domain
from .constants import DEFAULT_STUDY_ID, SAS_PROGRAM_TEMPLATE
from .utils import (
    get_assignment_for_mapping,
    get_default_assignments,
    get_keep_clause,
)


class SASProgramGenerator:
    """Generator for SAS transformation programs.

    This class encapsulates the logic for generating SAS DATA step programs
    that transform raw data into SDTM-compliant datasets.
    """

    def __init__(self) -> None:
        """Initialize the SAS program generator."""
        self._env = Environment(
            trim_blocks=True,
            lstrip_blocks=True,
            undefined=StrictUndefined,
        )
        self._template = self._env.from_string(SAS_PROGRAM_TEMPLATE)

    def generate(
        self,
        domain_code: str,
        config: MappingConfig,
        input_dataset: str,
        output_dataset: str,
    ) -> str:
        """Generate a SAS program for domain transformation.

        Args:
            domain_code: SDTM domain code (e.g., "DM", "AE")
            config: Mapping configuration with column mappings
            input_dataset: Name of input SAS dataset
            output_dataset: Name of output SAS dataset

        Returns:
            Complete SAS program as string

        Example:
            >>> generator = SASProgramGenerator()
            >>> program = generator.generate("DM", config, "rawdata", "dm")
        """
        domain = get_domain(domain_code)

        # Generate assignments for mapped columns
        assignments = [
            get_assignment_for_mapping(mapping, domain) for mapping in config.mappings
        ]

        # Generate default assignments for required unmapped variables
        default_assignments = get_default_assignments(domain, config)

        # Get the KEEP clause with all domain variables
        keep_clause = get_keep_clause(domain)

        # Use configured study ID or default
        study_id = config.study_id or DEFAULT_STUDY_ID

        # Render the SAS program from template
        program = self._template.render(
            domain=domain,
            timestamp=datetime.now(UTC).isoformat(timespec="seconds"),
            assignments=assignments,
            default_assignments=default_assignments,
            keep_clause=keep_clause,
            input_dataset=input_dataset,
            output_dataset=output_dataset,
            study_id=study_id,
        )

        return program


def generate_sas_program(
    domain_code: str,
    config: MappingConfig,
    input_dataset: str,
    output_dataset: str,
) -> str:
    """Generate a SAS program for domain transformation.

    This is a convenience function that creates a generator instance
    and generates the program. For repeated calls, consider reusing
    a SASProgramGenerator instance.

    Args:
        domain_code: SDTM domain code (e.g., "DM", "AE")
        config: Mapping configuration with column mappings
        input_dataset: Name of input SAS dataset
        output_dataset: Name of output SAS dataset

    Returns:
        Complete SAS program as string

    Example:
        >>> program = generate_sas_program("DM", config, "rawdata", "dm")
    """
    generator = SASProgramGenerator()
    return generator.generate(domain_code, config, input_dataset, output_dataset)
