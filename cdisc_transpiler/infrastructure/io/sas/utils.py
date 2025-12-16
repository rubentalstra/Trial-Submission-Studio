"""Utility functions for SAS code generation.

This module contains helper functions for generating SAS assignments,
defaults, and other code fragments.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from .normalizers import render_assignment

if TYPE_CHECKING:
    from cdisc_transpiler.domain.entities.sdtm_domain import SDTMDomain, SDTMVariable
    from cdisc_transpiler.domain.entities.mapping import ColumnMapping, MappingConfig


def get_default_assignments(domain: SDTMDomain, config: MappingConfig) -> list[str]:
    """Generate default value assignments for unmapped required variables.

    Args:
        domain: The SDTM domain definition
        config: Mapping configuration with existing mappings

    Returns:
        List of SAS assignment statements for required variables
    """
    defaults: list[str] = []
    mapped_variables = config.target_variables

    for variable in domain.variables:
        # Skip if variable is already mapped and required
        if (
            variable.name in mapped_variables
            and (variable.core or "").strip().lower() == "req"
        ):
            continue

        # Skip if variable is not required
        if (variable.core or "").strip().lower() != "req":
            continue

        defaults.append(get_default_value_assignment(variable))

    return defaults


def get_default_value_assignment(variable: SDTMVariable) -> str:
    """Generate a SAS assignment for a variable's default value.

    Args:
        variable: The SDTM variable to generate default for

    Returns:
        SAS assignment statement (e.g., "USUBJID = ''; ")
    """
    if variable.type.lower() == "num":
        return f"{variable.name} = .;"
    return f"{variable.name} = '';"


def get_assignment_for_mapping(mapping: ColumnMapping, domain: SDTMDomain) -> str:
    """Generate a SAS assignment statement from a column mapping.

    Args:
        mapping: Column mapping configuration
        domain: Target SDTM domain

    Returns:
        SAS assignment statement
    """
    variable_lookup = {var.name: var for var in domain.variables}
    variable = variable_lookup.get(mapping.target_variable)
    return render_assignment(mapping, variable)


def get_keep_clause(domain: SDTMDomain) -> str:
    """Generate the KEEP statement clause for a domain.

    Args:
        domain: SDTM domain definition

    Returns:
        Space-separated list of variable names
    """
    return " ".join(domain.variable_names())
