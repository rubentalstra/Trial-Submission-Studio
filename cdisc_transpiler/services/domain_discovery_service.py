"""Domain Discovery Service - File discovery and classification.

This service is responsible for discovering CSV files in a study folder
and classifying them by SDTM domain. It handles domain variants and
groups related files together.

SDTM Reference:
    Domain file naming follows SDTMIG v3.4 conventions:
    - Base domains: DM.csv, AE.csv, LB.csv
    - Domain variants: LBCC.csv, LBHM.csv (split datasets per Section 4.1.7)
    - Custom suffixes: LB_PREG.csv, QS_PGA.csv

Extracted from cli/commands/study.py as part of Phase 2 refactoring.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from ..infrastructure.sdtm_spec.utils import get_domain_class

if TYPE_CHECKING:
    from ..application.ports.services import LoggerPort


class DomainDiscoveryService:
    """Service for discovering and classifying domain files in a study folder.

    This service scans CSV files and matches them to SDTM domains,
    handling domain variants (e.g., LBCC, LBHM) and different naming conventions.

    File Matching Rules:
        1. Exact match: File contains domain code as a segment (e.g., _DM_ or _DM.)
        2. Prefix match: Segment starts with domain code (e.g., LBCC starts with LB)
        3. Metadata files are automatically skipped (CodeLists, Items, etc.)
    """

    def __init__(self, logger: LoggerPort | None = None):
        """Initialize the domain discovery service.

        Args:
            logger: Optional logger for verbose output. If None, logging is silently skipped.
        """
        self._logger = logger
        self._match_stats = {
            "total_files": 0,
            "matched_files": 0,
            "skipped_metadata": 0,
            "unmatched_files": 0,
        }

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
        self._match_stats = {
            "total_files": len(csv_files),
            "matched_files": 0,
            "skipped_metadata": 0,
            "unmatched_files": 0,
        }
        unmatched: list[str] = []

        for csv_file in csv_files:
            filename = csv_file.stem.upper()

            # Skip metadata files
            if self._is_metadata_file(filename):
                self._match_stats["skipped_metadata"] += 1
                self._log_verbose(f"Skipping metadata file: {csv_file.name}")
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
                self._match_stats["matched_files"] += 1

                # Enhanced logging with category info - use get_domain_class directly
                category = get_domain_class(matched_domain)
                match_type = "exact" if variant_name == matched_domain else "variant"
                self._log_verbose(
                    f"Matched {csv_file.name} → {matched_domain} "
                    f"(variant: {variant_name}, type: {match_type}, category: {category})"
                )
            else:
                self._match_stats["unmatched_files"] += 1
                unmatched.append(csv_file.name)
                self._log_verbose(f"No domain match for: {csv_file.name}")

        # Log summary statistics
        # Summary by category - use get_domain_class directly
        category_counts: dict[str, int] = {}
        for domain in domain_files.keys():
            category = get_domain_class(domain)
            category_counts[category] = category_counts.get(category, 0) + 1

        # Log detailed summary
        stats = self._match_stats
        self._log_verbose(
            f"File discovery complete: {stats['matched_files']}/{stats['total_files']} "
            f"files matched to {len(domain_files)} domains"
        )

        if stats["skipped_metadata"] > 0:
            self._log_verbose(f"  Metadata files skipped: {stats['skipped_metadata']}")

        if category_counts:
            summary = ", ".join(
                f"{cat}: {count}" for cat, count in sorted(category_counts.items())
            )
            self._log_verbose(f"  Domains by category: {summary}")

        if unmatched:
            self._log_verbose(
                f"  Unmatched files ({len(unmatched)}): {', '.join(unmatched[:5])}"
                + ("..." if len(unmatched) > 5 else "")
            )

        return domain_files

    def _log_verbose(self, message: str) -> None:
        """Log a verbose message if logger is available.

        Args:
            message: Message to log
        """
        if self._logger is not None:
            self._logger.verbose(message)

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
