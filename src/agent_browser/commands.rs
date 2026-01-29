//! Command types for the agent-browser protocol.
//!
//! These types match the agent-browser CLI command structure for compatibility.

use serde::{Deserialize, Serialize};

/// Mouse button type.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    #[default]
    Left,
    Right,
    Middle,
}

impl MouseButton {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Middle => "middle",
        }
    }
}

/// Keyboard modifier keys.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub meta: bool,
}

impl KeyModifiers {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn ctrl() -> Self {
        Self { ctrl: true, ..Default::default() }
    }

    pub fn alt() -> Self {
        Self { alt: true, ..Default::default() }
    }

    pub fn shift() -> Self {
        Self { shift: true, ..Default::default() }
    }

    pub fn meta() -> Self {
        Self { meta: true, ..Default::default() }
    }

    /// Convert to CDP modifier flags.
    pub fn to_cdp_flags(&self) -> i32 {
        let mut flags = 0;
        if self.alt {
            flags |= 1;
        }
        if self.ctrl {
            flags |= 2;
        }
        if self.meta {
            flags |= 4;
        }
        if self.shift {
            flags |= 8;
        }
        flags
    }
}

/// Wait states for page loading.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WaitUntil {
    /// Wait for the load event.
    #[default]
    Load,
    /// Wait for DOMContentLoaded.
    #[serde(rename = "domcontentloaded")]
    DomContentLoaded,
    /// Wait until there are no network connections for 500ms.
    #[serde(rename = "networkidle")]
    NetworkIdle,
}

/// Screenshot format.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScreenshotFormat {
    #[default]
    Png,
    Jpeg,
    Webp,
}

impl ScreenshotFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Webp => "webp",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }
}

/// PDF page format.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum PdfFormat {
    #[default]
    Letter,
    Legal,
    Tabloid,
    Ledger,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
}

impl PdfFormat {
    /// Get dimensions in inches (width, height).
    pub fn dimensions(&self) -> (f64, f64) {
        match self {
            Self::Letter => (8.5, 11.0),
            Self::Legal => (8.5, 14.0),
            Self::Tabloid => (11.0, 17.0),
            Self::Ledger => (17.0, 11.0),
            Self::A0 => (33.1, 46.8),
            Self::A1 => (23.4, 33.1),
            Self::A2 => (16.5, 23.4),
            Self::A3 => (11.7, 16.5),
            Self::A4 => (8.27, 11.7),
            Self::A5 => (5.83, 8.27),
            Self::A6 => (4.13, 5.83),
        }
    }
}

/// Scroll direction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ScrollDirection {
    /// Get delta values for this direction.
    pub fn to_deltas(&self, amount: i32) -> (i32, i32) {
        match self {
            Self::Up => (0, -amount),
            Self::Down => (0, amount),
            Self::Left => (-amount, 0),
            Self::Right => (amount, 0),
        }
    }
}

/// Element state for waiting.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ElementState {
    /// Wait for element to be attached to DOM.
    Attached,
    /// Wait for element to be detached from DOM.
    Detached,
    /// Wait for element to be visible.
    #[default]
    Visible,
    /// Wait for element to be hidden.
    Hidden,
}

/// Storage type.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    #[default]
    Local,
    Session,
}

impl StorageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "localStorage",
            Self::Session => "sessionStorage",
        }
    }
}

/// Color scheme for emulation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    Light,
    Dark,
    NoPreference,
}

/// Reduced motion preference.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReducedMotion {
    Reduce,
    NoPreference,
}

/// Forced colors preference.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ForcedColors {
    Active,
    None,
}

/// Media type for emulation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Screen,
    Print,
}

/// Dialog action.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DialogAction {
    Accept,
    Dismiss,
}

/// Click options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClickOptions {
    /// Mouse button to use.
    #[serde(default)]
    pub button: MouseButton,

    /// Number of clicks (1 = single, 2 = double).
    #[serde(default = "default_click_count")]
    pub click_count: u32,

    /// Delay between mousedown and mouseup in milliseconds.
    #[serde(default)]
    pub delay: u64,

    /// Whether to use human-like movement.
    #[serde(default)]
    pub human_like: bool,

    /// Keyboard modifiers.
    #[serde(default)]
    pub modifiers: KeyModifiers,
}

fn default_click_count() -> u32 {
    1
}

/// Type/input options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeOptions {
    /// Clear the field before typing.
    #[serde(default)]
    pub clear: bool,

    /// Delay between keystrokes in milliseconds.
    #[serde(default)]
    pub delay: u64,

    /// Use human-like typing with variable delays.
    #[serde(default)]
    pub human_like: bool,

    /// Include occasional typos and corrections.
    #[serde(default)]
    pub with_typos: bool,
}

/// Screenshot options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScreenshotOptions {
    /// File path to save (if not provided, returns base64).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// CSS selector to screenshot (if not provided, captures viewport/full page).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,

    /// Capture full page.
    #[serde(default)]
    pub full_page: bool,

    /// Image format.
    #[serde(default)]
    pub format: ScreenshotFormat,

    /// Quality (0-100) for JPEG/WebP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
}

/// Wait options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WaitOptions {
    /// Element state to wait for.
    #[serde(default)]
    pub state: ElementState,

    /// Timeout in milliseconds.
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_timeout() -> u64 {
    30000
}

/// Viewport dimensions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ViewportSize {
    pub width: u32,
    pub height: u32,
}

impl Default for ViewportSize {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
        }
    }
}

/// Geolocation coordinates.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Geolocation {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default = "default_accuracy")]
    pub accuracy: f64,
}

fn default_accuracy() -> f64 {
    1.0
}

/// Cookie data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<f64>,
    #[serde(default)]
    pub http_only: bool,
    #[serde(default)]
    pub secure: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub same_site: Option<String>,
}

/// Route response for mocking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum RouteResponse {
    /// Abort the request.
    #[serde(rename = "abort")]
    Abort,

    /// Fulfill with a mock response.
    #[serde(rename = "fulfill")]
    Fulfill {
        /// Response body.
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<String>,

        /// HTTP status code.
        #[serde(default = "default_status")]
        status: u16,

        /// Response headers.
        #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
        headers: std::collections::HashMap<String, String>,

        /// Content type.
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },

    /// Continue with the original request.
    #[serde(rename = "continue")]
    Continue,
}

fn default_status() -> u16 {
    200
}

/// Trace options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceOptions {
    /// Include screenshots.
    #[serde(default)]
    pub screenshots: bool,

    /// Include snapshots.
    #[serde(default)]
    pub snapshots: bool,
}

/// HAR (HTTP Archive) options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HarOptions {
    /// Track request content.
    #[serde(default)]
    pub content: bool,
}

/// Screencast options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScreencastOptions {
    /// Image format (jpeg or png).
    #[serde(default)]
    pub format: ScreenshotFormat,

    /// Quality (0-100).
    #[serde(default = "default_quality")]
    pub quality: u8,

    /// Maximum width.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<u32>,

    /// Maximum height.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<u32>,

    /// Capture every Nth frame.
    #[serde(default = "default_every_nth")]
    pub every_nth_frame: u32,
}

fn default_quality() -> u8 {
    80
}

fn default_every_nth() -> u32 {
    1
}

/// Raw mouse input event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMouseInput {
    /// Event type: mousePressed, mouseReleased, mouseMoved, mouseWheel.
    #[serde(rename = "type")]
    pub event_type: String,

    /// X coordinate.
    pub x: f64,

    /// Y coordinate.
    pub y: f64,

    /// Mouse button.
    #[serde(default)]
    pub button: MouseButton,

    /// Click count.
    #[serde(default = "default_click_count")]
    pub click_count: u32,

    /// Wheel delta X.
    #[serde(default)]
    pub delta_x: f64,

    /// Wheel delta Y.
    #[serde(default)]
    pub delta_y: f64,

    /// Modifier keys.
    #[serde(default)]
    pub modifiers: KeyModifiers,
}

/// Raw keyboard input event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawKeyboardInput {
    /// Event type: keyDown, keyUp, char.
    #[serde(rename = "type")]
    pub event_type: String,

    /// Key identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Key code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Text to insert.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Modifier keys.
    #[serde(default)]
    pub modifiers: KeyModifiers,
}

/// Touch point for touch events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchPoint {
    /// X coordinate.
    pub x: f64,

    /// Y coordinate.
    pub y: f64,

    /// Unique identifier.
    #[serde(default)]
    pub id: u32,
}

/// Raw touch input event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTouchInput {
    /// Event type: touchStart, touchMove, touchEnd, touchCancel.
    #[serde(rename = "type")]
    pub event_type: String,

    /// Touch points.
    pub touch_points: Vec<TouchPoint>,

    /// Modifier keys.
    #[serde(default)]
    pub modifiers: KeyModifiers,
}

/// Device descriptor for emulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDescriptor {
    /// Device name.
    pub name: String,

    /// Viewport dimensions.
    pub viewport: ViewportSize,

    /// Device scale factor.
    #[serde(default = "default_scale")]
    pub device_scale_factor: f64,

    /// Is mobile device.
    #[serde(default)]
    pub is_mobile: bool,

    /// Has touch.
    #[serde(default)]
    pub has_touch: bool,

    /// User agent string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

fn default_scale() -> f64 {
    1.0
}

/// Predefined devices.
pub mod devices {
    use super::{DeviceDescriptor, ViewportSize};

    pub fn iphone_14() -> DeviceDescriptor {
        DeviceDescriptor {
            name: "iPhone 14".to_string(),
            viewport: ViewportSize {
                width: 390,
                height: 844,
            },
            device_scale_factor: 3.0,
            is_mobile: true,
            has_touch: true,
            user_agent: Some("Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1".to_string()),
        }
    }

    pub fn iphone_14_pro_max() -> DeviceDescriptor {
        DeviceDescriptor {
            name: "iPhone 14 Pro Max".to_string(),
            viewport: ViewportSize {
                width: 430,
                height: 932,
            },
            device_scale_factor: 3.0,
            is_mobile: true,
            has_touch: true,
            user_agent: Some("Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1".to_string()),
        }
    }

    pub fn pixel_7() -> DeviceDescriptor {
        DeviceDescriptor {
            name: "Pixel 7".to_string(),
            viewport: ViewportSize {
                width: 412,
                height: 915,
            },
            device_scale_factor: 2.625,
            is_mobile: true,
            has_touch: true,
            user_agent: Some("Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Mobile Safari/537.36".to_string()),
        }
    }

    pub fn ipad_pro_11() -> DeviceDescriptor {
        DeviceDescriptor {
            name: "iPad Pro 11".to_string(),
            viewport: ViewportSize {
                width: 834,
                height: 1194,
            },
            device_scale_factor: 2.0,
            is_mobile: true,
            has_touch: true,
            user_agent: Some("Mozilla/5.0 (iPad; CPU OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1".to_string()),
        }
    }

    pub fn desktop_chrome() -> DeviceDescriptor {
        DeviceDescriptor {
            name: "Desktop Chrome".to_string(),
            viewport: ViewportSize {
                width: 1280,
                height: 720,
            },
            device_scale_factor: 1.0,
            is_mobile: false,
            has_touch: false,
            user_agent: None,
        }
    }

    pub fn desktop_firefox() -> DeviceDescriptor {
        DeviceDescriptor {
            name: "Desktop Firefox".to_string(),
            viewport: ViewportSize {
                width: 1280,
                height: 720,
            },
            device_scale_factor: 1.0,
            is_mobile: false,
            has_touch: false,
            user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/119.0".to_string()),
        }
    }

    /// Get a device by name.
    pub fn get_device(name: &str) -> Option<DeviceDescriptor> {
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            "iphone 14" | "iphone14" => Some(iphone_14()),
            "iphone 14 pro max" | "iphone14promax" => Some(iphone_14_pro_max()),
            "pixel 7" | "pixel7" => Some(pixel_7()),
            "ipad pro 11" | "ipadpro11" => Some(ipad_pro_11()),
            "desktop chrome" | "chrome" => Some(desktop_chrome()),
            "desktop firefox" | "firefox" => Some(desktop_firefox()),
            _ => None,
        }
    }

    /// List all available device names.
    pub fn list_devices() -> Vec<&'static str> {
        vec![
            "iPhone 14",
            "iPhone 14 Pro Max",
            "Pixel 7",
            "iPad Pro 11",
            "Desktop Chrome",
            "Desktop Firefox",
        ]
    }
}
