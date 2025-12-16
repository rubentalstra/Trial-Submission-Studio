"""Infrastructure adapter for terminology helpers."""

from __future__ import annotations

from ...application.ports import TerminologyPort


def _get_variable_codelist_code(domain_code: str, variable_name: str) -> str | None:
    """Resolve a variable's CT codelist code using SDTM domain metadata."""
    from ..sdtm_spec.registry import get_domain

    try:
        domain = get_domain(domain_code)
    except (KeyError, ValueError):
        return None

    var_upper = variable_name.upper()
    for var in domain.variables:
        if var.name.upper() == var_upper and var.codelist_code:
            return var.codelist_code
    return None


def _normalize_to_submission_value(ct, source_value: str) -> str | None:
    if not source_value:
        return None
    source_upper = source_value.upper().strip()
    if not source_upper:
        return None

    if source_upper in ct.submission_values:
        return source_upper
    if ct.synonyms and source_upper in ct.synonyms:
        return ct.synonyms[source_upper]
    return None


class TerminologyServiceAdapter(TerminologyPort):
    def normalize_testcd(self, domain_code: str, source_code: str) -> str | None:
        from ..repositories.ct_repository import CTRepository

        codelist_code = _get_variable_codelist_code(
            domain_code, f"{domain_code.upper()}TESTCD"
        )
        if not codelist_code:
            return None

        ct_repo = CTRepository()
        ct = ct_repo.get_by_code(codelist_code)
        if ct is None:
            return None

        return _normalize_to_submission_value(ct, source_code)

    def get_testcd_label(self, domain_code: str, testcd: str) -> str:
        from ..repositories.ct_repository import CTRepository

        codelist_code = _get_variable_codelist_code(
            domain_code, f"{domain_code.upper()}TESTCD"
        )
        if not codelist_code:
            return testcd

        ct_repo = CTRepository()
        ct = ct_repo.get_by_code(codelist_code)
        if ct is None:
            return testcd

        return ct.preferred_terms.get(testcd.upper(), testcd)
