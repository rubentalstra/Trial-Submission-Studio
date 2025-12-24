from pathlib import Path
from typing import cast

import pandas as pd

from ...config import TranspilerConfig
from ..caching.memory_cache import MemoryCache


class SDTMSpecRepository:
    pass

    def __init__(
        self,
        config: TranspilerConfig | None = None,
        cache: MemoryCache[object] | None = None,
    ) -> None:
        super().__init__()
        self._config = config or TranspilerConfig()
        self._cache: MemoryCache[object] = cache or MemoryCache()
        self._variables_cache_key = "sdtm_variables"
        self._datasets_cache_key = "sdtm_datasets"

    def get_domain_variables(self, domain_code: str) -> list[dict[str, str]]:
        variables_by_domain = self._load_variables()
        return variables_by_domain.get(domain_code.upper(), [])

    def get_dataset_attributes(self, domain_code: str) -> dict[str, str] | None:
        datasets = self._load_datasets()
        return datasets.get(domain_code.upper())

    def list_available_domains(self) -> list[str]:
        variables_by_domain = self._load_variables()
        return sorted(variables_by_domain.keys())

    def _load_variables(self) -> dict[str, list[dict[str, str]]]:
        cached = self._cache.get(self._variables_cache_key)
        if isinstance(cached, dict):
            return cast("dict[str, list[dict[str, str]]]", cached)
        variables_by_domain: dict[str, list[dict[str, str]]] = {}
        variables_file = self._resolve_spec_path("Variables.csv")
        if variables_file and variables_file.exists():
            df = self._read_spec_csv(variables_file)
            if df is not None:
                variables_by_domain = self._parse_variables_df(df)
        self._cache.set(self._variables_cache_key, variables_by_domain)
        return variables_by_domain

    def _load_datasets(self) -> dict[str, dict[str, str]]:
        cached = self._cache.get(self._datasets_cache_key)
        if isinstance(cached, dict):
            return cast("dict[str, dict[str, str]]", cached)
        datasets: dict[str, dict[str, str]] = {}
        datasets_file = self._resolve_spec_path("Datasets.csv")
        if datasets_file and datasets_file.exists():
            df = self._read_spec_csv(datasets_file)
            if df is not None:
                datasets = self._parse_datasets_df(df)
        self._cache.set(self._datasets_cache_key, datasets)
        return datasets

    def _resolve_spec_path(self, filename: str) -> Path | None:
        spec_dir = self._config.sdtm_spec_dir
        if not spec_dir.is_absolute():
            package_root = Path(__file__).resolve().parent.parent.parent.parent
            spec_dir = package_root / spec_dir
        spec_path = spec_dir / filename
        if spec_path.exists():
            return spec_path
        return None

    @staticmethod
    def _read_spec_csv(path: Path) -> pd.DataFrame | None:
        try:
            return pd.read_csv(path, dtype=str, na_filter=False)
        except Exception:
            return None

    def _parse_variables_df(self, df: pd.DataFrame) -> dict[str, list[dict[str, str]]]:
        result: dict[str, list[dict[str, str]]] = {}
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
            var_dict = {col: str(row[col]) for col in df.columns}
            if domain not in result:
                result[domain] = []
            result[domain].append(var_dict)
        return result

    def _parse_datasets_df(self, df: pd.DataFrame) -> dict[str, dict[str, str]]:
        result: dict[str, dict[str, str]] = {}
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
            attrs = {}
            for col in df.columns:
                key = col.lower().replace(" ", "_")
                attrs[key] = str(row[col])
            result[domain] = attrs
        return result

    def clear_cache(self) -> None:
        self._cache.clear()
