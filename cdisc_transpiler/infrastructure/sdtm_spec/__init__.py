"""Infrastructure SDTM spec loading utilities."""

from .constants import ALWAYS_PROPAGATE_GENERAL, CT_VERSION
from .registry import (
    generalized_identifiers,
    get_domain,
    get_domain_class,
    list_domains,
)

__all__ = [
    "ALWAYS_PROPAGATE_GENERAL",
    "CT_VERSION",
    "generalized_identifiers",
    "get_domain",
    "get_domain_class",
    "list_domains",
]
