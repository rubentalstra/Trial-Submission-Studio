"""Data models for Define-XML 2.1 elements.

This module contains all dataclass definitions used in Define-XML generation.
These are pure data containers with no business logic.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import pandas as pd

from ...mapping_module import MappingConfig


class DefineGenerationError(RuntimeError):
    """Raised when Define-XML export fails."""


@dataclass(frozen=True)
class StandardDefinition:
    """Represents a def:Standard element in Define-XML 2.1."""

    oid: str
    name: str
    type: str  # IG, CT
    version: str
    status: str = "Final"
    publishing_set: str | None = None  # For CT: SDTM, ADaM, SEND, etc.
    comment_oid: str | None = None


@dataclass(frozen=True)
class OriginDefinition:
    """Represents def:Origin metadata for a variable."""

    type: str  # Collected, Derived, Assigned, Protocol, Predecessor
    source: str | None = None  # Sponsor, Investigator, Vendor, Subject
    description: str | None = None
    document_ref: str | None = None
    page_refs: str | None = None


@dataclass(frozen=True)
class MethodDefinition:
    """Represents a MethodDef element for derivation algorithms."""

    oid: str
    name: str
    type: str  # Computation, Imputation, etc.
    description: str
    document_refs: tuple[str, ...] = ()


@dataclass(frozen=True)
class CommentDefinition:
    """Represents a def:CommentDef element."""

    oid: str
    text: str


@dataclass(frozen=True)
class WhereClauseDefinition:
    """Represents a def:WhereClauseDef for value-level metadata."""

    oid: str
    dataset_name: str
    variable_name: str
    variable_oid: str
    comparator: str  # EQ, NE, LT, LE, GT, GE, IN, NOTIN
    check_values: tuple[str, ...]
    comment_oid: str | None = None


@dataclass(frozen=True)
class ValueListItemDefinition:
    """Represents a single item in a value list (def:ItemRef within def:ValueListDef)."""

    item_oid: str
    where_clause_oid: str | None = None
    order_number: int | None = None
    mandatory: str | None = None  # Yes/No
    method_oid: str | None = None


@dataclass(frozen=True)
class ValueListDefinition:
    """Represents a def:ValueListDef for value-level metadata."""

    oid: str
    items: tuple[ValueListItemDefinition, ...]


@dataclass(frozen=True)
class StudyDataset:
    """Represents a single dataset with its metadata and data.

    Supports split datasets per SDTMIG v3.4 Section 4.1.7:
    - is_split: True if this dataset is a split of a parent domain
    - split_suffix: The suffix added to parent domain (e.g., "HM" for LBHM)
    """

    domain_code: str
    dataframe: pd.DataFrame
    config: MappingConfig
    label: str | None = None
    structure: str = "One record per subject per domain-specific entity"
    is_split: bool = False
    split_suffix: str | None = None
    archive_location: Path | None = None
