from functools import lru_cache
from pathlib import Path
from typing import TYPE_CHECKING

from ...config import TranspilerConfig
from ..caching.memory_cache import MemoryCache
from .ct_loader import build_registry

_DEFAULT_CT_CACHE: MemoryCache[
    tuple[dict[str, ControlledTerminology], dict[str, ControlledTerminology]]
] = MemoryCache()
if TYPE_CHECKING:
    from ...application.ports.repositories import CTRepositoryPort
    from ...domain.entities.controlled_terminology import ControlledTerminology


class CTRepository:
    pass

    def __init__(
        self,
        config: TranspilerConfig | None = None,
        cache: MemoryCache[
            tuple[dict[str, ControlledTerminology], dict[str, ControlledTerminology]]
        ]
        | None = None,
        ct_version: str | None = None,
    ) -> None:
        super().__init__()
        self._config = config or TranspilerConfig()
        self._cache = cache or _DEFAULT_CT_CACHE
        self._ct_version = ct_version
        self._registry_cache_key = "ct_registry"

    def get_by_code(self, codelist_code: str) -> ControlledTerminology | None:
        by_code, _ = self._load_registry()
        return by_code.get(codelist_code.strip().upper())

    def get_by_name(self, codelist_name: str) -> ControlledTerminology | None:
        _, by_name = self._load_registry()
        return by_name.get(codelist_name.strip().upper())

    def list_all_codes(self) -> list[str]:
        by_code, _ = self._load_registry()
        return sorted(by_code.keys())

    def _load_registry(
        self,
    ) -> tuple[dict[str, ControlledTerminology], dict[str, ControlledTerminology]]:
        cached = self._cache.get(self._registry_cache_key)
        if cached is not None:
            return cached
        ct_dir = self._resolve_ct_dir()
        if ct_dir and ct_dir.exists():
            try:
                by_code, by_name = build_registry(ct_dir)
            except Exception:
                by_code, by_name = ({}, {})
        else:
            by_code, by_name = ({}, {})
        result = (by_code, by_name)
        self._cache.set(self._registry_cache_key, result)
        return result

    def _resolve_ct_dir(self) -> Path | None:
        ct_base = self._config.ct_dir
        if not ct_base.is_absolute():
            package_root = Path(__file__).resolve().parent.parent.parent.parent
            ct_base = package_root / ct_base
        if not ct_base.exists():
            return None
        if self._ct_version:
            target = ct_base / self._ct_version
            if target.exists():
                return target
            return None
        candidates = sorted(
            [
                d
                for d in ct_base.iterdir()
                if d.is_dir() and (not d.name.startswith("."))
            ]
        )
        if candidates:
            return candidates[-1]
        return ct_base

    def clear_cache(self) -> None:
        self._cache.clear()


@lru_cache(maxsize=1)
def get_default_ct_repository() -> CTRepositoryPort:
    return CTRepository()
