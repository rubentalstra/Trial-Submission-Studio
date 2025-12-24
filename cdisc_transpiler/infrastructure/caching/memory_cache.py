from dataclasses import dataclass, field
from datetime import datetime
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from datetime import timedelta


@dataclass(slots=True)
class CacheEntry[T]:
    value: T
    created_at: datetime = field(default_factory=datetime.now)
    expires_at: datetime | None = None

    def is_expired(self) -> bool:
        if self.expires_at is None:
            return False
        return datetime.now() > self.expires_at


class MemoryCache[T]:
    def __init__(self, default_ttl: timedelta | None = None) -> None:
        super().__init__()
        self._store: dict[str, CacheEntry[T]] = {}
        self._default_ttl = default_ttl

    def get(self, key: str) -> T | None:
        entry = self._store.get(key)
        if entry is None:
            return None

        if entry.is_expired():
            del self._store[key]
            return None

        return entry.value

    def set(self, key: str, value: T, ttl: timedelta | None = None) -> None:
        expires_at = None
        effective_ttl = ttl if ttl is not None else self._default_ttl
        if effective_ttl is not None:
            expires_at = datetime.now() + effective_ttl

        self._store[key] = CacheEntry(
            value=value,
            expires_at=expires_at,
        )

    def delete(self, key: str) -> bool:
        if key in self._store:
            del self._store[key]
            return True
        return False

    def clear(self) -> None:
        self._store.clear()

    def has(self, key: str) -> bool:
        return self.get(key) is not None

    def size(self) -> int:
        return len(self._store)

    def keys(self) -> list[str]:
        return list(self._store.keys())
