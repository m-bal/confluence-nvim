//! Content caching module

use lru::LruCache;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Invalid cache capacity: {0}")]
    InvalidCapacity(String),
}

/// Cached content entry
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    inserted_at: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            inserted_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.inserted_at.elapsed() > self.ttl
    }
}

/// LRU cache for Confluence content
pub struct ContentCache {
    cache: LruCache<String, CacheEntry<String>>,
    default_ttl: Duration,
}

impl ContentCache {
    /// Create a new content cache with specified capacity (C-08: Fixed panic)
    pub fn new(capacity: usize) -> Result<Self, CacheError> {
        let non_zero_capacity = NonZeroUsize::new(capacity).ok_or_else(|| {
            CacheError::InvalidCapacity("Capacity must be greater than zero".to_string())
        })?;

        Ok(Self {
            cache: LruCache::new(non_zero_capacity),
            default_ttl: Duration::from_secs(900), // 15 minutes
        })
    }

    /// Get a value from cache if it exists and is not expired
    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Some(entry) = self.cache.get(key) {
            if entry.is_expired() {
                tracing::debug!("Cache entry expired for key: {}", key);
                self.cache.pop(key);
                None
            } else {
                tracing::debug!("Cache hit for key: {}", key);
                Some(entry.value.clone())
            }
        } else {
            tracing::debug!("Cache miss for key: {}", key);
            None
        }
    }

    /// Insert a value into the cache
    pub fn insert(&mut self, key: String, value: String) {
        let entry = CacheEntry::new(value, self.default_ttl);
        self.cache.put(key, entry);
    }

    /// Insert a value with custom TTL
    pub fn insert_with_ttl(&mut self, key: String, value: String, ttl: Duration) {
        let entry = CacheEntry::new(value, ttl);
        self.cache.put(key, entry);
    }

    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.cache.clear();
        tracing::info!("Cache cleared");
    }

    /// Get the current cache capacity
    pub fn capacity(&self) -> usize {
        self.cache.cap().get()
    }

    /// Get the number of items currently in cache
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_cache_creation() {
        let cache = ContentCache::new(10).unwrap();
        assert_eq!(cache.capacity(), 10);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_zero_capacity_error() {
        let result = ContentCache::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = ContentCache::new(10).unwrap();

        cache.insert("key1".to_string(), "value1".to_string());

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = ContentCache::new(10).unwrap();
        assert_eq!(cache.get("nonexistent"), None);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = ContentCache::new(10).unwrap();

        // Insert with very short TTL
        cache.insert_with_ttl(
            "key1".to_string(),
            "value1".to_string(),
            Duration::from_millis(100),
        );

        // Should be in cache immediately
        assert_eq!(cache.get("key1"), Some("value1".to_string()));

        // Wait for expiration
        sleep(Duration::from_millis(150));

        // Should be expired and removed
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = ContentCache::new(2).unwrap();

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        cache.insert("key3".to_string(), "value3".to_string());

        // key1 should be evicted (LRU)
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), Some("value2".to_string()));
        assert_eq!(cache.get("key3"), Some("value3".to_string()));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = ContentCache::new(10).unwrap();

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
}
