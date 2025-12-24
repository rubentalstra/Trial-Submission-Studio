from dataclasses import dataclass
from typing import TYPE_CHECKING, Protocol, TypedDict

import pandas as pd

from .column_ordering import ordered_columns_for_domain
from .mapping.utils import unquote_column_name

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence

    from ..entities.mapping import ColumnMapping, MappingConfig
    from ..entities.sdtm_domain import SDTMDomain, SDTMVariable
    from ..entities.study_metadata import StudyMetadata
    from .domain_processors.base import BaseDomainProcessor
    from .transformers.codelist import CTResolver


class DomainFrameBuildError(RuntimeError):
    pass


class DateTransformerPort(Protocol):
    pass

    @staticmethod
    def normalize_dates(
        frame: pd.DataFrame, domain_variables: Sequence[SDTMVariable]
    ) -> None: ...

    @staticmethod
    def normalize_durations(
        frame: pd.DataFrame, domain_variables: Sequence[SDTMVariable]
    ) -> None: ...

    @staticmethod
    def calculate_dy(
        frame: pd.DataFrame,
        domain_variables: Sequence[SDTMVariable],
        reference_starts: dict[str, str],
    ) -> None: ...


class NumericTransformerPort(Protocol):
    pass

    @staticmethod
    def populate_stresc_from_orres(frame: pd.DataFrame, domain_code: str) -> None: ...


class CodelistTransformerPort(Protocol):
    pass

    def apply_codelist_transformation(
        self,
        source_data: object,
        codelist_name: str,
        code_column: str | None = None,
        source_frame: pd.DataFrame | None = None,
        unquote_func: Callable[[str], str] | None = None,
    ) -> pd.Series: ...

    @staticmethod
    def apply_codelist_validations(
        frame: pd.DataFrame,
        domain_variables: Sequence[SDTMVariable],
        *,
        ct_resolver: CTResolver | None = None,
    ) -> None: ...


class CodelistTransformerFactory(Protocol):
    pass

    def __call__(
        self, metadata: StudyMetadata | None = None
    ) -> CodelistTransformerPort: ...

    @staticmethod
    def apply_codelist_validations(
        frame: pd.DataFrame,
        domain_variables: Sequence[SDTMVariable],
        *,
        ct_resolver: CTResolver | None = None,
    ) -> None: ...


class DomainFrameValidatorPort(Protocol):
    pass

    def drop_empty_optional_columns(
        self, frame: pd.DataFrame, domain_variables: Sequence[SDTMVariable]
    ) -> None: ...

    def enforce_required_values(
        self,
        frame: pd.DataFrame,
        domain_variables: Sequence[SDTMVariable],
        lenient: bool,
    ) -> None: ...

    def enforce_lengths(
        self, frame: pd.DataFrame, domain_variables: Sequence[SDTMVariable]
    ) -> None: ...


class TransformerRegistry(TypedDict, total=False):
    date: DateTransformerPort
    codelist: CodelistTransformerFactory
    numeric: NumericTransformerPort


class ValidatorRegistry(TypedDict, total=False):
    xpt: DomainFrameValidatorPort


@dataclass(slots=True)
class DomainFrameBuildRequest:
    frame: pd.DataFrame
    config: MappingConfig
    domain: SDTMDomain
    reference_starts: dict[str, str] | None = None
    lenient: bool = False
    metadata: StudyMetadata | None = None
    domain_processor_factory: (
        Callable[
            [SDTMDomain, dict[str, str] | None, StudyMetadata | None],
            BaseDomainProcessor,
        ]
        | None
    ) = None
    transformers: TransformerRegistry | None = None
    validators: ValidatorRegistry | None = None


def build_domain_dataframe(request: DomainFrameBuildRequest) -> pd.DataFrame:
    builder = DomainFrameBuilder(request)
    return builder.build()


class DomainFrameBuilder:
    pass

    def __init__(self, request: DomainFrameBuildRequest) -> None:
        super().__init__()
        self.frame = request.frame.reset_index(drop=True)
        self.config = request.config
        self.domain = request.domain
        self.variable_lookup = {var.name: var for var in request.domain.variables}
        self.length = len(request.frame)
        self.reference_starts = request.reference_starts or {}
        self.lenient = request.lenient
        self.metadata = request.metadata
        self._domain_processor_factory = request.domain_processor_factory
        self._transformers: TransformerRegistry = request.transformers or {}
        self._validators: ValidatorRegistry = request.validators or {}
        codelist_transformer_cls = self._transformers.get("codelist")
        self.codelist_transformer: CodelistTransformerPort | None = (
            codelist_transformer_cls(request.metadata)
            if codelist_transformer_cls
            else None
        )

    def build(self) -> pd.DataFrame:
        result = pd.DataFrame(
            {var.name: self._default_column(var) for var in self.domain.variables}
        )
        if self.config and self.config.mappings:
            for mapping in self.config.mappings:
                self._apply_mapping(result, mapping)
        else:
            for col in self.frame.columns:
                if col in result.columns:
                    result[col] = self.frame[col]
        if self.config and self.config.study_id:
            result["STUDYID"] = self.config.study_id
        if "DOMAIN" in result.columns:
            result["DOMAIN"] = self.domain.code
        self._apply_transformations(result)
        self._apply_common_normalizations(result)
        self._post_process_domain(result)
        self._validate_and_cleanup(result)
        return self._reorder_columns_for_domain(result)

    def _reorder_columns_for_domain(self, result: pd.DataFrame) -> pd.DataFrame:
        ordered = ordered_columns_for_domain(result, domain=self.domain)
        if list(result.columns) == ordered:
            return result
        return result.loc[:, ordered]

    def _apply_common_normalizations(self, result: pd.DataFrame) -> None:
        if self.domain.code.upper() == "DM" and "SEX" in result.columns:
            normalized = (
                result["SEX"].astype("string").fillna("").str.strip().str.upper()
            )
            result.loc[:, "SEX"] = normalized.replace(
                {
                    "F": "F",
                    "FEMALE": "F",
                    "M": "M",
                    "MALE": "M",
                    "U": "U",
                    "UNKNOWN": "U",
                    "UNK": "U",
                    "": "",
                    "INTERSEX": "UNDIFFERENTIATED",
                    "UNDIFFERENTIATED": "UNDIFFERENTIATED",
                }
            )
        if "USUBJID" in result.columns:
            usubjid = result["USUBJID"].astype("string").fillna("").str.strip()
            needs_usubjid = usubjid.str.upper().isin(
                {"", "NAN", "<NA>", "NONE", "NULL"}
            )
            if needs_usubjid.any():
                subject_source = None
                for candidate in (
                    "SubjectId",
                    "Subject ID",
                    "SUBJECTID",
                    "SUBJECT ID",
                    "SUBJID",
                ):
                    if candidate in self.frame.columns:
                        subject_source = self.frame[candidate]
                        break
                if subject_source is not None:
                    subj = subject_source.astype("string").fillna("").str.strip()
                    subj = subj.where(
                        ~subj.str.upper().isin({"SUBJECTID", "SUBJECT ID", "SUBJID"}),
                        "",
                    )
                    study_id = (
                        ""
                        if not (self.config and self.config.study_id)
                        else str(self.config.study_id)
                    )
                    if study_id:
                        prefixed = (study_id + "-" + subj).where(subj != "", "")
                        already_prefixed = subj.str.upper().str.startswith(
                            (study_id + "-").upper()
                        )
                        derived = subj.where(already_prefixed, prefixed)
                    else:
                        derived = subj
                    usubjid = usubjid.where(~needs_usubjid, derived)
                    result.loc[:, "USUBJID"] = usubjid
        if "USUBJID" not in result.columns:
            return
        usubjid = result["USUBJID"].astype("string").fillna("").str.strip()
        for col in result.columns:
            if not col.upper().endswith("SEQ"):
                continue
            series = result[col]
            numeric = pd.to_numeric(series, errors="coerce")
            if numeric.isna().all() or numeric.nunique(dropna=True) <= 1:
                result.loc[:, col] = result.groupby(usubjid).cumcount() + 1

    def _apply_mapping(self, result: pd.DataFrame, mapping: ColumnMapping) -> None:
        if mapping.target_variable not in self.variable_lookup:
            return
        source_column = mapping.source_column
        raw_source = unquote_column_name(source_column)
        if mapping.transformation:
            expr = mapping.transformation
            raw_expr = unquote_column_name(expr)
            if expr in self.frame.columns:
                result[mapping.target_variable] = self.frame[expr].copy()
            elif raw_expr in self.frame.columns:
                result[mapping.target_variable] = self.frame[raw_expr].copy()
            return
        if source_column in self.frame.columns:
            source_data = self.frame[source_column].copy()
        elif raw_source in self.frame.columns:
            source_data = self.frame[raw_source].copy()
        else:
            return
        if (
            mapping.codelist_name
            and self.metadata
            and (mapping.target_variable != "TSVCDREF")
            and self.codelist_transformer
        ):
            code_column = mapping.use_code_column
            code_column = unquote_column_name(code_column) if code_column else None
            source_data = self.codelist_transformer.apply_codelist_transformation(
                source_data,
                mapping.codelist_name,
                code_column,
                self.frame,
                unquote_column_name,
            )
        result[mapping.target_variable] = source_data

    def _default_column(self, variable: SDTMVariable) -> pd.Series:
        dtype = variable.pandas_dtype()
        return pd.Series([None] * self.length, dtype=dtype)

    def _apply_transformations(self, result: pd.DataFrame) -> None:
        date_transformer = self._transformers.get("date")
        codelist_transformer = self._transformers.get("codelist")
        numeric_transformer = self._transformers.get("numeric")
        if date_transformer:
            date_transformer.normalize_dates(result, self.domain.variables)
            date_transformer.calculate_dy(
                result, self.domain.variables, self.reference_starts
            )
            date_transformer.normalize_durations(result, self.domain.variables)
        if codelist_transformer:
            codelist_transformer.apply_codelist_validations(
                result, self.domain.variables
            )
        if numeric_transformer:
            numeric_transformer.populate_stresc_from_orres(result, self.domain.code)

    def _post_process_domain(self, result: pd.DataFrame) -> None:
        if not self._domain_processor_factory:
            return
        processor = self._domain_processor_factory(
            self.domain, self.reference_starts, self.metadata
        )
        processor.config = self.config
        processor.process(result)

    def _validate_and_cleanup(self, result: pd.DataFrame) -> None:
        xpt_validator = self._validators.get("xpt")
        if xpt_validator:
            xpt_validator.drop_empty_optional_columns(result, self.domain.variables)
            xpt_validator.enforce_required_values(
                result, self.domain.variables, self.lenient
            )
            xpt_validator.enforce_lengths(result, self.domain.variables)
