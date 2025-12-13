"""Optimized mapping utilities and caching.

This module provides performance optimizations for the mapping engine:
- LRU caching for expensive operations
- Pre-compiled regex patterns
- Optimized fuzzy matching
"""

from __future__ import annotations

import re
from functools import lru_cache
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from rapidfuzz import fuzz

# Pre-compile regex patterns for performance
_NORMALIZE_PATTERN = re.compile(r"[^A-Z0-9]")
_SAFE_NAME_PATTERN = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")

# Cache for normalized strings
@lru_cache(maxsize=1024)
def normalize_text(text: str) -> str:
    """Normalize text for matching (cached)."""
    return _NORMALIZE_PATTERN.sub("", text.upper())


@lru_cache(maxsize=2048)
def compute_similarity(text1: str, text2: str, method: str = "token_set") -> float:
    """Compute similarity score with caching.
    
    Args:
        text1: First text
        text2: Second text
        method: Similarity method ('token_set' or 'ratio')
    
    Returns:
        Similarity score (0.0 to 1.0)
    """
    from rapidfuzz import fuzz
    
    if method == "token_set":
        return fuzz.token_set_ratio(text1.upper(), text2.upper()) / 100.0
    else:
        return fuzz.ratio(text1.upper(), text2.upper()) / 100.0


def is_safe_name(name: str) -> bool:
    """Check if name is SAS-safe (cached via pattern)."""
    return bool(_SAFE_NAME_PATTERN.match(name))


def make_safe_name(name: str) -> str:
    """Make column name SAS-safe.
    
    Args:
        name: Column name
        
    Returns:
        SAS-safe column name
    """
    if is_safe_name(name):
        return name
    # Escape and quote
    escaped = name.replace('"', '""')
    return f'"{escaped}"n'


# Cache for pattern matching
_pattern_cache: dict[str, dict[str, list[str]]] = {}


def get_inference_patterns() -> dict[str, dict[str, list[str]]]:
    """Get SDTM inference patterns (cached)."""
    if _pattern_cache:
        return _pattern_cache
    
    # Build patterns once
    from .mapping_module.constants import SDTM_INFERENCE_PATTERNS
    _pattern_cache.update(SDTM_INFERENCE_PATTERNS)
    return _pattern_cache


def clear_caches() -> None:
    """Clear all LRU caches (useful for testing)."""
    normalize_text.cache_clear()
    compute_similarity.cache_clear()
    _pattern_cache.clear()


def get_cache_info() -> dict[str, tuple]:
    """Get cache statistics for monitoring."""
    return {
        "normalize_text": normalize_text.cache_info(),
        "compute_similarity": compute_similarity.cache_info(),
        "pattern_cache_size": len(_pattern_cache),
    }
