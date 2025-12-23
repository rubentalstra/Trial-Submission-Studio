"""Tests for MemoryCache."""

from datetime import timedelta
import time

from cdisc_transpiler.infrastructure.caching.memory_cache import MemoryCache


class TestMemoryCache:
    """Tests for MemoryCache functionality."""

    def test_set_and_get(self):
        """Test basic set and get operations."""
        cache = MemoryCache()
        cache.set("key1", {"data": [1, 2, 3]})

        result = cache.get("key1")

        assert result == {"data": [1, 2, 3]}

    def test_get_missing_key(self):
        """Test getting a key that doesn't exist."""
        cache = MemoryCache()

        result = cache.get("nonexistent")

        assert result is None

    def test_clear(self):
        """Test clearing the cache."""
        cache = MemoryCache()
        cache.set("key1", "value1")
        cache.set("key2", "value2")

        cache.clear()

        assert cache.get("key1") is None
        assert cache.get("key2") is None
        assert cache.size() == 0

    def test_delete(self):
        """Test deleting a specific key."""
        cache = MemoryCache()
        cache.set("key1", "value1")
        cache.set("key2", "value2")

        result = cache.delete("key1")

        assert result is True
        assert cache.get("key1") is None
        assert cache.get("key2") == "value2"

    def test_delete_missing_key(self):
        """Test deleting a key that doesn't exist."""
        cache = MemoryCache()

        result = cache.delete("nonexistent")

        assert result is False

    def test_has(self):
        """Test checking if a key exists."""
        cache = MemoryCache()
        cache.set("key1", "value1")

        assert cache.has("key1") is True
        assert cache.has("nonexistent") is False

    def test_size(self):
        """Test getting the cache size."""
        cache = MemoryCache()

        assert cache.size() == 0

        cache.set("key1", "value1")
        assert cache.size() == 1

        cache.set("key2", "value2")
        assert cache.size() == 2

    def test_keys(self):
        """Test getting all keys."""
        cache = MemoryCache()
        cache.set("key1", "value1")
        cache.set("key2", "value2")

        keys = cache.keys()

        assert sorted(keys) == ["key1", "key2"]

    def test_overwrite_value(self):
        """Test overwriting an existing value."""
        cache = MemoryCache()
        cache.set("key1", "old_value")
        cache.set("key1", "new_value")

        result = cache.get("key1")

        assert result == "new_value"

    def test_ttl_expiration(self):
        """Test that entries expire after TTL."""
        cache = MemoryCache()
        # Use a very short TTL for testing
        cache.set("key1", "value1", ttl=timedelta(milliseconds=50))

        # Should exist immediately
        assert cache.get("key1") == "value1"

        # Wait for expiration
        time.sleep(0.1)

        # Should be expired now
        assert cache.get("key1") is None

    def test_default_ttl(self):
        """Test default TTL set on cache initialization."""
        cache = MemoryCache(default_ttl=timedelta(milliseconds=50))
        cache.set("key1", "value1")

        # Should exist immediately
        assert cache.get("key1") == "value1"

        # Wait for expiration
        time.sleep(0.1)

        # Should be expired now
        assert cache.get("key1") is None

    def test_no_ttl_never_expires(self):
        """Test that entries without TTL never expire."""
        cache = MemoryCache()  # No default TTL
        cache.set("key1", "value1")  # No per-entry TTL

        # Should exist
        assert cache.get("key1") == "value1"

        # Still exists (no expiration)
        assert cache.get("key1") == "value1"

    def test_per_entry_ttl_overrides_default(self):
        """Test that per-entry TTL overrides default TTL."""
        cache = MemoryCache(default_ttl=timedelta(hours=1))
        cache.set("key1", "value1", ttl=timedelta(milliseconds=50))

        # Should exist immediately
        assert cache.get("key1") == "value1"

        # Wait for per-entry TTL expiration
        time.sleep(0.1)

        # Should be expired now
        assert cache.get("key1") is None

    def test_has_respects_expiration(self):
        """Test that has() returns False for expired entries."""
        cache = MemoryCache()
        cache.set("key1", "value1", ttl=timedelta(milliseconds=50))

        assert cache.has("key1") is True

        time.sleep(0.1)

        assert cache.has("key1") is False
