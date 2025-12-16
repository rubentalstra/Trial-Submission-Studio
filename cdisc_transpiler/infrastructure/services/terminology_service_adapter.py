"""Infrastructure adapter for terminology helpers."""

from __future__ import annotations

from ...application.ports import TerminologyPort


class TerminologyServiceAdapter(TerminologyPort):
    def normalize_testcd(self, domain_code: str, source_code: str) -> str | None:
        from ...terminology_module import normalize_testcd

        return normalize_testcd(domain_code, source_code)

    def get_testcd_label(self, domain_code: str, testcd: str) -> str:
        from ...terminology_module import get_testcd_label

        return get_testcd_label(domain_code, testcd)
