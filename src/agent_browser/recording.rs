//! Recording & Tracing module (Phase 17).
//!
//! Provides trace recording (CDP Tracing domain), HAR capture,
//! and screencast-based video recording capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::commands::TraceOptions;
use super::response::{AgentError, AgentResult};

/// A single trace entry captured during tracing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    /// Command or action name.
    pub action: String,
    /// Timestamp (ms since epoch).
    pub timestamp: f64,
    /// Duration in ms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<f64>,
    /// Optional screenshot (base64).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<String>,
    /// Optional snapshot tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<String>,
    /// Optional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// HAR entry representing a single HTTP request/response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarEntry {
    /// Request URL.
    pub url: String,
    /// HTTP method.
    pub method: String,
    /// Response status code.
    pub status: u16,
    /// Response status text.
    pub status_text: String,
    /// Request headers.
    pub request_headers: Vec<HarHeader>,
    /// Response headers.
    pub response_headers: Vec<HarHeader>,
    /// Response content size in bytes.
    pub response_size: i64,
    /// Response MIME type.
    pub mime_type: String,
    /// Time taken in ms.
    pub time: f64,
    /// Start time (ISO 8601).
    pub started_date_time: String,
    /// Optional response body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_body: Option<String>,
}

/// A single HTTP header name-value pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarHeader {
    pub name: String,
    pub value: String,
}

/// HAR log container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarLog {
    /// HAR version.
    pub version: String,
    /// Creator info.
    pub creator: HarCreator,
    /// Captured entries.
    pub entries: Vec<HarEntry>,
}

/// HAR creator info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarCreator {
    pub name: String,
    pub version: String,
}

impl Default for HarLog {
    fn default() -> Self {
        Self {
            version: "1.2".to_string(),
            creator: HarCreator {
                name: "chaser-oxide".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            entries: Vec::new(),
        }
    }
}

/// Trace recorder that captures actions, screenshots, and snapshots.
#[derive(Debug)]
pub struct TraceRecorder {
    /// Whether tracing is active.
    active: bool,
    /// Trace options.
    options: TraceOptions,
    /// Recorded entries.
    entries: Vec<TraceEntry>,
    /// Start time.
    start_time: Option<std::time::Instant>,
}

impl TraceRecorder {
    /// Create a new inactive trace recorder.
    pub fn new() -> Self {
        Self {
            active: false,
            options: TraceOptions::default(),
            entries: Vec::new(),
            start_time: None,
        }
    }

    /// Start tracing.
    pub fn start(&mut self, options: TraceOptions) {
        self.active = true;
        self.options = options;
        self.entries.clear();
        self.start_time = Some(std::time::Instant::now());
    }

    /// Stop tracing and return entries.
    pub fn stop(&mut self) -> Vec<TraceEntry> {
        self.active = false;
        self.start_time = None;
        std::mem::take(&mut self.entries)
    }

    /// Check if tracing is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Record a trace entry.
    pub fn record(&mut self, action: &str) {
        if !self.active {
            return;
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

        self.entries.push(TraceEntry {
            action: action.to_string(),
            timestamp,
            duration_ms: None,
            screenshot: None,
            snapshot: None,
            metadata: HashMap::new(),
        });
    }

    /// Record with metadata.
    pub fn record_with_metadata(
        &mut self,
        action: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) {
        if !self.active {
            return;
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

        self.entries.push(TraceEntry {
            action: action.to_string(),
            timestamp,
            duration_ms: None,
            screenshot: None,
            snapshot: None,
            metadata,
        });
    }

    /// Get the trace options.
    pub fn options(&self) -> &TraceOptions {
        &self.options
    }

    /// Get current entries count.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Serialize entries to JSON.
    pub fn to_json(&self) -> AgentResult<String> {
        serde_json::to_string_pretty(&self.entries).map_err(|e| AgentError::Internal {
            message: format!("Failed to serialize trace: {}", e),
        })
    }
}

impl Default for TraceRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// HAR recorder that captures network requests.
#[derive(Debug)]
pub struct HarRecorder {
    /// Whether recording is active.
    active: bool,
    /// The HAR log being built.
    log: HarLog,
    /// Whether to capture response content.
    capture_content: bool,
}

impl HarRecorder {
    /// Create a new inactive HAR recorder.
    pub fn new() -> Self {
        Self {
            active: false,
            log: HarLog::default(),
            capture_content: false,
        }
    }

    /// Start HAR recording.
    pub fn start(&mut self, capture_content: bool) {
        self.active = true;
        self.capture_content = capture_content;
        self.log = HarLog::default();
    }

    /// Stop recording and return the HAR log.
    pub fn stop(&mut self) -> HarLog {
        self.active = false;
        std::mem::take(&mut self.log)
    }

    /// Check if recording is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Record a network request entry.
    pub fn record_entry(&mut self, entry: HarEntry) {
        if self.active {
            self.log.entries.push(entry);
        }
    }

    /// Get current entry count.
    pub fn entry_count(&self) -> usize {
        self.log.entries.len()
    }

    /// Serialize the HAR log to JSON.
    pub fn to_json(&self) -> AgentResult<String> {
        let har = serde_json::json!({
            "log": self.log
        });
        serde_json::to_string_pretty(&har).map_err(|e| AgentError::Internal {
            message: format!("Failed to serialize HAR: {}", e),
        })
    }

    /// Save the HAR log to a file.
    pub fn save_to_file(&self, path: &str) -> AgentResult<()> {
        let json = self.to_json()?;
        std::fs::write(path, json).map_err(|e| AgentError::Internal {
            message: format!("Failed to write HAR file: {}", e),
        })
    }
}

impl Default for HarRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_recorder_lifecycle() {
        let mut recorder = TraceRecorder::new();
        assert!(!recorder.is_active());

        recorder.start(TraceOptions {
            screenshots: true,
            snapshots: false,
        });
        assert!(recorder.is_active());

        recorder.record("navigate");
        recorder.record("click");
        assert_eq!(recorder.entry_count(), 2);

        let entries = recorder.stop();
        assert!(!recorder.is_active());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].action, "navigate");
        assert_eq!(entries[1].action, "click");
    }

    #[test]
    fn test_trace_recorder_inactive_no_record() {
        let mut recorder = TraceRecorder::new();
        recorder.record("should_not_record");
        assert_eq!(recorder.entry_count(), 0);
    }

    #[test]
    fn test_har_recorder_lifecycle() {
        let mut recorder = HarRecorder::new();
        assert!(!recorder.is_active());

        recorder.start(true);
        assert!(recorder.is_active());

        recorder.record_entry(HarEntry {
            url: "https://example.com".to_string(),
            method: "GET".to_string(),
            status: 200,
            status_text: "OK".to_string(),
            request_headers: vec![],
            response_headers: vec![],
            response_size: 1234,
            mime_type: "text/html".to_string(),
            time: 150.0,
            started_date_time: "2026-01-01T00:00:00Z".to_string(),
            response_body: Some("<html></html>".to_string()),
        });

        assert_eq!(recorder.entry_count(), 1);

        let log = recorder.stop();
        assert!(!recorder.is_active());
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.version, "1.2");
        assert_eq!(log.creator.name, "chaser-oxide");
    }

    #[test]
    fn test_har_log_default() {
        let log = HarLog::default();
        assert_eq!(log.version, "1.2");
        assert!(log.entries.is_empty());
    }

    #[test]
    fn test_har_recorder_to_json() {
        let recorder = HarRecorder::new();
        let json = recorder.to_json().unwrap();
        assert!(json.contains("\"log\""));
        assert!(json.contains("\"version\""));
    }

    #[test]
    fn test_trace_recorder_with_metadata() {
        let mut recorder = TraceRecorder::new();
        recorder.start(TraceOptions::default());

        let mut meta = HashMap::new();
        meta.insert("url".to_string(), serde_json::json!("https://example.com"));
        recorder.record_with_metadata("navigate", meta);

        assert_eq!(recorder.entry_count(), 1);
        let entries = recorder.stop();
        assert!(entries[0].metadata.contains_key("url"));
    }
}
