"""SDTM Specification Repository implementation.

This module provides access to SDTM Implementation Guide specifications
(domain variables, dataset attributes) through a clean repository interface.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import pandas as pd

from ...application.ports.repositories import SDTMSpecRepositoryPort
from ...config import TranspilerConfig
from ..caching import MemoryCache


class SDTMSpecRepository:
    """Repository for SDTM specification data.

    This implementation loads SDTM specifications from CSV files (Variables.csv,
    Datasets.csv) and provides them through the SDTMSpecRepositoryPort interface.

    Uses configurable paths via TranspilerConfig instead of hardcoded paths,
    and provides in-memory caching to avoid re-reading CSVs on every access.

    Example:
        >>> config = TranspilerConfig(sdtm_spec_dir=Path("docs/SDTMIG_v3.4"))
        >>> repo = SDTMSpecRepository(config)
        >>> variables = repo.get_domain_variables("DM")
        >>> for var in variables:
        ...     print(f"{var['Variable Name']}: {var['Label']}")
    """

    def __init__(
        self,
        config: TranspilerConfig | None = None,
        cache: MemoryCache | None = None,
    ):
        """Initialize the repository.

        Args:
            config: Configuration with spec directory paths. Uses default if None.
            cache: Optional cache instance for memoization. Creates one if None.
        """
        self._config = config or TranspilerConfig()
        self._cache = cache or MemoryCache()
        self._variables_cache_key = "sdtm_variables"
        self._datasets_cache_key = "sdtm_datasets"

    def get_domain_variables(self, domain_code: str) -> list[dict[str, str]]:
        """Retrieve variable specifications for a domain.

        Args:
            domain_code: SDTM domain code (e.g., "DM", "AE", "LB")

        Returns:
            List of variable specification dictionaries containing metadata
            such as Variable Name, Label, Type, Length, Role, etc.
            Returns empty list if domain not found.
        """
        variables_by_domain = self._load_variables()
        return variables_by_domain.get(domain_code.upper(), [])

    def get_dataset_attributes(self, domain_code: str) -> dict[str, str] | None:
        """Retrieve dataset-level attributes for a domain.

        Args:
            domain_code: SDTM domain code (e.g., "DM", "AE", "LB")

        Returns:
            Dictionary with dataset attributes like class, label, and structure,
            or None if domain not found
        """
        datasets = self._load_datasets()
        return datasets.get(domain_code.upper())

    def list_available_domains(self) -> list[str]:
        """List all available SDTM domains in the specification.

        Returns:
            List of domain codes available in the SDTM specification
        """
        variables_by_domain = self._load_variables()
        return sorted(variables_by_domain.keys())

    def _load_variables(self) -> dict[str, list[dict[str, str]]]:
        """Load and cache variables from CSV.

        Returns:
            Dictionary mapping domain codes to lists of variable dictionaries
        """
        cached = self._cache.get(self._variables_cache_key)
        if cached is not None:
            return cached

        variables_by_domain: dict[str, list[dict[str, str]]] = {}

        # Try SDTMIG v3.4 first, then SDTM v2.0 as fallback
        variables_file = self._resolve_spec_path("Variables.csv")

        if variables_file and variables_file.exists():
            try:
                df = pd.read_csv(variables_file, dtype=str, na_filter=False)
                variables_by_domain = self._parse_variables_df(df)
            except Exception:
                pass  # Fall through to empty dict

        self._cache.set(self._variables_cache_key, variables_by_domain)
        return variables_by_domain

    def _load_datasets(self) -> dict[str, dict[str, str]]:
        """Load and cache dataset attributes from CSV.

        Returns:
            Dictionary mapping domain codes to dataset attribute dictionaries
        """
        cached = self._cache.get(self._datasets_cache_key)
        if cached is not None:
            return cached

        datasets: dict[str, dict[str, str]] = {}

        datasets_file = self._resolve_spec_path("Datasets.csv")

        if datasets_file and datasets_file.exists():
            try:
                df = pd.read_csv(datasets_file, dtype=str, na_filter=False)
                datasets = self._parse_datasets_df(df)
            except Exception:
                pass  # Fall through to empty dict

        self._cache.set(self._datasets_cache_key, datasets)
        return datasets

    def _resolve_spec_path(self, filename: str) -> Path | None:
        """Resolve the path to a spec file.

        Args:
            filename: Name of the spec file (e.g., "Variables.csv")

        Returns:
            Path to the spec file if found, None otherwise
        """
        # Try the configured directory first
        spec_dir = self._config.sdtm_spec_dir

        # Make path absolute if it's relative
        if not spec_dir.is_absolute():
            # Try from package location
            package_root = Path(__file__).resolve().parent.parent.parent.parent
            spec_dir = package_root / spec_dir

        spec_path = spec_dir / filename
        if spec_path.exists():
            return spec_path

        return None

    def _parse_variables_df(self, df: pd.DataFrame) -> dict[str, list[dict[str, str]]]:
        """Parse a Variables DataFrame into domain-grouped dictionaries.

        Args:
            df: DataFrame from Variables.csv

        Returns:
            Dictionary mapping domain codes to lists of variable dictionaries
        """
        result: dict[str, list[dict[str, str]]] = {}

        # Find the domain column (could be "Domain" or "Dataset Name")
        domain_col = None
        for col in df.columns:
            if col.lower() in ("domain", "dataset name", "dataset"):
                domain_col = col
                break

        if domain_col is None:
            return result

        for _, row in df.iterrows():
            domain = str(row.get(domain_col, "")).strip().upper()
            if not domain or domain == "DOMAIN":
                continue

            # Convert row to dict, using column names as keys
            var_dict = {col: str(row[col]) for col in df.columns}

            if domain not in result:
                result[domain] = []
            result[domain].append(var_dict)

        return result

    def _parse_datasets_df(self, df: pd.DataFrame) -> dict[str, dict[str, str]]:
        """Parse a Datasets DataFrame into a domain-keyed dictionary.

        Args:
            df: DataFrame from Datasets.csv

        Returns:
            Dictionary mapping domain codes to attribute dictionaries
        """
        result: dict[str, dict[str, str]] = {}

        # Find the domain column
        domain_col = None
        for col in df.columns:
            if col.lower() in ("domain", "dataset name", "dataset"):
                domain_col = col
                break

        if domain_col is None:
            return result

        for _, row in df.iterrows():
            domain = str(row.get(domain_col, "")).strip().upper()
            if not domain or domain.lower() in ("domain", "dataset name"):
                continue

            # Convert row to dict with lowercase keys for consistency
            attrs = {}
            for col in df.columns:
                key = col.lower().replace(" ", "_")
                attrs[key] = str(row[col])

            result[domain] = attrs

        return result

    def clear_cache(self) -> None:
        """Clear the cache to force re-reading from disk."""
        self._cache.clear()


# Verify protocol compliance at runtime (duck typing)
def _verify_protocol_compliance() -> None:
    """Verify SDTMSpecRepository implements SDTMSpecRepositoryPort."""
    repo: SDTMSpecRepositoryPort = SDTMSpecRepository()
    assert isinstance(repo, SDTMSpecRepositoryPort)


_verify_protocol_compliance()
