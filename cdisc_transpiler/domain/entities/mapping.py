"""Configuration models for column mapping.

This module contains the core data models used for defining and managing
column mappings between source data and SDTM target variables.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable, Protocol

from pydantic import BaseModel, Field

from .sdtm_domain import SDTMDomain


class DomainResolver(Protocol):
    def __call__(self, domain_code: str) -> SDTMDomain: ...


# =============================================================================
# Configuration Models
# =============================================================================


class ColumnMapping(BaseModel):
    """Mapping from a source column to an SDTM target variable."""

    source_column: str
    target_variable: str
    transformation: str | None = None
    confidence_score: float = Field(ge=0.0, le=1.0)
    # New fields for metadata-driven mappings
    codelist_name: str | None = None  # Name of codelist to apply
    use_code_column: str | None = None  # Column containing coded values

    def to_assignment(self) -> str:
        """Return SAS assignment snippet for the mapping."""
        expr = self.transformation or self.source_column
        return f"{self.target_variable} = {expr};"


class MappingConfig(BaseModel):
    """Configuration for mapping source data to an SDTM domain."""

    domain: str
    study_id: str | None = None
    mappings: list[ColumnMapping]

    # Optional study-level defaults that domain processors may use to populate
    # required values when the source does not contain them.
    default_country: str | None = None

    @property
    def target_variables(self) -> set[str]:
        return {m.target_variable for m in self.mappings}

    def enforce_domain(self, domain_resolver: DomainResolver | None = None) -> None:
        if domain_resolver is None:
            return
        domain_resolver(self.domain)  # raises if invalid

    def missing_required(
        self, domain_resolver: DomainResolver | None = None
    ) -> set[str]:
        if domain_resolver is None:
            return set()

        domain = domain_resolver(self.domain)
        required = {
            var.name
            for var in domain.variables
            if (var.core or "").strip().lower() == "req"
        }
        auto_populated = {"STUDYID", "DOMAIN"}
        return (required - auto_populated) - self.target_variables

    def validate_required(self, domain_resolver: DomainResolver | None = None) -> None:
        missing = self.missing_required(domain_resolver)
        if missing:
            raise ValueError(
                f"Missing required variables for domain {self.domain}: {sorted(missing)}"
            )


# =============================================================================
# Suggestion Models
# =============================================================================


@dataclass
class Suggestion:
    """A single mapping suggestion."""

    column: str
    candidate: str
    confidence: float


@dataclass
class MappingSuggestions:
    """Collection of mapping suggestions."""

    mappings: list[ColumnMapping]
    unmapped_columns: list[str]


# =============================================================================
# Utility Functions
# =============================================================================


def merge_mappings(
    base: MappingConfig, extra: Iterable[ColumnMapping]
) -> MappingConfig:
    """Merge additional mappings into an existing config."""
    existing = {m.target_variable: m for m in base.mappings}
    for mapping in extra:
        existing.setdefault(mapping.target_variable, mapping)
    base.mappings = list(existing.values())
    return base


def build_config(domain_code: str, mappings: Iterable[ColumnMapping]) -> MappingConfig:
    """Build a MappingConfig from a list of column mappings."""
    config = MappingConfig(domain=domain_code, mappings=list(mappings))
    config.enforce_domain()
    return config
