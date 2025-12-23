"""Dynamic pattern generation for SDTM variable mapping.

This module generates mapping patterns dynamically from the domain metadata
instead of using hardcoded patterns.
"""

from functools import lru_cache
import re

from ...entities.sdtm_domain import SDTMDomain


def _deduplicate_preserving_order(items: list[str]) -> list[str]:
    """Remove duplicates while preserving first occurrence order.

    Args:
        items: List that may contain duplicates

    Returns:
        List with duplicates removed, first occurrence preserved
    """
    return list(dict.fromkeys(items))


def _normalize_text(text: str) -> str:
    """Normalize text for pattern matching (uppercase, alphanumeric only)."""
    return re.sub(r"[^A-Z0-9]", "", text.upper())


def _extract_label_terms(label: str) -> list[str]:
    """Extract meaningful search terms from a variable label.

    Args:
        label: Variable label (e.g., "Start Date/Time", "Test Name")

    Returns:
        List of normalized search terms
    """
    # Common words to skip
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

    # Split on common delimiters and extract words
    # Regex splits CamelCase/PascalCase (e.g., "StartDate" -> ["Start", "Date"])
    # Pattern: [A-Z][a-z]+ matches capitalized words, [A-Z]+(?=[A-Z][a-z]|\b) matches acronyms
    words = re.findall(r"[A-Z][a-z]+|[A-Z]+(?=[A-Z][a-z]|\b)", label)
    terms: list[str] = []

    for word in words:
        word_upper = word.upper()
        if word_upper not in skip_words and len(word_upper) >= 3:
            terms.append(_normalize_text(word))

    # Also add the full normalized label
    terms.append(_normalize_text(label))

    return terms


@lru_cache(maxsize=128)
def build_variable_patterns(domain: SDTMDomain) -> dict[str, list[str]]:
    """Build search patterns for each variable in a domain.

    This generates patterns dynamically from variable metadata:
    - Variable name itself
    - Suffix (for domain-prefixed variables)
    - Terms extracted from variable label

    Args:
        domain: SDTM domain definition

    Returns:
        Dictionary mapping target variable names to list of search patterns
    """
    patterns: dict[str, list[str]] = {}
    domain_prefix = domain.code.upper()

    for variable in domain.variables:
        var_patterns: list[str] = []
        var_name = variable.name.upper()

        # Add the exact variable name
        var_patterns.append(_normalize_text(var_name))

        # If variable is domain-prefixed, add the suffix
        if var_name.startswith(domain_prefix) and len(var_name) > len(domain_prefix):
            suffix = var_name[len(domain_prefix) :]
            var_patterns.append(_normalize_text(suffix))

            # If this implements a general class pattern (e.g., --STDTC)
            if variable.implements and variable.implements.startswith("--"):
                general_suffix = variable.implements[2:]
                var_patterns.append(_normalize_text(general_suffix))

        # Add patterns from variable label
        if variable.label:
            label_terms = _extract_label_terms(variable.label)
            var_patterns.extend(label_terms)

        # Store unique patterns (deduplicated while preserving order)
        patterns[var_name] = _deduplicate_preserving_order(var_patterns)

    return patterns


@lru_cache(maxsize=32)
def get_domain_suffix_patterns(domain: SDTMDomain) -> dict[str, list[str]]:
    """Get suffix-based patterns for domain variables.

    This extracts the suffix patterns (e.g., AETERM -> TERM, AESTDTC -> STDTC)
    that can be used for pattern matching.

    Args:
        domain: SDTM domain definition

    Returns:
        Dictionary mapping suffixes to list of search patterns
    """
    suffix_patterns: dict[str, list[str]] = {}
    domain_prefix = domain.code.upper()

    for variable in domain.variables:
        var_name = variable.name.upper()

        # Extract suffix for domain-prefixed variables
        if var_name.startswith(domain_prefix) and len(var_name) > len(domain_prefix):
            suffix = var_name[len(domain_prefix) :]

            patterns: list[str] = [_normalize_text(suffix)]

            # Add general class pattern if available (e.g., --STDTC)
            if variable.implements and variable.implements.startswith("--"):
                general_suffix = variable.implements[2:]
                patterns.append(_normalize_text(general_suffix))

            # Add label-based patterns
            if variable.label:
                label_terms = _extract_label_terms(variable.label)
                patterns.extend(label_terms)

            suffix_patterns[suffix] = _deduplicate_preserving_order(patterns)

    return suffix_patterns
