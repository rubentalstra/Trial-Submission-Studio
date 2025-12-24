from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

    import pandas as pd

    from cdisc_transpiler.domain.entities.mapping import MappingConfig


class DefineGenerationError(RuntimeError):
    pass


@dataclass(frozen=True, slots=True)
class StandardDefinition:
    oid: str
    name: str
    type: str
    version: str
    status: str = "Final"
    publishing_set: str | None = None
    comment_oid: str | None = None


@dataclass(frozen=True, slots=True)
class OriginDefinition:
    type: str
    source: str | None = None
    description: str | None = None
    document_ref: str | None = None
    page_refs: str | None = None


@dataclass(frozen=True, slots=True)
class MethodDefinition:
    oid: str
    name: str
    type: str
    description: str
    document_refs: tuple[str, ...] = ()


@dataclass(frozen=True, slots=True)
class CommentDefinition:
    oid: str
    text: str


@dataclass(frozen=True, slots=True)
class WhereClauseDefinition:
    oid: str
    dataset_name: str
    variable_name: str
    variable_oid: str
    comparator: str
    check_values: tuple[str, ...]
    comment_oid: str | None = None


@dataclass(frozen=True, slots=True)
class ValueListItemDefinition:
    item_oid: str
    where_clause_oid: str | None = None
    order_number: int | None = None
    mandatory: str | None = None
    method_oid: str | None = None


@dataclass(frozen=True, slots=True)
class ValueListDefinition:
    oid: str
    items: tuple[ValueListItemDefinition, ...]


@dataclass(frozen=True, slots=True)
class StudyDataset:
    domain_code: str
    dataframe: pd.DataFrame
    config: MappingConfig
    label: str | None = None
    structure: str = "One record per subject per domain-specific entity"
    archive_location: Path | None = None
