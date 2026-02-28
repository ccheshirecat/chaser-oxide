//! Streaming Server & Screencast (Phase 20).
//!
//! Provides CDP-based screencast and viewport streaming with optional
//! WebSocket server for real-time frame broadcasting and input injection.

use serde::{Deserialize, Serialize};

use super::commands::ScreencastOptions;

/// A single screencast frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreencastFrame {
    /// Base64-encoded image data.
    pub data: String,
    /// Frame metadata.
    pub metadata: FrameMetadata,
    /// Session ID for acknowledgment.
    pub session_id: i64,
}

/// Metadata for a screencast frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMetadata {
    /// X offset into the page.
    pub offset_top: f64,
    /// Page scale factor.
    pub page_scale_factor: f64,
    /// Device width.
    pub device_width: f64,
    /// Device height.
    pub device_height: f64,
    /// Scroll offset X.
    pub scroll_offset_x: f64,
    /// Scroll offset Y.
    pub scroll_offset_y: f64,
    /// Frame timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
}

/// WebSocket input message from a connected client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamInputMessage {
    /// Mouse event from client.
    #[serde(rename = "mouse")]
    Mouse {
        action: String,
        x: f64,
        y: f64,
        #[serde(default)]
        button: String,
    },
    /// Keyboard event from client.
    #[serde(rename = "keyboard")]
    Keyboard {
        action: String,
        key: String,
        #[serde(default)]
        code: String,
    },
    /// Touch event from client.
    #[serde(rename = "touch")]
    Touch {
        action: String,
        touches: Vec<TouchInfo>,
    },
}

/// Touch point info from a stream client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchInfo {
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub id: u32,
}

/// Stream server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Port for the WebSocket server.
    pub port: u16,
    /// Allowed origins for CORS.
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    /// Screencast options.
    #[serde(default)]
    pub screencast: ScreencastOptions,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            port: 9223,
            allowed_origins: vec!["*".to_string()],
            screencast: ScreencastOptions::default(),
        }
    }
}

impl StreamConfig {
    /// Create config from environment variable or use defaults.
    pub fn from_env() -> Self {
        let port = std::env::var("CHASER_STREAM_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(9223);

        Self {
            port,
            ..Default::default()
        }
    }
}

/// Screencast controller for managing CDP screencast sessions.
#[derive(Debug)]
pub struct ScreencastController {
    /// Whether the screencast is active.
    active: bool,
    /// Options for the screencast.
    options: ScreencastOptions,
    /// Frame counter.
    frame_count: u64,
    /// Collected frames (for non-streaming use).
    frames: Vec<ScreencastFrame>,
    /// Maximum frames to buffer (0 = unlimited).
    max_buffer: usize,
}

impl ScreencastController {
    /// Create a new screencast controller.
    pub fn new() -> Self {
        Self {
            active: false,
            options: ScreencastOptions::default(),
            frame_count: 0,
            frames: Vec::new(),
            max_buffer: 100,
        }
    }

    /// Start the screencast.
    pub fn start(&mut self, options: ScreencastOptions) {
        self.active = true;
        self.options = options;
        self.frame_count = 0;
        self.frames.clear();
    }

    /// Stop the screencast and return buffered frames.
    pub fn stop(&mut self) -> Vec<ScreencastFrame> {
        self.active = false;
        std::mem::take(&mut self.frames)
    }

    /// Whether the screencast is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Buffer a frame.
    pub fn push_frame(&mut self, frame: ScreencastFrame) {
        if !self.active {
            return;
        }
        self.frame_count += 1;

        if self.max_buffer > 0 && self.frames.len() >= self.max_buffer {
            self.frames.remove(0);
        }
        self.frames.push(frame);
    }

    /// Get the frame count since start.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get the current options.
    pub fn options(&self) -> &ScreencastOptions {
        &self.options
    }

    /// Set maximum buffer size.
    pub fn set_max_buffer(&mut self, max: usize) {
        self.max_buffer = max;
    }
}

impl Default for ScreencastController {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate CDP params JSON for starting a screencast.
pub fn screencast_start_params(options: &ScreencastOptions) -> serde_json::Value {
    let mut params = serde_json::json!({
        "quality": options.quality,
        "everyNthFrame": options.every_nth_frame,
    });

    let format_str = match options.format {
        super::commands::ScreenshotFormat::Jpeg => "jpeg",
        super::commands::ScreenshotFormat::Png => "png",
        _ => "jpeg",
    };
    params["format"] = serde_json::Value::String(format_str.to_string());

    if let Some(w) = options.max_width {
        params["maxWidth"] = serde_json::json!(w);
    }
    if let Some(h) = options.max_height {
        params["maxHeight"] = serde_json::json!(h);
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screencast_controller_lifecycle() {
        let mut ctrl = ScreencastController::new();
        assert!(!ctrl.is_active());

        ctrl.start(ScreencastOptions::default());
        assert!(ctrl.is_active());

        ctrl.push_frame(ScreencastFrame {
            data: "base64data".to_string(),
            metadata: FrameMetadata {
                offset_top: 0.0,
                page_scale_factor: 1.0,
                device_width: 1280.0,
                device_height: 720.0,
                scroll_offset_x: 0.0,
                scroll_offset_y: 0.0,
                timestamp: Some(1234567890.0),
            },
            session_id: 1,
        });

        assert_eq!(ctrl.frame_count(), 1);

        let frames = ctrl.stop();
        assert!(!ctrl.is_active());
        assert_eq!(frames.len(), 1);
    }

    #[test]
    fn test_screencast_buffer_limit() {
        let mut ctrl = ScreencastController::new();
        ctrl.set_max_buffer(2);
        ctrl.start(ScreencastOptions::default());

        for i in 0..5 {
            ctrl.push_frame(ScreencastFrame {
                data: format!("frame_{}", i),
                metadata: FrameMetadata {
                    offset_top: 0.0,
                    page_scale_factor: 1.0,
                    device_width: 1280.0,
                    device_height: 720.0,
                    scroll_offset_x: 0.0,
                    scroll_offset_y: 0.0,
                    timestamp: None,
                },
                session_id: i as i64,
            });
        }

        let frames = ctrl.stop();
        assert_eq!(frames.len(), 2);
        // Should have the last 2 frames
        assert_eq!(frames[0].data, "frame_3");
        assert_eq!(frames[1].data, "frame_4");
    }

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.port, 9223);
        assert_eq!(config.allowed_origins, vec!["*"]);
    }

    #[test]
    fn test_screencast_start_params() {
        let options = ScreencastOptions {
            quality: 80,
            every_nth_frame: 2,
            max_width: Some(1920),
            max_height: None,
            ..Default::default()
        };
        let params = screencast_start_params(&options);
        assert_eq!(params["quality"], 80);
        assert_eq!(params["everyNthFrame"], 2);
        assert_eq!(params["maxWidth"], 1920);
        assert!(params.get("maxHeight").is_none() || params["maxHeight"].is_null());
    }

    #[test]
    fn test_stream_input_message_deserialize() {
        let json = r#"{"type":"mouse","action":"click","x":100,"y":200,"button":"left"}"#;
        let msg: StreamInputMessage = serde_json::from_str(json).unwrap();
        match msg {
            StreamInputMessage::Mouse {
                action,
                x,
                y,
                button,
            } => {
                assert_eq!(action, "click");
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
                assert_eq!(button, "left");
            }
            _ => panic!("Expected Mouse variant"),
        }
    }
}
