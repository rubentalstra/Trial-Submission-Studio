"""Data models for SDTM domains and variables."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class SDTMVariable:
    """SDTM variable definition."""

    name: str
    label: str
    type: str  # Char or Num
    length: int
    core: str | None = None  # Core: Req, Exp, Perm
    codelist_code: str | None = None  # CDISC CT codelist code (e.g., C66742)
    variable_order: int | None = None  # CSV: Variable Order
    role: str | None = None  # CSV: Role
    value_list: str | None = None  # CSV: Value List
    described_value_domain: str | None = None  # CSV: Described Value Domain(s)
    codelist_submission_values: str | None = None  # CSV: Codelist Submission Values
    usage_restrictions: str | None = None  # CSV: Usage Restrictions (SDTM v2.1)
    definition: str | None = None  # CSV: Definition/CDISC Notes
    notes: str | None = None  # CSV: CDISC Notes/Notes
    variables_qualified: str | None = None  # CSV: Variables Qualified
    source_dataset: str | None = None  # CSV: Dataset Name
    source_version: str | None = None  # CSV: Version
    # General Observation Class linkage (e.g., --SEQ, --DTC)
    implements: str | None = None

    def pandas_dtype(self) -> str:
        """Return the pandas dtype for the variable."""
        if self.type == "Num":
            return "float64"
        return "string"


@dataclass(frozen=True)
class SDTMDomain:
    """SDTM domain definition."""

    code: str
    description: str
    class_name: str
    structure: str
    label: str | None
    variables: tuple[SDTMVariable, ...]
    dataset_name: str | None = None

    def variable_names(self) -> tuple[str, ...]:
        """Return tuple of variable names in this domain."""
        return tuple(var.name for var in self.variables)

    def implements_mapping(self) -> dict[str, str]:
        """Return mapping of variable name to generalized identifier/timing concept."""
        return {var.name: var.implements for var in self.variables if var.implements}

    def resolved_dataset_name(self) -> str:
        """Return the 8-character dataset name."""
        name = (self.dataset_name or self.code).upper()
        return name[:8]
