use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

/// Cache wrapper for Redis/Dragonfly operations
#[derive(Clone)]
pub struct Cache {
    conn: ConnectionManager,
    default_ttl: Duration,
}

impl Cache {
    /// Create a new cache instance
    pub fn new(conn: ConnectionManager, ttl_seconds: u64) -> Self {
        Self {
            conn,
            default_ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Get a value from cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = self.conn.clone();
        let result: Option<String> = conn.get(key).await.ok()?;
        result.and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Set a value in cache with default TTL
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), redis::RedisError> {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    /// Set a value in cache with custom TTL
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(value).unwrap();
        conn.set_ex(key, json, ttl.as_secs()).await
    }

    /// Delete a key from cache
    pub async fn delete(&self, key: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        conn.del(key).await
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> bool {
        let mut conn = self.conn.clone();
        conn.exists(key).await.unwrap_or(false)
    }
}
