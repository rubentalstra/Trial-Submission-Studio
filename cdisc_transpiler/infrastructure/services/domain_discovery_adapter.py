"""Infrastructure adapter for SDTM domain file discovery."""

from pathlib import Path
from typing import override

from ...application.ports.services import DomainDiscoveryPort, LoggerPort
from ..sdtm_spec.registry import get_domain_class


class DomainDiscoveryAdapter(DomainDiscoveryPort):
    """Discover and classify SDTM domain CSV files in a study folder."""

    def __init__(self, *, logger: LoggerPort | None = None) -> None:
        super().__init__()
        self._logger = logger
        self._match_stats: dict[str, int] = {
            "total_files": 0,
            "matched_files": 0,
            "skipped_metadata": 0,
            "skipped_helpers": 0,
            "unmatched_files": 0,
        }

    @override
    def discover_domain_files(
        self,
        csv_files: list[Path],
        supported_domains: list[str],
    ) -> dict[str, list[tuple[Path, str]]]:
        domain_files: dict[str, list[tuple[Path, str]]] = {}
        self._match_stats = {
            "total_files": len(csv_files),
            "matched_files": 0,
            "skipped_metadata": 0,
            "skipped_helpers": 0,
            "unmatched_files": 0,
        }
        unmatched: list[str] = []

        for csv_file in csv_files:
            filename = csv_file.stem.upper()
            parts = filename.split("_")

            if self._is_metadata_file(filename):
                self._match_stats["skipped_metadata"] += 1
                self._log_verbose(f"Skipping metadata file: {csv_file.name}")
                continue

            if self._is_helper_file(parts):
                self._match_stats["skipped_helpers"] += 1
                self._log_verbose(
                    f"Skipping helper file: {csv_file.name} (not an SDTM domain)"
                )
                continue

            matched_domain, variant_name = self._match_domain(
                filename, supported_domains
            )

            if matched_domain:
                domain_files.setdefault(matched_domain, []).append(
                    (csv_file, variant_name or matched_domain)
                )
                self._match_stats["matched_files"] += 1

                category = get_domain_class(matched_domain)
                match_type = "exact" if variant_name == matched_domain else "variant"
                self._log_verbose(
                    f"Matched {csv_file.name} -> {matched_domain} (variant: {variant_name}, type: {match_type}, category: {category})"
                )
            else:
                self._match_stats["unmatched_files"] += 1
                unmatched.append(csv_file.name)
                self._log_verbose(f"No domain match for: {csv_file.name}")

        category_counts: dict[str, int] = {}
        for domain in domain_files:
            category = get_domain_class(domain)
            category_counts[category] = category_counts.get(category, 0) + 1

        stats = self._match_stats
        self._log_verbose(
            f"File discovery complete: {stats['matched_files']}/{stats['total_files']} files matched to {len(domain_files)} domains"
        )

        if stats["skipped_metadata"] > 0:
            self._log_verbose(f"  Metadata files skipped: {stats['skipped_metadata']}")

        if stats.get("skipped_helpers", 0) > 0:
            self._log_verbose(f"  Helper files skipped: {stats['skipped_helpers']}")

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
        if self._logger is not None:
            self._logger.verbose(message)

    def _is_metadata_file(self, filename: str) -> bool:
        skip_patterns = ["CODELISTS", "CODELIST", "ITEMS", "README", "METADATA"]
        return any(skip in filename for skip in skip_patterns)

    def _is_helper_file(self, filename_parts: list[str]) -> bool:
        if not filename_parts:
            return False
        return filename_parts[-1] in {"LC"}

    def _match_domain(
        self, filename: str, supported_domains: list[str]
    ) -> tuple[str | None, str | None]:
        exact_matches = [d for d in supported_domains if f"_{d}_" in f"_{filename}_"]
        if exact_matches:
            return exact_matches[0], exact_matches[0]

        parts = filename.split("_")
        if parts:
            prefix = parts[0]
            for domain in supported_domains:
                if prefix.startswith(domain):
                    return domain, prefix

        for domain in supported_domains:
            if filename.startswith(domain):
                return domain, filename

        return None, None
