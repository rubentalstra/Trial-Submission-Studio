from dataclasses import dataclass, field
from typing import TYPE_CHECKING

from ...pandas_utils import is_missing_scalar

if TYPE_CHECKING:
    from pathlib import Path


def _empty_codelist_values() -> list[CodeListValue]:
    return []


def _empty_items() -> dict[str, SourceColumn]:
    return {}


def _empty_codelists() -> dict[str, CodeList]:
    return {}


@dataclass(slots=True)
class CodeListValue:
    code_value: str
    code_text: str
    data_type: str


@dataclass(slots=True)
class CodeList:
    format_name: str
    values: list[CodeListValue] = field(default_factory=_empty_codelist_values)

    def get_text(self, code: object) -> str | None:
        if code is None or is_missing_scalar(code):
            return None
        code_str = str(code).strip().upper()
        for value in self.values:
            if str(value.code_value).strip().upper() == code_str:
                return value.code_text
        return None

    def get_code(self, text: object) -> str | None:
        if text is None or is_missing_scalar(text):
            return None
        text_str = str(text).strip().upper()
        for value in self.values:
            if str(value.code_text).strip().upper() == text_str:
                return value.code_value
        return None


@dataclass(slots=True)
class SourceColumn:
    id: str
    label: str
    data_type: str
    mandatory: bool
    format_name: str | None
    content_length: int | None

    @property
    def is_code_column(self) -> bool:
        return self.id.endswith("CD") and self.format_name is not None

    @property
    def base_column_id(self) -> str:
        if self.id.endswith("CD"):
            return self.id[:-2]
        return self.id


@dataclass(slots=True)
class StudyMetadata:
    items: dict[str, SourceColumn] = field(default_factory=_empty_items)
    codelists: dict[str, CodeList] = field(default_factory=_empty_codelists)
    source_path: Path | None = None

    def get_column(self, column_id: str) -> SourceColumn | None:
        return self.items.get(column_id.upper())

    def get_codelist(self, format_name: str) -> CodeList | None:
        return self.codelists.get(format_name.upper())

    def get_codelist_for_column(self, column_id: str) -> CodeList | None:
        column = self.get_column(column_id)
        if column and column.format_name:
            return self.get_codelist(column.format_name)
        return None

    def transform_value(self, column_id: str, value: object) -> object:
        column = self.get_column(column_id)
        if not column:
            return value
        if column.is_code_column:
            codelist = (
                self.get_codelist(column.format_name) if column.format_name else None
            )
            if codelist:
                text = codelist.get_text(value)
                if text is not None:
                    return text
        return value
