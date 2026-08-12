use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use tokio::sync::OnceCell;

use crate::error::AppError;

/// A minimal in-memory TTL cache for serialized JSON responses.
///
/// Expired entries are skipped on read and replaced on write. A
/// background task is not needed for a server of this size.
pub struct Cache {
    inner: RwLock<HashMap<String, Entry>>,
}

struct Entry {
    body: String,
    expires_at: Instant,
}

impl Cache {
    pub fn new() -> Self {
        Self { inner: RwLock::new(HashMap::new()) }
    }

    /// Returns the cached body if present and not expired.
    pub fn get(&self, key: &str) -> Option<String> {
        let map = self.inner.read().ok()?;
        let entry = map.get(key)?;
        if Instant::now() < entry.expires_at {
            Some(entry.body.clone())
        } else {
            None
        }
    }

    /// Stores a body with the given TTL.
    pub fn set(&self, key: &str, body: &str, ttl: Duration) {
        if let Ok(mut map) = self.inner.write() {
            map.insert(key.to_string(), Entry { body: body.to_string(), expires_at: Instant::now() + ttl });
        }
    }

    /// Removes all entries.
    pub fn clear(&self) {
        if let Ok(mut map) = self.inner.write() {
            map.clear();
        }
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.inner.read().map(|m| m.len()).unwrap_or(0)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

/// A shared in-flight outcome cell: the leader's success or failure, cloned by
/// followers once it resolves.
type CoalescedCell = Arc<OnceCell<Result<String, AppError>>>;

/// In-process single-flight for cache misses. Concurrent requests for the same
/// key share one in-flight upstream call and clone its outcome, so a burst on a
/// cold cache produces one game-server request per cache window instead of one
/// per requester.
#[derive(Default)]
pub struct Coalescer {
    inflight: Mutex<HashMap<String, CoalescedCell>>,
}

impl Coalescer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs `fetch` once per in-flight window for `key`; concurrent callers
    /// await the leader's outcome, success or failure. The slot is removed when
    /// the leader finishes so a later request fetches afresh rather than reusing
    /// a stale in-flight result.
    pub async fn run<F, Fut>(&self, key: &str, fetch: F) -> Result<String, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<String, AppError>>,
    {
        let cell = {
            let mut inflight = self.inflight.lock().unwrap();
            inflight
                .entry(key.to_string())
                .or_insert_with(|| Arc::new(OnceCell::new()))
                .clone()
        };
        let outcome = cell.get_or_init(|| async { fetch().await }).await.clone();
        if let Ok(mut inflight) = self.inflight.lock() {
            if let Some(slot) = inflight.get(key) {
                if Arc::ptr_eq(slot, &cell) {
                    inflight.remove(key);
                }
            }
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A burst of concurrent callers for one key must run the fetch once, all
    /// receive the same result, and the in-flight slot is cleaned up after.
    #[tokio::test]
    async fn coalesces_concurrent_callers_to_one_fetch() {
        let coalescer = Arc::new(Coalescer::new());
        let calls = Arc::new(AtomicU32::new(0));
        let mut handles = Vec::new();
        for _ in 0..16 {
            let coalescer = coalescer.clone();
            let calls = calls.clone();
            handles.push(tokio::spawn(async move {
                coalescer
                    .run("k", || async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        Ok("body".to_string())
                    })
                    .await
            }));
        }
        for handle in handles {
            assert_eq!(handle.await.unwrap().unwrap(), "body");
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1, "fetch must run once");
        assert!(
            coalescer.inflight.lock().unwrap().is_empty(),
            "in-flight slot cleaned up"
        );
    }

    /// A later request after the window completes fetches again; the slot is not
    /// a permanent cache, that is the TTL layer's job.
    #[tokio::test]
    async fn refetches_after_window_completes() {
        let coalescer = Coalescer::new();
        let calls = Arc::new(AtomicU32::new(0));
        for _ in 0..3 {
            let calls = calls.clone();
            let result = coalescer
                .run("k", || async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok("x".to_string())
                })
                .await;
            assert_eq!(result.unwrap(), "x");
        }
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }
}
