from functools import lru_cache
import re
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ...entities.sdtm_domain import SDTMDomain
MIN_LABEL_TERM_LENGTH = 3


def _deduplicate_preserving_order(items: list[str]) -> list[str]:
    return list(dict.fromkeys(items))


def _normalize_text(text: str) -> str:
    return re.sub("[^A-Z0-9]", "", text.upper())


def _extract_label_terms(label: str) -> list[str]:
    skip_words = {
        "FOR",
        "THE",
        "OF",
        "AND",
        "OR",
        "IN",
        "AT",
        "TO",
        "FROM",
        "WITH",
        "BY",
        "AS",
        "IS",
        "WAS",
        "ARE",
        "WERE",
        "BE",
        "BEEN",
        "A",
        "AN",
        "THIS",
        "THAT",
        "THESE",
        "THOSE",
    }
    words = re.findall("[A-Z][a-z]+|[A-Z]+(?=[A-Z][a-z]|\\b)", label)
    terms: list[str] = []
    for word in words:
        word_upper = word.upper()
        if word_upper not in skip_words and len(word_upper) >= MIN_LABEL_TERM_LENGTH:
            terms.append(_normalize_text(word))
    terms.append(_normalize_text(label))
    return terms


@lru_cache(maxsize=128)
def build_variable_patterns(domain: SDTMDomain) -> dict[str, list[str]]:
    patterns: dict[str, list[str]] = {}
    domain_prefix = domain.code.upper()
    for variable in domain.variables:
        var_patterns: list[str] = []
        var_name = variable.name.upper()
        var_patterns.append(_normalize_text(var_name))
        if var_name.startswith(domain_prefix) and len(var_name) > len(domain_prefix):
            suffix = var_name[len(domain_prefix) :]
            var_patterns.append(_normalize_text(suffix))
            if variable.implements and variable.implements.startswith("--"):
                general_suffix = variable.implements[2:]
                var_patterns.append(_normalize_text(general_suffix))
        if variable.label:
            label_terms = _extract_label_terms(variable.label)
            var_patterns.extend(label_terms)
        patterns[var_name] = _deduplicate_preserving_order(var_patterns)
    return patterns


@lru_cache(maxsize=32)
def get_domain_suffix_patterns(domain: SDTMDomain) -> dict[str, list[str]]:
    suffix_patterns: dict[str, list[str]] = {}
    domain_prefix = domain.code.upper()
    for variable in domain.variables:
        var_name = variable.name.upper()
        if var_name.startswith(domain_prefix) and len(var_name) > len(domain_prefix):
            suffix = var_name[len(domain_prefix) :]
            patterns: list[str] = [_normalize_text(suffix)]
            if variable.implements and variable.implements.startswith("--"):
                general_suffix = variable.implements[2:]
                patterns.append(_normalize_text(general_suffix))
            if variable.label:
                label_terms = _extract_label_terms(variable.label)
                patterns.extend(label_terms)
            suffix_patterns[suffix] = _deduplicate_preserving_order(patterns)
    return suffix_patterns
