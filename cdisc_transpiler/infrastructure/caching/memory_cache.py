"""In-memory caching primitives.

This module provides a simple in-memory cache with optional TTL support
for expensive operations like CSV parsing.
"""

from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Any, Generic, TypeVar

T = TypeVar("T")


@dataclass
class CacheEntry(Generic[T]):
    """A cache entry with optional expiration."""

    value: T
    created_at: datetime = field(default_factory=datetime.now)
    expires_at: datetime | None = None

    def is_expired(self) -> bool:
        """Check if this entry has expired."""
        if self.expires_at is None:
            return False
        return datetime.now() > self.expires_at


class MemoryCache:
    """Simple in-memory cache with optional TTL.

    This cache provides a controlled interface for caching expensive operations
    like CSV parsing. Unlike module-level global caches, this can be:
    - Cleared between tests
    - Configured per-instance
    - Injected as a dependency

    Example:
        >>> cache = MemoryCache()
        >>> cache.set("key1", {"data": [1, 2, 3]})
        >>> cache.get("key1")
        {'data': [1, 2, 3]}
        >>> cache.clear()
        >>> cache.get("key1") is None
        True
    """

    def __init__(self, default_ttl: timedelta | None = None):
        """Initialize the cache.

        Args:
            default_ttl: Default time-to-live for entries. None means no expiration.
        """
        super().__init__()
        self._store: dict[str, CacheEntry[Any]] = {}
        self._default_ttl = default_ttl

    def get(self, key: str) -> Any | None:
        """Get a value from the cache.

        Args:
            key: Cache key

        Returns:
            Cached value or None if not found or expired
        """
        entry = self._store.get(key)
        if entry is None:
            return None

        if entry.is_expired():
            del self._store[key]
            return None

        return entry.value

    def set(self, key: str, value: Any, ttl: timedelta | None = None) -> None:
        """Set a value in the cache.

        Args:
            key: Cache key
            value: Value to cache
            ttl: Time-to-live for this entry (overrides default_ttl)
        """
        expires_at = None
        effective_ttl = ttl if ttl is not None else self._default_ttl
        if effective_ttl is not None:
            expires_at = datetime.now() + effective_ttl

        self._store[key] = CacheEntry(
            value=value,
            expires_at=expires_at,
        )

    def delete(self, key: str) -> bool:
        """Delete a key from the cache.

        Args:
            key: Cache key

        Returns:
            True if the key existed and was deleted, False otherwise
        """
        if key in self._store:
            del self._store[key]
            return True
        return False

    def clear(self) -> None:
        """Clear all entries from the cache."""
        self._store.clear()

    def has(self, key: str) -> bool:
        """Check if a key exists and is not expired.

        Args:
            key: Cache key

        Returns:
            True if key exists and is not expired
        """
        return self.get(key) is not None

    def size(self) -> int:
        """Get the number of entries in the cache.

        Note: This returns the raw count of all entries, including entries
        that may be expired but have not been cleaned up through access.
        Expired entries are lazily removed when accessed via get() or has().

        Returns:
            Number of entries (may include not-yet-cleaned expired entries)
        """
        return len(self._store)

    def keys(self) -> list[str]:
        """Get all keys in the cache.

        Returns:
            List of cache keys
        """
        return list(self._store.keys())
