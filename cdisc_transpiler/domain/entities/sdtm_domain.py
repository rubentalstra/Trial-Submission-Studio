from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class SDTMVariable:
    name: str
    label: str
    type: str
    length: int
    core: str | None = None
    codelist_code: str | None = None
    variable_order: int | None = None
    role: str | None = None
    value_list: str | None = None
    described_value_domain: str | None = None
    codelist_submission_values: str | None = None
    usage_restrictions: str | None = None
    definition: str | None = None
    notes: str | None = None
    variables_qualified: str | None = None
    source_dataset: str | None = None
    source_version: str | None = None
    implements: str | None = None

    def pandas_dtype(self) -> str:
        if self.type == "Num":
            return "float64"
        return "string"


@dataclass(frozen=True, slots=True)
class SDTMDomain:
    code: str
    description: str
    class_name: str
    structure: str
    label: str | None
    variables: tuple[SDTMVariable, ...]
    dataset_name: str | None = None

    def variable_names(self) -> tuple[str, ...]:
        return tuple(var.name for var in self.variables)

    def implements_mapping(self) -> dict[str, str]:
        return {var.name: var.implements for var in self.variables if var.implements}

    def resolved_dataset_name(self) -> str:
        name = (self.dataset_name or self.code).upper()
        return name[:8]
