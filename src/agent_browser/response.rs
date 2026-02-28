//! Agent-Browser response types.
//!
//! Provides unified response types that match the agent-browser JSON protocol.

// These types are part of the JSON protocol and may be constructed via deserialization.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type for agent operations.
pub type AgentResult<T> = std::result::Result<T, AgentError>;

/// Unified response type matching agent-browser's JSON protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum AgentResponse<T> {
    /// Successful response with data payload.
    #[serde(rename = "success")]
    Success {
        /// The response data.
        data: T,
    },
    /// Error response with message.
    #[serde(rename = "error")]
    Error {
        /// Error message describing what went wrong.
        message: String,
        /// Optional error code for programmatic handling.
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
}

impl<T> AgentResponse<T> {
    /// Create a success response.
    pub fn success(data: T) -> Self {
        Self::Success { data }
    }

    /// Create an error response.
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            code: None,
        }
    }

    /// Create an error response with a code.
    pub fn error_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            code: Some(code.into()),
        }
    }

    /// Check if this is a success response.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if this is an error response.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Convert to a Result.
    pub fn into_result(self) -> AgentResult<T> {
        match self {
            Self::Success { data } => Ok(data),
            Self::Error { message, code } => Err(AgentError::Command { message, code }),
        }
    }
}

impl<T> From<AgentResult<T>> for AgentResponse<T> {
    fn from(result: AgentResult<T>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(e) => Self::error(e.to_string()),
        }
    }
}

/// Error types for agent operations.
#[derive(Debug, Clone)]
pub enum AgentError {
    /// Element not found by selector or ref.
    ElementNotFound {
        /// The selector or ref that was not found.
        selector: String,
    },

    /// Invalid ref format (should be @e1, @e2, etc.).
    InvalidRef {
        /// The invalid ref string.
        ref_str: String,
    },

    /// Ref not found in the current snapshot.
    RefNotFound {
        /// The ref that was not found.
        ref_id: String,
    },

    /// No snapshot available (need to call snapshot() first).
    NoSnapshot,

    /// Timeout waiting for condition.
    Timeout {
        /// What we were waiting for.
        waiting_for: String,
        /// Timeout duration in milliseconds.
        timeout_ms: u64,
    },

    /// Navigation error.
    Navigation {
        /// Error message.
        message: String,
    },

    /// JavaScript evaluation error.
    JavaScript {
        /// Error message from JS execution.
        message: String,
    },

    /// Network error.
    Network {
        /// Error message.
        message: String,
    },

    /// Command execution error.
    Command {
        /// Error message.
        message: String,
        /// Optional error code.
        code: Option<String>,
    },

    /// Browser/page not available.
    NotConnected,

    /// Invalid command or parameters.
    InvalidCommand {
        /// Error message.
        message: String,
    },

    /// Frame not found.
    FrameNotFound {
        /// Frame identifier.
        frame: String,
    },

    /// Dialog handling error.
    Dialog {
        /// Error message.
        message: String,
    },

    /// Internal error (wraps underlying errors).
    Internal {
        /// Error message.
        message: String,
    },
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ElementNotFound { selector } => {
                write!(f, "Element not found: {}", selector)
            }
            Self::InvalidRef { ref_str } => {
                write!(
                    f,
                    "Invalid ref format '{}' (expected @e1, @e2, etc.)",
                    ref_str
                )
            }
            Self::RefNotFound { ref_id } => {
                write!(f, "Ref '{}' not found in current snapshot", ref_id)
            }
            Self::NoSnapshot => {
                write!(f, "No snapshot available. Call snapshot() first.")
            }
            Self::Timeout {
                waiting_for,
                timeout_ms,
            } => {
                write!(
                    f,
                    "Timeout after {}ms waiting for: {}",
                    timeout_ms, waiting_for
                )
            }
            Self::Navigation { message } => {
                write!(f, "Navigation error: {}", message)
            }
            Self::JavaScript { message } => {
                write!(f, "JavaScript error: {}", message)
            }
            Self::Network { message } => {
                write!(f, "Network error: {}", message)
            }
            Self::Command { message, code } => {
                if let Some(code) = code {
                    write!(f, "Command error [{}]: {}", code, message)
                } else {
                    write!(f, "Command error: {}", message)
                }
            }
            Self::NotConnected => {
                write!(f, "Browser or page not connected")
            }
            Self::InvalidCommand { message } => {
                write!(f, "Invalid command: {}", message)
            }
            Self::FrameNotFound { frame } => {
                write!(f, "Frame not found: {}", frame)
            }
            Self::Dialog { message } => {
                write!(f, "Dialog error: {}", message)
            }
            Self::Internal { message } => {
                write!(f, "Internal error: {}", message)
            }
        }
    }
}

impl std::error::Error for AgentError {}

impl From<crate::error::CdpError> for AgentError {
    fn from(err: crate::error::CdpError) -> Self {
        Self::Internal {
            message: err.to_string(),
        }
    }
}

/// Navigate command response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigateData {
    /// The URL after navigation.
    pub url: String,
    /// The page title after navigation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Screenshot response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScreenshotData {
    /// Screenshot saved to file.
    Path {
        /// Path to the saved file.
        path: String,
    },
    /// Screenshot as base64 data.
    Base64 {
        /// Base64-encoded image data.
        data: String,
    },
}

/// Element count response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountData {
    /// Number of matching elements.
    pub count: usize,
}

/// Bounding box response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBoxData {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

/// Tab information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    /// Tab index.
    pub index: usize,
    /// Tab URL.
    pub url: String,
    /// Tab title.
    pub title: String,
    /// Whether this tab is active.
    pub active: bool,
}

/// Tab list response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabListData {
    /// List of tabs.
    pub tabs: Vec<TabInfo>,
    /// Index of the active tab.
    pub active_index: usize,
}

/// Console message data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    /// Message type (log, warn, error, etc.).
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Message text.
    pub text: String,
    /// Timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
}

/// Console response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleData {
    /// Console messages.
    pub messages: Vec<ConsoleMessage>,
}

/// Page error data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageError {
    /// Error message.
    pub message: String,
    /// Source URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Line number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Column number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

/// Errors response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorsData {
    /// Page errors.
    pub errors: Vec<PageError>,
}

/// Network request info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInfo {
    /// Request ID.
    pub id: String,
    /// HTTP method.
    pub method: String,
    /// Request URL.
    pub url: String,
    /// Resource type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    /// Response status code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u32>,
}

/// Requests response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestsData {
    /// Tracked requests.
    pub requests: Vec<RequestInfo>,
}

/// Computed styles response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylesData {
    /// Bounding box.
    pub box_model: BoundingBoxData,
    /// Computed styles (subset).
    pub styles: std::collections::HashMap<String, String>,
}
