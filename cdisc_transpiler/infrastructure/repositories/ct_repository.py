"""Controlled Terminology Repository implementation.

This module provides access to CDISC Controlled Terminology through a
clean repository interface with configurable paths and caching.
"""

from functools import lru_cache
from pathlib import Path

from ...application.ports.repositories import CTRepositoryPort
from ...config import TranspilerConfig
from ...domain.entities.controlled_terminology import ControlledTerminology
from ..caching import MemoryCache
from .ct_loader import build_registry

_DEFAULT_CT_CACHE = MemoryCache()


class CTRepository:
    """Repository for CDISC Controlled Terminology data.

    This implementation loads CT from CSV files in a configured directory
    and provides lookup by codelist code or name. Uses in-memory caching
    to avoid re-reading large CT files.

    Example:
        >>> config = TranspilerConfig(ct_dir=Path("docs/Controlled_Terminology"))
        >>> repo = CTRepository(config)
        >>> ct = repo.get_by_code("C66767")  # SEX codelist
        >>> if ct:
        ...     print(f"Values: {ct.submission_values}")
    """

    def __init__(
        self,
        config: TranspilerConfig | None = None,
        cache: MemoryCache | None = None,
        ct_version: str | None = None,
    ) -> None:
        """Initialize the repository.

        Args:
            config: Configuration with CT directory paths. Uses default if None.
            cache: Optional cache instance for memoization. Creates one if None.
            ct_version: CT version folder name (e.g., "2025-09-26").
                       If None, uses latest version folder.
        """
        super().__init__()
        self._config = config or TranspilerConfig()
        # Default to a shared cache so repeated CTRepository() instantiations
        # (e.g., per normalization call) don't re-load large CT registries.
        self._cache = cache or _DEFAULT_CT_CACHE
        self._ct_version = ct_version
        self._registry_cache_key = "ct_registry"

    def get_by_code(self, codelist_code: str) -> ControlledTerminology | None:
        """Retrieve controlled terminology by codelist code.

        Args:
            codelist_code: NCI codelist code (e.g., "C66767" for SEX)

        Returns:
            ControlledTerminology object if found, None otherwise
        """
        by_code, _ = self._load_registry()
        return by_code.get(codelist_code.strip().upper())

    def get_by_name(self, codelist_name: str) -> ControlledTerminology | None:
        """Retrieve controlled terminology by codelist name.

        Args:
            codelist_name: Human-readable codelist name (e.g., "SEX")

        Returns:
            ControlledTerminology object if found, None otherwise
        """
        _, by_name = self._load_registry()
        return by_name.get(codelist_name.strip().upper())

    def list_all_codes(self) -> list[str]:
        """List all available codelist codes.

        Returns:
            List of all NCI codelist codes available in the repository
        """
        by_code, _ = self._load_registry()
        return sorted(by_code.keys())

    def _load_registry(
        self,
    ) -> tuple[
        dict[str, ControlledTerminology],
        dict[str, ControlledTerminology],
    ]:
        """Load and cache the CT registries.

        Returns:
            Tuple of (registry_by_code, registry_by_name)
        """
        cached = self._cache.get(self._registry_cache_key)
        if cached is not None:
            return cached

        ct_dir = self._resolve_ct_dir()

        if ct_dir and ct_dir.exists():
            try:
                by_code, by_name = build_registry(ct_dir)
            except Exception:
                by_code, by_name = {}, {}
        else:
            by_code, by_name = {}, {}

        result = (by_code, by_name)
        self._cache.set(self._registry_cache_key, result)
        return result

    def _resolve_ct_dir(self) -> Path | None:
        """Resolve the CT directory path.

        If ct_version is specified, uses that subfolder.
        Otherwise, uses the latest (alphabetically last) subfolder.

        Returns:
            Path to the CT directory or None if not found
        """
        ct_base = self._config.ct_dir

        # Make path absolute if it's relative
        if not ct_base.is_absolute():
            package_root = Path(__file__).resolve().parent.parent.parent.parent
            ct_base = package_root / ct_base

        if not ct_base.exists():
            return None

        # If a specific version is requested
        if self._ct_version:
            target = ct_base / self._ct_version
            if target.exists():
                return target
            return None

        # Otherwise, find the latest version folder
        candidates = sorted(
            [d for d in ct_base.iterdir() if d.is_dir() and not d.name.startswith(".")]
        )

        if candidates:
            return candidates[-1]  # Latest by name (ISO date naming)

        # Fall back to the base directory if no subfolders
        return ct_base

    def clear_cache(self) -> None:
        """Clear the cache to force re-reading from disk."""
        self._cache.clear()


@lru_cache(maxsize=1)
def get_default_ct_repository() -> CTRepositoryPort:
    """Return a cached default CT repository instance.

    The repository itself uses a shared in-memory cache, but centralizing the
    default instance removes duplicated singleton patterns across adapters.
    """
    return CTRepository()


# Verify protocol compliance at runtime (duck typing)
def _verify_protocol_compliance() -> None:
    """Verify CTRepository implements CTRepositoryPort."""
    repo: CTRepositoryPort = CTRepository()
    assert isinstance(repo, CTRepositoryPort)


_verify_protocol_compliance()
