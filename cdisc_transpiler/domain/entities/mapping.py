from dataclasses import dataclass
from typing import TYPE_CHECKING, Protocol

from pydantic import BaseModel, Field

if TYPE_CHECKING:
    from collections.abc import Iterable

    from .sdtm_domain import SDTMDomain


class DomainResolver(Protocol):
    pass

    def __call__(self, domain_code: str) -> SDTMDomain: ...


class ColumnMapping(BaseModel):
    source_column: str
    target_variable: str
    transformation: str | None = None
    confidence_score: float = Field(ge=0.0, le=1.0)
    codelist_name: str | None = None
    use_code_column: str | None = None

    def to_assignment(self) -> str:
        expr = self.transformation or self.source_column
        return f"{self.target_variable} = {expr};"


class MappingConfig(BaseModel):
    domain: str
    study_id: str | None = None
    mappings: list[ColumnMapping]
    default_country: str | None = None

    @property
    def target_variables(self) -> set[str]:
        return {m.target_variable for m in self.mappings}

    def enforce_domain(self, domain_resolver: DomainResolver | None = None) -> None:
        if domain_resolver is None:
            return
        domain_resolver(self.domain)

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
        return required - auto_populated - self.target_variables

    def validate_required(self, domain_resolver: DomainResolver | None = None) -> None:
        missing = self.missing_required(domain_resolver)
        if missing:
            raise ValueError(
                f"Missing required variables for domain {self.domain}: {sorted(missing)}"
            )


@dataclass(slots=True)
class Suggestion:
    column: str
    candidate: str
    confidence: float


@dataclass(slots=True)
class MappingSuggestions:
    mappings: list[ColumnMapping]
    unmapped_columns: list[str]


def merge_mappings(
    base: MappingConfig, extra: Iterable[ColumnMapping]
) -> MappingConfig:
    existing = {m.target_variable: m for m in base.mappings}
    for mapping in extra:
        existing.setdefault(mapping.target_variable, mapping)
    base.mappings = list(existing.values())
    return base


def build_config(domain_code: str, mappings: Iterable[ColumnMapping]) -> MappingConfig:
    config = MappingConfig(domain=domain_code, mappings=list(mappings))
    config.enforce_domain()
    return config
