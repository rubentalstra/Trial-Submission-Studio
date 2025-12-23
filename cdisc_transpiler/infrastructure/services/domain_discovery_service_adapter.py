"""Infrastructure adapter for domain discovery.

Implements the application port DomainDiscoveryPort by delegating to the
existing discovery logic.

This keeps StudyProcessingUseCase decoupled from concrete discovery services.
"""

from pathlib import Path
from typing import override

from ...application.ports import DomainDiscoveryPort, LoggerPort


class DomainDiscoveryServiceAdapter(DomainDiscoveryPort):
    def __init__(self, *, logger: LoggerPort):
        from ...services.domain_discovery_service import DomainDiscoveryService

        self._delegate = DomainDiscoveryService(logger=logger)

    @override
    def discover_domain_files(
        self,
        csv_files: list[Path],
        supported_domains: list[str],
    ) -> dict[str, list[tuple[Path, str]]]:
        return self._delegate.discover_domain_files(
            csv_files=csv_files,
            supported_domains=supported_domains,
        )
