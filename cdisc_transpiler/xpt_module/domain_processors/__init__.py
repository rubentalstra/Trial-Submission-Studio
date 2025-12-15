"""Domain processor registry and factory.

This module provides a registry system for domain-specific processors,
allowing each SDTM domain to have custom post-processing logic.

NOTE: This module is a compatibility wrapper. The actual implementation
has been moved to `cdisc_transpiler.domain.services.domain_processors`.
"""

from __future__ import annotations

# Re-export everything from the domain services location
from ...domain.services.domain_processors import (
    BaseDomainProcessor,
    DefaultDomainProcessor,
    DomainProcessorRegistry,
    get_domain_processor,
    # Individual processors
    AEProcessor,
    CMProcessor,
    DAProcessor,
    DMProcessor,
    DSProcessor,
    EXProcessor,
    IEProcessor,
    LBProcessor,
    MHProcessor,
    PEProcessor,
    PRProcessor,
    QSProcessor,
    SEProcessor,
    TAProcessor,
    TEProcessor,
    TSProcessor,
    VSProcessor,
)

__all__ = [
    "BaseDomainProcessor",
    "DefaultDomainProcessor",
    "DomainProcessorRegistry",
    "get_domain_processor",
    # Individual processors
    "AEProcessor",
    "CMProcessor",
    "DAProcessor",
    "DMProcessor",
    "DSProcessor",
    "EXProcessor",
    "IEProcessor",
    "LBProcessor",
    "MHProcessor",
    "PEProcessor",
    "PRProcessor",
    "QSProcessor",
    "SEProcessor",
    "TAProcessor",
    "TEProcessor",
    "TSProcessor",
    "VSProcessor",
]
