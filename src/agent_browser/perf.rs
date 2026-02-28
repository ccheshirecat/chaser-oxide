//! Performance Optimization module (Phase 27).
//!
//! Provides snapshot caching, lazy filtering, optimized serialization,
//! and benchmarking utilities for AI-focused performance.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::refs::RefMap;
use super::snapshot::{Snapshot, SnapshotOptions};

/// Snapshot cache with invalidation support.
#[derive(Debug)]
pub struct SnapshotCache {
    /// Cached snapshot.
    snapshot: Option<CachedSnapshot>,
    /// Cache TTL.
    ttl: Duration,
    /// Hit counter for statistics.
    hits: u64,
    /// Miss counter for statistics.
    misses: u64,
}

/// A cached snapshot with metadata.
#[derive(Debug, Clone)]
struct CachedSnapshot {
    /// The snapshot data.
    snapshot: Snapshot,
    /// When the snapshot was cached.
    cached_at: Instant,
    /// URL at time of caching.
    url: Option<String>,
    /// Options used for the snapshot.
    options: SnapshotOptions,
}

impl SnapshotCache {
    /// Create a new snapshot cache with the given TTL.
    pub fn new(ttl: Duration) -> Self {
        Self {
            snapshot: None,
            ttl,
            hits: 0,
            misses: 0,
        }
    }

    /// Create a cache with a default 5-second TTL.
    pub fn default_ttl() -> Self {
        Self::new(Duration::from_secs(5))
    }

    /// Get a cached snapshot if still valid.
    pub fn get(
        &mut self,
        options: &SnapshotOptions,
        current_url: Option<&str>,
    ) -> Option<&Snapshot> {
        if let Some(ref cached) = self.snapshot {
            // Check TTL
            if cached.cached_at.elapsed() > self.ttl {
                self.misses += 1;
                return None;
            }

            // Check URL hasn't changed
            if cached.url.as_deref() != current_url {
                self.misses += 1;
                return None;
            }

            // Check options match
            if cached.options != *options {
                self.misses += 1;
                return None;
            }

            self.hits += 1;
            Some(&cached.snapshot)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Store a snapshot in the cache.
    pub fn put(&mut self, snapshot: Snapshot, options: SnapshotOptions, url: Option<String>) {
        self.snapshot = Some(CachedSnapshot {
            snapshot,
            cached_at: Instant::now(),
            url,
            options,
        });
    }

    /// Invalidate the cache.
    pub fn invalidate(&mut self) {
        self.snapshot = None;
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            has_cached: self.snapshot.is_some(),
            ttl_ms: self.ttl.as_millis() as u64,
        }
    }

    /// Set the TTL.
    pub fn set_ttl(&mut self, ttl: Duration) {
        self.ttl = ttl;
    }
}

/// Cache statistics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheStats {
    /// Number of cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Whether a snapshot is currently cached.
    pub has_cached: bool,
    /// Cache TTL in milliseconds.
    pub ttl_ms: u64,
}

impl CacheStats {
    /// Calculate hit rate percentage.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Performance timer for benchmarking operations.
#[derive(Debug)]
pub struct PerfTimer {
    /// Operation name.
    name: String,
    /// Start time.
    start: Instant,
}

impl PerfTimer {
    /// Start a new timer.
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    /// Stop the timer and return the measurement.
    pub fn stop(self) -> PerfMeasurement {
        let duration_ms = self.start.elapsed().as_secs_f64() * 1000.0;
        PerfMeasurement {
            name: self.name,
            duration_ms,
        }
    }
}

/// A performance measurement result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerfMeasurement {
    /// Operation name.
    pub name: String,
    /// Duration in milliseconds.
    pub duration_ms: f64,
}

/// Performance tracker that collects measurements.
#[derive(Debug, Default)]
pub struct PerfTracker {
    /// Collected measurements.
    measurements: Vec<PerfMeasurement>,
    /// Running timers by name.
    active_timers: HashMap<String, Instant>,
}

impl PerfTracker {
    /// Create a new performance tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start timing an operation.
    pub fn start(&mut self, name: &str) {
        self.active_timers.insert(name.to_string(), Instant::now());
    }

    /// Stop timing and record the measurement.
    pub fn stop(&mut self, name: &str) -> Option<f64> {
        if let Some(start) = self.active_timers.remove(name) {
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            self.measurements.push(PerfMeasurement {
                name: name.to_string(),
                duration_ms,
            });
            Some(duration_ms)
        } else {
            None
        }
    }

    /// Record a pre-measured duration.
    pub fn record(&mut self, name: &str, duration_ms: f64) {
        self.measurements.push(PerfMeasurement {
            name: name.to_string(),
            duration_ms,
        });
    }

    /// Get all measurements.
    pub fn measurements(&self) -> &[PerfMeasurement] {
        &self.measurements
    }

    /// Get summary statistics.
    pub fn summary(&self) -> PerfSummary {
        let mut by_name: HashMap<String, Vec<f64>> = HashMap::new();
        for m in &self.measurements {
            by_name
                .entry(m.name.clone())
                .or_default()
                .push(m.duration_ms);
        }

        let operations = by_name
            .into_iter()
            .map(|(name, durations)| {
                let count = durations.len();
                let total: f64 = durations.iter().sum();
                let avg = total / count as f64;
                let min = durations.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = durations.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                OperationStats {
                    name,
                    count,
                    total_ms: total,
                    avg_ms: avg,
                    min_ms: min,
                    max_ms: max,
                }
            })
            .collect();

        PerfSummary { operations }
    }

    /// Clear all measurements.
    pub fn clear(&mut self) {
        self.measurements.clear();
        self.active_timers.clear();
    }
}

/// Summary of performance measurements grouped by operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerfSummary {
    /// Per-operation statistics.
    pub operations: Vec<OperationStats>,
}

/// Statistics for a single operation type.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperationStats {
    /// Operation name.
    pub name: String,
    /// Number of measurements.
    pub count: usize,
    /// Total time in ms.
    pub total_ms: f64,
    /// Average time in ms.
    pub avg_ms: f64,
    /// Minimum time in ms.
    pub min_ms: f64,
    /// Maximum time in ms.
    pub max_ms: f64,
}

/// Optimize a JSON response by removing null values and empty arrays.
pub fn compact_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let filtered: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter(|(_, v)| !v.is_null())
                .filter(|(_, v)| !matches!(v, serde_json::Value::Array(a) if a.is_empty()))
                .map(|(k, v)| (k.clone(), compact_json(v)))
                .collect();
            serde_json::Value::Object(filtered)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(compact_json).collect())
        }
        other => other.clone(),
    }
}

/// Estimate the memory usage of a RefMap in bytes.
pub fn estimate_refmap_memory(ref_map: &RefMap) -> usize {
    let mut total = std::mem::size_of::<RefMap>();

    for ref_id in ref_map.refs() {
        total += std::mem::size_of_val(ref_id);
        if let Some(info) = ref_map.get(ref_id) {
            total += std::mem::size_of_val(info);
            total += info.role.capacity();
            if let Some(ref name) = info.name {
                total += name.capacity();
            }
            if let Some(ref selector) = info.selector {
                total += selector.capacity();
            }
            if let Some(ref xpath) = info.xpath {
                total += xpath.capacity();
            }
            for (k, v) in &info.attributes {
                total += k.capacity() + v.capacity();
            }
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_cache_ttl() {
        let mut cache = SnapshotCache::new(Duration::from_millis(50));
        let opts = SnapshotOptions::default();

        // Cache miss (empty)
        assert!(cache.get(&opts, Some("https://example.com")).is_none());
        assert_eq!(cache.stats().misses, 1);

        // Put a snapshot
        let snapshot = Snapshot {
            tree: "test tree".to_string(),
            refs: RefMap::new(),
            root: None,
            total_nodes: 0,
            interactive_count: 0,
            url: Some("https://example.com".to_string()),
            title: None,
        };
        cache.put(
            snapshot,
            opts.clone(),
            Some("https://example.com".to_string()),
        );

        // Cache hit
        assert!(cache.get(&opts, Some("https://example.com")).is_some());
        assert_eq!(cache.stats().hits, 1);

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(60));
        assert!(cache.get(&opts, Some("https://example.com")).is_none());
        assert_eq!(cache.stats().misses, 2);
    }

    #[test]
    fn test_snapshot_cache_url_invalidation() {
        let mut cache = SnapshotCache::new(Duration::from_secs(60));
        let opts = SnapshotOptions::default();

        let snapshot = Snapshot {
            tree: "test".to_string(),
            refs: RefMap::new(),
            root: None,
            total_nodes: 0,
            interactive_count: 0,
            url: Some("https://example.com".to_string()),
            title: None,
        };
        cache.put(
            snapshot,
            opts.clone(),
            Some("https://example.com".to_string()),
        );

        // Different URL = miss
        assert!(cache.get(&opts, Some("https://other.com")).is_none());
    }

    #[test]
    fn test_snapshot_cache_invalidate() {
        let mut cache = SnapshotCache::new(Duration::from_secs(60));
        let opts = SnapshotOptions::default();

        let snapshot = Snapshot {
            tree: "test".to_string(),
            refs: RefMap::new(),
            root: None,
            total_nodes: 0,
            interactive_count: 0,
            url: None,
            title: None,
        };
        cache.put(snapshot, opts.clone(), None);
        assert!(cache.get(&opts, None).is_some());

        cache.invalidate();
        assert!(cache.get(&opts, None).is_none());
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let stats = CacheStats {
            hits: 75,
            misses: 25,
            has_cached: true,
            ttl_ms: 5000,
        };
        assert!((stats.hit_rate() - 75.0).abs() < 0.001);

        let empty = CacheStats {
            hits: 0,
            misses: 0,
            has_cached: false,
            ttl_ms: 5000,
        };
        assert!((empty.hit_rate() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::start("test_op");
        std::thread::sleep(Duration::from_millis(10));
        let measurement = timer.stop();
        assert_eq!(measurement.name, "test_op");
        assert!(measurement.duration_ms >= 5.0); // Allow some tolerance
    }

    #[test]
    fn test_perf_tracker() {
        let mut tracker = PerfTracker::new();

        tracker.record("snapshot", 15.5);
        tracker.record("snapshot", 20.0);
        tracker.record("click", 5.0);

        assert_eq!(tracker.measurements().len(), 3);

        let summary = tracker.summary();
        assert_eq!(summary.operations.len(), 2);

        let snapshot_stats = summary
            .operations
            .iter()
            .find(|o| o.name == "snapshot")
            .unwrap();
        assert_eq!(snapshot_stats.count, 2);
        assert!((snapshot_stats.avg_ms - 17.75).abs() < 0.001);
        assert!((snapshot_stats.min_ms - 15.5).abs() < 0.001);
        assert!((snapshot_stats.max_ms - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_perf_tracker_start_stop() {
        let mut tracker = PerfTracker::new();
        tracker.start("op");
        std::thread::sleep(Duration::from_millis(10));
        let duration = tracker.stop("op");
        assert!(duration.is_some());
        assert!(duration.unwrap() >= 5.0);
        assert_eq!(tracker.measurements().len(), 1);
    }

    #[test]
    fn test_compact_json() {
        let input = serde_json::json!({
            "name": "test",
            "value": null,
            "items": [],
            "nested": {
                "keep": "yes",
                "remove": null
            }
        });

        let compacted = compact_json(&input);
        assert!(compacted.get("name").is_some());
        assert!(compacted.get("value").is_none());
        assert!(compacted.get("items").is_none());
        let nested = compacted.get("nested").unwrap();
        assert!(nested.get("keep").is_some());
        assert!(nested.get("remove").is_none());
    }
}
