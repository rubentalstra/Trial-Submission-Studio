"""Domain Discovery Service - File discovery and classification.

This service is responsible for discovering CSV files in a study folder
and classifying them by SDTM domain. It handles domain variants and
groups related files together.

Extracted from cli/commands/study.py as part of Phase 2 refactoring.
"""

from __future__ import annotations

from pathlib import Path
from typing import Protocol


class Logger(Protocol):
    """Protocol for logging operations."""

    def log_verbose(self, message: str) -> None:
        """Log a verbose message."""
        ...


class DomainDiscoveryService:
    """Service for discovering and classifying domain files in a study folder.

    This service scans CSV files and matches them to SDTM domains,
    handling domain variants (e.g., LBCC, LBHM) and different naming conventions.
    """

    def __init__(self, logger: Logger | None = None):
        """Initialize the domain discovery service.

        Args:
            logger: Optional logger for verbose output
        """
        self.logger = logger

    def discover_domain_files(
        self,
        csv_files: list[Path],
        supported_domains: list[str],
    ) -> dict[str, list[tuple[Path, str]]]:
        """Discover and map CSV files to SDTM domains.

        Args:
            csv_files: List of CSV file paths to classify
            supported_domains: List of supported SDTM domain codes

        Returns:
            Dictionary mapping domain codes to list of (file_path, variant_name) tuples

        Examples:
            >>> service = DomainDiscoveryService()
            >>> files = [Path("DM.csv"), Path("LBCC.csv"), Path("LB_PREG.csv")]
            >>> domains = ["DM", "LB"]
            >>> result = service.discover_domain_files(files, domains)
            >>> result["DM"]
            [(Path("DM.csv"), "DM")]
            >>> result["LB"]
            [(Path("LBCC.csv"), "LBCC"), (Path("LB_PREG.csv"), "LB_PREG")]
        """
        domain_files: dict[str, list[tuple[Path, str]]] = {}

        for csv_file in csv_files:
            filename = csv_file.stem.upper()

            # Skip metadata files
            if self._is_metadata_file(filename):
                self._log(f"Skipping metadata file: {csv_file.name}")
                continue

            # Try to match the file to a domain
            matched_domain, variant_name = self._match_domain(
                filename, supported_domains
            )

            if matched_domain:
                if matched_domain not in domain_files:
                    domain_files[matched_domain] = []
                domain_files[matched_domain].append(
                    (csv_file, variant_name or matched_domain)
                )
                self._log(
                    f"Matched {csv_file.name} -> {matched_domain} (variant: {variant_name})"
                )
            else:
                self._log(f"No domain match for: {csv_file.name}")

        return domain_files

    def _is_metadata_file(self, filename: str) -> bool:
        """Check if a filename should be skipped as metadata.

        Args:
            filename: Uppercase filename stem

        Returns:
            True if the file should be skipped
        """
        skip_patterns = ["CODELISTS", "CODELIST", "ITEMS", "README", "METADATA"]
        return any(skip in filename for skip in skip_patterns)

    def _match_domain(
        self, filename: str, supported_domains: list[str]
    ) -> tuple[str | None, str | None]:
        """Match a filename to a domain code and variant name.

        Args:
            filename: Uppercase filename stem
            supported_domains: List of supported domain codes

        Returns:
            Tuple of (domain_code, variant_name) or (None, None) if no match
        """
        parts = filename.split("_")

        # Try each part of the filename
        for i, part in enumerate(parts):
            # Exact match
            if part in supported_domains:
                matched_domain = part
                # Build variant name from this part onwards
                if i < len(parts) - 1:
                    variant_name = "_".join(parts[i:])
                else:
                    variant_name = part
                return matched_domain, variant_name

            # Prefix match (e.g., "LBCC" starts with "LB")
            for domain_code in supported_domains:
                if part.startswith(domain_code) and len(part) > len(domain_code):
                    return domain_code, part

        return None, None

    def _log(self, message: str) -> None:
        """Log a message if logger is available.

        Args:
            message: Message to log
        """
        if self.logger:
            self.logger.log_verbose(message)
