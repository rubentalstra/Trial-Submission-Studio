"""Domain processor registry and factory.

This module provides a registry system for domain-specific processors,
allowing each SDTM domain to have custom post-processing logic.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd

from .base import BaseDomainProcessor, DefaultDomainProcessor

if TYPE_CHECKING:
    from ...domains import SDTMDomain


class DomainProcessorRegistry:
    """Registry for domain-specific processors.
    
    This registry maintains a mapping of domain codes to processor classes,
    allowing the system to apply domain-specific logic as needed.
    """

    def __init__(self):
        self._processors: dict[str, type[BaseDomainProcessor]] = {}
        self._default_processor = DefaultDomainProcessor

    def register(self, domain_code: str, processor_class: type[BaseDomainProcessor]):
        """Register a processor for a specific domain.
        
        Args:
            domain_code: SDTM domain code (e.g., "DM", "AE")
            processor_class: Processor class to use for this domain
        """
        self._processors[domain_code.upper()] = processor_class

    def get_processor(
        self,
        domain: "SDTMDomain",
        reference_starts: dict[str, str] | None = None,
        metadata=None,
    ) -> BaseDomainProcessor:
        """Get the appropriate processor for a domain.
        
        Args:
            domain: SDTM domain definition
            reference_starts: Mapping of USUBJID -> RFSTDTC
            metadata: Study metadata (optional)
            
        Returns:
            Processor instance for the domain
        """
        processor_class = self._processors.get(
            domain.code.upper(),
            self._default_processor
        )
        return processor_class(domain, reference_starts, metadata)


# Global registry instance
_registry = DomainProcessorRegistry()


def get_domain_processor(
    domain: "SDTMDomain",
    reference_starts: dict[str, str] | None = None,
    metadata=None,
) -> BaseDomainProcessor:
    """Get a processor for the specified domain.
    
    This is the main entry point for domain-specific processing.
    
    Args:
        domain: SDTM domain definition
        reference_starts: Mapping of USUBJID -> RFSTDTC
        metadata: Study metadata (optional)
        
    Returns:
        Processor instance for the domain
        
    Example:
        >>> processor = get_domain_processor(domain, reference_starts)
        >>> processor.process(dataframe)
    """
    return _registry.get_processor(domain, reference_starts, metadata)


def register_processor(domain_code: str, processor_class: type[BaseDomainProcessor]):
    """Register a custom processor for a domain.
    
    Args:
        domain_code: SDTM domain code (e.g., "DM", "AE")
        processor_class: Processor class to use for this domain
        
    Example:
        >>> class CustomDMProcessor(BaseDomainProcessor):
        ...     def process(self, frame):
        ...         # Custom DM processing
        ...         pass
        >>> register_processor("DM", CustomDMProcessor)
    """
    _registry.register(domain_code, processor_class)




# Register the default processor for all domains
# Domain-specific processors can be added later by creating processor classes
# and registering them with register_processor()
_registry._default_processor = DefaultDomainProcessor
