"""Data models for SDTM domains and variables.

SDTM Reference:
    SDTMIG v3.4 Section 4 defines the structure of SDTM datasets.
    Each domain contains variables with specific roles and attributes.
"""

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class SDTMVariable:
    """SDTM variable definition.

    Represents a single variable (column) within an SDTM domain, including
    its name, type, and metadata as defined in SDTMIG v3.4.

    SDTM Reference:
        Variables follow naming conventions where domain-specific variables
        use a 2-character domain prefix (e.g., AESTDTC = AE + STDTC).

        Core Status:
        - Req (Required): Must be included and populated
        - Exp (Expected): Must be included when applicable
        - Perm (Permissible): May be included when needed

    Attributes:
        name: Variable name (8 characters max, uppercase)
        label: Descriptive label (40 characters max)
        type: Data type - "Char" (character) or "Num" (numeric)
        length: Maximum length for character variables
        core: Core status (Req/Exp/Perm)
        codelist_code: CDISC Controlled Terminology code (C-code)
        variable_order: Position within domain
        role: Variable role (Identifier, Topic, Timing, Qualifier)
    """

    name: str
    label: str
    type: str  # Char or Num
    length: int
    core: str | None = None  # Core: Req, Exp, Perm
    codelist_code: str | None = None  # CDISC CT codelist code (e.g., C66742)
    variable_order: int | None = None  # Variable Order from CSV
    role: str | None = None  # Role: Identifier, Topic, Timing, Qualifier
    value_list: str | None = None  # Value List reference
    described_value_domain: str | None = None  # Described Value Domain(s)
    codelist_submission_values: str | None = None  # Codelist Submission Values
    usage_restrictions: str | None = None  # Usage Restrictions
    definition: str | None = None  # SDTM Definition
    notes: str | None = None  # CDISC Notes
    variables_qualified: str | None = None  # Variables Qualified by this variable
    source_dataset: str | None = None  # Source dataset name
    source_version: str | None = None  # Source standard version
    # General Observation Class linkage (e.g., --SEQ, --DTC)
    implements: str | None = None

    def pandas_dtype(self) -> str:
        """Return the pandas dtype for the variable.

        Returns:
            'float64' for numeric variables, 'string' for character variables
        """
        if self.type == "Num":
            return "float64"
        return "string"


@dataclass(frozen=True, slots=True)
class SDTMDomain:
    """SDTM domain definition.

    Represents an SDTM dataset (domain) containing a collection of
    observations with topic-specific commonality.

    SDTM Reference:
        SDTMIG v3.4 Section 4 defines domain structures. Each domain
        belongs to a General Observation Class:

        - Special-Purpose: DM, CO, SE, SV, SM
        - Interventions: EX, CM, EC, SU, PR, AG
        - Events: AE, DS, MH, DV, CE, HO
        - Findings: LB, VS, EG, PE, QS, SC, FA
        - Trial Design: TA, TE, TV, TI, TS
        - Relationship: RELREC, RELSUB, RELSPEC, SUPPQUAL

    Attributes:
        code: 2-character domain abbreviation (e.g., 'DM', 'AE')
        description: Full domain description
        class_name: General Observation Class name
        structure: Dataset structure description
        label: Dataset label (40 characters max)
        variables: Tuple of SDTMVariable definitions
        dataset_name: 8-character dataset filename
    """

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
        """Return mapping of variable name to generalized placeholder.

        For General Observation Class variables, returns the placeholder
        format (e.g., AESTDTC implements --STDTC).
        """
        return {var.name: var.implements for var in self.variables if var.implements}

    def resolved_dataset_name(self) -> str:
        """Return the 8-character dataset name for file generation.

        SDTM dataset names must be 8 characters or fewer for SAS
        Transport (XPT) compatibility.
        """
        name = (self.dataset_name or self.code).upper()
        return name[:8]
