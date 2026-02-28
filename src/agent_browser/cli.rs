//! CLI Interface module (Phase 25).
//!
//! Provides command-line parsing, JSON protocol handling, and daemon mode
//! for the chaser-agent binary. This module defines the CLI command structures
//! and protocol format used by AI agents to communicate with the browser.

use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

use super::response::{AgentError, AgentResult};

/// Top-level CLI command parsed from JSON or CLI arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum CliCommand {
    // Navigation
    Navigate {
        url: String,
    },
    Back,
    Forward,
    Reload,
    Close,

    // Snapshot
    Snapshot {
        #[serde(default)]
        interactive_only: bool,
        #[serde(default)]
        compact: bool,
        #[serde(default)]
        max_depth: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        scope: Option<String>,
    },

    // Element Actions
    Click {
        selector: String,
    },
    Dblclick {
        selector: String,
    },
    Hover {
        selector: String,
    },
    Focus {
        selector: String,
    },
    Type {
        selector: String,
        text: String,
        #[serde(default)]
        clear: bool,
    },
    Fill {
        selector: String,
        value: String,
    },
    Clear {
        selector: String,
    },
    Check {
        selector: String,
    },
    Uncheck {
        selector: String,
    },
    Select {
        selector: String,
        value: String,
    },

    // Keyboard & Mouse
    Press {
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        selector: Option<String>,
    },
    Keyboard {
        shortcut: String,
    },
    Scroll {
        direction: String,
        #[serde(default = "default_scroll_amount")]
        amount: i32,
    },

    // Information
    GetText {
        selector: String,
    },
    GetHtml {
        #[serde(skip_serializing_if = "Option::is_none")]
        selector: Option<String>,
    },
    GetValue {
        selector: String,
    },
    GetAttribute {
        selector: String,
        attribute: String,
    },
    GetUrl,
    GetTitle,
    GetCount {
        selector: String,
    },

    // State
    IsVisible {
        selector: String,
    },
    IsEnabled {
        selector: String,
    },
    IsChecked {
        selector: String,
    },

    // Wait
    Wait {
        selector: String,
        #[serde(default)]
        state: String,
        #[serde(default = "default_timeout")]
        timeout: u64,
    },
    WaitForUrl {
        pattern: String,
        #[serde(default = "default_timeout")]
        timeout: u64,
    },

    // JavaScript
    Evaluate {
        expression: String,
    },

    // Screenshots & PDFs
    Screenshot {
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(default)]
        full_page: bool,
    },
    Pdf {
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },

    // Tabs
    TabNew {
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },
    TabList,
    TabSwitch {
        index: usize,
    },
    TabClose {
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<usize>,
    },

    // Settings
    SetViewport {
        width: u32,
        height: u32,
    },
    SetDevice {
        name: String,
    },
    SetGeolocation {
        latitude: f64,
        longitude: f64,
    },
    SetOffline {
        offline: bool,
    },

    // Cookies & Storage
    CookiesGet,
    CookiesClear,
    StorageGet {
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        #[serde(default)]
        storage_type: String,
    },
    StorageSet {
        key: String,
        value: String,
        #[serde(default)]
        storage_type: String,
    },
    StorageClear,

    // Clipboard
    Clipboard {
        action: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },

    // Console & Errors
    Console {
        #[serde(default)]
        clear: bool,
    },
    Errors {
        #[serde(default)]
        clear: bool,
    },

    // Recording
    TraceStart {
        #[serde(default)]
        screenshots: bool,
        #[serde(default)]
        snapshots: bool,
    },
    TraceStop {
        path: String,
    },
    HarStart,
    HarStop {
        path: String,
    },

    // Session
    StateSave {
        path: String,
    },
    StateLoad {
        path: String,
    },

    // Meta
    ListDevices,
    Version,
    Status,
}

fn default_scroll_amount() -> i32 {
    300
}

fn default_timeout() -> u64 {
    30000
}

/// CLI response wrapper for JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResponse {
    /// Whether the command succeeded.
    pub success: bool,
    /// Response data (for successful commands).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Error message (for failed commands).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Error code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl CliResponse {
    /// Create a success response.
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            code: None,
        }
    }

    /// Create a success response with no data.
    pub fn ok() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
            code: None,
        }
    }

    /// Create an error response.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            code: None,
        }
    }

    /// Create an error response with a code.
    pub fn error_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            code: Some(code.into()),
        }
    }

    /// Serialize to JSON string (line-delimited format).
    pub fn to_json_line(&self) -> AgentResult<String> {
        serde_json::to_string(self).map_err(|e| AgentError::Internal {
            message: format!("Failed to serialize response: {}", e),
        })
    }
}

impl From<AgentError> for CliResponse {
    fn from(err: AgentError) -> Self {
        Self::error(err.to_string())
    }
}

/// Parse a CLI command from a JSON string.
pub fn parse_command(json: &str) -> AgentResult<CliCommand> {
    serde_json::from_str(json).map_err(|e| AgentError::InvalidCommand {
        message: format!("Failed to parse command: {}", e),
    })
}

/// Read commands from stdin (line-delimited JSON protocol).
pub struct StdinReader {
    #[allow(missing_debug_implementations)]
    reader: io::BufReader<io::Stdin>,
}

impl std::fmt::Debug for StdinReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdinReader").finish()
    }
}

impl StdinReader {
    /// Create a new stdin reader.
    pub fn new() -> Self {
        Self {
            reader: io::BufReader::new(io::stdin()),
        }
    }

    /// Read the next command from stdin.
    pub fn next_command(&mut self) -> Option<AgentResult<CliCommand>> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return self.next_command();
                }
                Some(parse_command(trimmed))
            }
            Err(e) => Some(Err(AgentError::Internal {
                message: format!("Failed to read stdin: {}", e),
            })),
        }
    }
}

impl Default for StdinReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Write a response to stdout (line-delimited JSON).
pub fn write_response(response: &CliResponse) -> AgentResult<()> {
    let json = response.to_json_line()?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "{}", json).map_err(|e| AgentError::Internal {
        message: format!("Failed to write response: {}", e),
    })
}

/// Version information for the CLI.
pub fn version_info() -> serde_json::Value {
    serde_json::json!({
        "name": "chaser-oxide",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "agent-browser-compatible",
        "features": [
            "snapshot-refs",
            "semantic-locators",
            "stealth-mode",
            "screencast",
            "recording",
            "session-management"
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_navigate_command() {
        let json = r#"{"command": "navigate", "url": "https://example.com"}"#;
        let cmd = parse_command(json).unwrap();
        match cmd {
            CliCommand::Navigate { url } => assert_eq!(url, "https://example.com"),
            _ => panic!("Expected Navigate"),
        }
    }

    #[test]
    fn test_parse_click_command() {
        let json = r#"{"command": "click", "selector": "@e1"}"#;
        let cmd = parse_command(json).unwrap();
        match cmd {
            CliCommand::Click { selector } => assert_eq!(selector, "@e1"),
            _ => panic!("Expected Click"),
        }
    }

    #[test]
    fn test_parse_snapshot_command() {
        let json = r#"{"command": "snapshot", "interactive_only": true, "max_depth": 5}"#;
        let cmd = parse_command(json).unwrap();
        match cmd {
            CliCommand::Snapshot {
                interactive_only,
                max_depth,
                ..
            } => {
                assert!(interactive_only);
                assert_eq!(max_depth, 5);
            }
            _ => panic!("Expected Snapshot"),
        }
    }

    #[test]
    fn test_parse_tab_commands() {
        let json = r#"{"command": "tab_new", "url": "https://example.com"}"#;
        let cmd = parse_command(json).unwrap();
        match cmd {
            CliCommand::TabNew { url } => assert_eq!(url, Some("https://example.com".to_string())),
            _ => panic!("Expected TabNew"),
        }

        let json = r#"{"command": "tab_list"}"#;
        let cmd = parse_command(json).unwrap();
        assert!(matches!(cmd, CliCommand::TabList));

        let json = r#"{"command": "tab_switch", "index": 2}"#;
        let cmd = parse_command(json).unwrap();
        match cmd {
            CliCommand::TabSwitch { index } => assert_eq!(index, 2),
            _ => panic!("Expected TabSwitch"),
        }
    }

    #[test]
    fn test_cli_response_success() {
        let resp = CliResponse::success(serde_json::json!({"url": "https://example.com"}));
        assert!(resp.success);
        let json = resp.to_json_line().unwrap();
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_cli_response_error() {
        let resp = CliResponse::error("something went wrong");
        assert!(!resp.success);
        assert_eq!(resp.error.as_deref(), Some("something went wrong"));
    }

    #[test]
    fn test_cli_response_ok() {
        let resp = CliResponse::ok();
        assert!(resp.success);
        assert!(resp.data.is_none());
    }

    #[test]
    fn test_parse_invalid_command() {
        let result = parse_command("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_info() {
        let info = version_info();
        assert_eq!(info["name"], "chaser-oxide");
        assert!(info["features"].is_array());
    }

    #[test]
    fn test_parse_evaluate_command() {
        let json = r#"{"command": "evaluate", "expression": "document.title"}"#;
        let cmd = parse_command(json).unwrap();
        match cmd {
            CliCommand::Evaluate { expression } => assert_eq!(expression, "document.title"),
            _ => panic!("Expected Evaluate"),
        }
    }

    #[test]
    fn test_parse_trace_commands() {
        let json = r#"{"command": "trace_start", "screenshots": true, "snapshots": false}"#;
        let cmd = parse_command(json).unwrap();
        match cmd {
            CliCommand::TraceStart {
                screenshots,
                snapshots,
            } => {
                assert!(screenshots);
                assert!(!snapshots);
            }
            _ => panic!("Expected TraceStart"),
        }

        let json = r#"{"command": "har_start"}"#;
        let cmd = parse_command(json).unwrap();
        assert!(matches!(cmd, CliCommand::HarStart));
    }

    #[test]
    fn test_parse_clipboard_command() {
        let json = r#"{"command": "clipboard", "action": "copy"}"#;
        let cmd = parse_command(json).unwrap();
        match cmd {
            CliCommand::Clipboard { action, text } => {
                assert_eq!(action, "copy");
                assert!(text.is_none());
            }
            _ => panic!("Expected Clipboard"),
        }
    }
}
