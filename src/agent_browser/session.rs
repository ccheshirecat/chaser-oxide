//! Session & Profile Management (Phase 22-24).
//!
//! Provides session isolation, profile persistence, launch configuration,
//! and cloud provider integration for browser automation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::response::{AgentError, AgentResult};

// =========================================================================
// Session Management (Phase 22)
// =========================================================================

/// Session configuration for isolated browser instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Unique session identifier.
    pub session_id: String,
    /// Optional profile name for persistent data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Custom user data directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data_dir: Option<PathBuf>,
    /// Whether to auto-save state on session end.
    #[serde(default)]
    pub auto_save: bool,
    /// PID file path for daemon detection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid_file: Option<PathBuf>,
}

impl SessionConfig {
    /// Create a new session with the given ID.
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            profile: None,
            user_data_dir: None,
            auto_save: false,
            pid_file: None,
        }
    }

    /// Set a profile name for persistent data.
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    /// Set a custom user data directory.
    pub fn with_user_data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.user_data_dir = Some(path.into());
        self
    }

    /// Enable auto-save of state on session end.
    pub fn with_auto_save(mut self) -> Self {
        self.auto_save = true;
        self
    }

    /// Get the user data directory, creating a default one based on session/profile.
    pub fn data_dir(&self) -> PathBuf {
        if let Some(ref dir) = self.user_data_dir {
            dir.clone()
        } else if let Some(ref profile) = self.profile {
            dirs_base_path().join("profiles").join(profile)
        } else {
            dirs_base_path().join("sessions").join(&self.session_id)
        }
    }

    /// Get the PID file path.
    pub fn pid_path(&self) -> PathBuf {
        self.pid_file.clone().unwrap_or_else(|| {
            dirs_base_path()
                .join("pids")
                .join(format!("{}.pid", self.session_id))
        })
    }

    /// Write the PID file.
    pub fn write_pid(&self) -> AgentResult<()> {
        let pid_path = self.pid_path();
        if let Some(parent) = pid_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AgentError::Internal {
                message: format!("Failed to create PID directory: {}", e),
            })?;
        }
        std::fs::write(&pid_path, std::process::id().to_string()).map_err(|e| {
            AgentError::Internal {
                message: format!("Failed to write PID file: {}", e),
            }
        })
    }

    /// Remove the PID file.
    pub fn remove_pid(&self) -> AgentResult<()> {
        let pid_path = self.pid_path();
        if pid_path.exists() {
            std::fs::remove_file(&pid_path).map_err(|e| AgentError::Internal {
                message: format!("Failed to remove PID file: {}", e),
            })?;
        }
        Ok(())
    }

    /// Check if another instance is running for this session.
    pub fn is_running(&self) -> bool {
        let pid_path = self.pid_path();
        if !pid_path.exists() {
            return false;
        }
        // Check if the PID is still alive
        if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                return is_process_alive(pid);
            }
        }
        false
    }
}

/// State snapshot for saving/loading session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Cookies.
    #[serde(default)]
    pub cookies: Vec<serde_json::Value>,
    /// LocalStorage data (per origin).
    #[serde(default)]
    pub local_storage: HashMap<String, HashMap<String, String>>,
    /// SessionStorage data (per origin).
    #[serde(default)]
    pub session_storage: HashMap<String, HashMap<String, String>>,
    /// Timestamp of the state snapshot.
    pub timestamp: f64,
}

impl SessionState {
    /// Create an empty state.
    pub fn new() -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        Self {
            cookies: Vec::new(),
            local_storage: HashMap::new(),
            session_storage: HashMap::new(),
            timestamp,
        }
    }

    /// Save state to a file.
    pub fn save(&self, path: &str) -> AgentResult<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| AgentError::Internal {
            message: format!("Failed to serialize state: {}", e),
        })?;
        std::fs::write(path, json).map_err(|e| AgentError::Internal {
            message: format!("Failed to write state file: {}", e),
        })
    }

    /// Load state from a file.
    pub fn load(path: &str) -> AgentResult<Self> {
        let json = std::fs::read_to_string(path).map_err(|e| AgentError::Internal {
            message: format!("Failed to read state file: {}", e),
        })?;
        serde_json::from_str(&json).map_err(|e| AgentError::Internal {
            message: format!("Failed to parse state file: {}", e),
        })
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// Launch Options (Phase 23)
// =========================================================================

/// Browser launch options compatible with agent-browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchOptions {
    /// Run in headless mode.
    #[serde(default = "default_headless")]
    pub headless: bool,

    /// Custom browser executable path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<String>,

    /// Proxy server URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyConfig>,

    /// Extensions to load (paths).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<String>,

    /// Additional browser arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    /// User data directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data_dir: Option<String>,

    /// Viewport width.
    #[serde(default = "default_width")]
    pub viewport_width: u32,

    /// Viewport height.
    #[serde(default = "default_height")]
    pub viewport_height: u32,

    /// Whether to enable stealth mode.
    #[serde(default)]
    pub stealth: bool,

    /// Session configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<SessionConfig>,

    /// CDP endpoint to connect to (instead of launching).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdp_endpoint: Option<String>,

    /// Cloud provider to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Timezone override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    /// Locale override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    /// Storage state file to load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_state: Option<String>,
}

fn default_headless() -> bool {
    true
}
fn default_width() -> u32 {
    1280
}
fn default_height() -> u32 {
    720
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            headless: true,
            executable_path: None,
            proxy: None,
            extensions: Vec::new(),
            args: Vec::new(),
            user_data_dir: None,
            viewport_width: 1280,
            viewport_height: 720,
            stealth: false,
            session: None,
            cdp_endpoint: None,
            provider: None,
            timezone: None,
            locale: None,
            storage_state: None,
        }
    }
}

impl LaunchOptions {
    /// Create default launch options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set headless mode.
    pub fn headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }

    /// Set executable path.
    pub fn executable(mut self, path: impl Into<String>) -> Self {
        self.executable_path = Some(path.into());
        self
    }

    /// Set proxy configuration.
    pub fn proxy(mut self, proxy: ProxyConfig) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Add an extension path.
    pub fn extension(mut self, path: impl Into<String>) -> Self {
        self.extensions.push(path.into());
        self
    }

    /// Add a browser argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set viewport dimensions.
    pub fn viewport(mut self, width: u32, height: u32) -> Self {
        self.viewport_width = width;
        self.viewport_height = height;
        self
    }

    /// Enable stealth mode.
    pub fn stealth(mut self) -> Self {
        self.stealth = true;
        self
    }

    /// Set CDP endpoint for connecting to existing browser.
    pub fn cdp(mut self, endpoint: impl Into<String>) -> Self {
        self.cdp_endpoint = Some(endpoint.into());
        self
    }

    /// Convert to BrowserConfig builder arguments.
    pub fn to_browser_args(&self) -> Vec<String> {
        let mut args = self.args.clone();

        if let Some(ref proxy) = self.proxy {
            args.push(format!("--proxy-server={}", proxy.server));
        }

        for ext in &self.extensions {
            args.push(format!("--load-extension={}", ext));
        }

        if !self.extensions.is_empty() {
            args.push(format!(
                "--disable-extensions-except={}",
                self.extensions.join(",")
            ));
        }

        args
    }

    /// Create from environment variables.
    pub fn from_env() -> Self {
        let mut opts = Self::default();

        if let Ok(v) = std::env::var("CHASER_HEADLESS") {
            opts.headless = v != "false" && v != "0";
        }
        if let Ok(v) = std::env::var("CHASER_EXECUTABLE") {
            opts.executable_path = Some(v);
        }
        if let Ok(v) = std::env::var("CHASER_PROXY") {
            opts.proxy = Some(ProxyConfig {
                server: v,
                username: std::env::var("CHASER_PROXY_USERNAME").ok(),
                password: std::env::var("CHASER_PROXY_PASSWORD").ok(),
            });
        }
        if let Ok(v) = std::env::var("CHASER_CDP") {
            opts.cdp_endpoint = Some(v);
        }
        if let Ok(v) = std::env::var("AGENT_BROWSER_PROVIDER") {
            opts.provider = Some(v);
        }
        if let Ok(v) = std::env::var("CHASER_STEALTH") {
            if v == "true" || v == "1" {
                opts.stealth = true;
            }
        }

        opts
    }
}

/// Proxy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy server URL (e.g., "http://proxy:8080").
    pub server: String,
    /// Optional username for authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Optional password for authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

// =========================================================================
// Cloud Providers (Phase 24)
// =========================================================================

/// Cloud browser provider trait.
pub trait CloudProvider: Send + Sync {
    /// Get the provider name.
    fn name(&self) -> &str;

    /// Get the WebSocket endpoint URL.
    fn endpoint(&self) -> AgentResult<String>;

    /// Get additional browser arguments for this provider.
    fn browser_args(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Browserbase cloud provider.
#[derive(Debug, Clone)]
pub struct BrowserbaseProvider {
    /// API key.
    pub api_key: String,
    /// Project ID.
    pub project_id: String,
    /// Optional session ID for reconnecting.
    pub session_id: Option<String>,
}

impl BrowserbaseProvider {
    /// Create from environment variables.
    pub fn from_env() -> AgentResult<Self> {
        let api_key = std::env::var("BROWSERBASE_API_KEY").map_err(|_| AgentError::Internal {
            message: "BROWSERBASE_API_KEY not set".to_string(),
        })?;
        let project_id =
            std::env::var("BROWSERBASE_PROJECT_ID").map_err(|_| AgentError::Internal {
                message: "BROWSERBASE_PROJECT_ID not set".to_string(),
            })?;
        Ok(Self {
            api_key,
            project_id,
            session_id: std::env::var("BROWSERBASE_SESSION_ID").ok(),
        })
    }
}

impl CloudProvider for BrowserbaseProvider {
    fn name(&self) -> &str {
        "browserbase"
    }

    fn endpoint(&self) -> AgentResult<String> {
        if let Some(ref session_id) = self.session_id {
            Ok(format!(
                "wss://connect.browserbase.com?apiKey={}&sessionId={}",
                self.api_key, session_id
            ))
        } else {
            Ok(format!(
                "wss://connect.browserbase.com?apiKey={}&projectId={}",
                self.api_key, self.project_id
            ))
        }
    }
}

/// Generic cloud provider that connects to any CDP endpoint.
#[derive(Debug, Clone)]
pub struct GenericCdpProvider {
    /// WebSocket endpoint URL.
    pub endpoint_url: String,
}

impl GenericCdpProvider {
    /// Create a new generic CDP provider.
    pub fn new(endpoint_url: impl Into<String>) -> Self {
        Self {
            endpoint_url: endpoint_url.into(),
        }
    }
}

impl CloudProvider for GenericCdpProvider {
    fn name(&self) -> &str {
        "generic-cdp"
    }

    fn endpoint(&self) -> AgentResult<String> {
        Ok(self.endpoint_url.clone())
    }
}

/// Resolve a cloud provider by name.
pub fn resolve_provider(name: &str) -> AgentResult<Box<dyn CloudProvider>> {
    match name.to_lowercase().as_str() {
        "browserbase" => {
            let provider = BrowserbaseProvider::from_env()?;
            Ok(Box::new(provider))
        }
        _ => Err(AgentError::InvalidCommand {
            message: format!(
                "Unknown provider '{}'. Available: browserbase. Or use --cdp for direct connection.",
                name
            ),
        }),
    }
}

// =========================================================================
// Helpers
// =========================================================================

/// Get the base path for chaser-oxide data.
fn dirs_base_path() -> PathBuf {
    if let Ok(v) = std::env::var("CHASER_DATA_DIR") {
        return PathBuf::from(v);
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("chaser-oxide");
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg).join("chaser-oxide");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".local/share/chaser-oxide");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(appdata).join("chaser-oxide");
        }
    }

    PathBuf::from(".chaser-oxide")
}

/// Check if a process is alive by PID.
fn is_process_alive(pid: u32) -> bool {
    // Check /proc/<pid> on Linux, which doesn't require libc
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new(&format!("/proc/{}", pid)).exists()
    }

    #[cfg(not(target_os = "linux"))]
    {
        // On other platforms, try a non-destructive process check
        let _ = pid;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_new() {
        let config = SessionConfig::new("test-session");
        assert_eq!(config.session_id, "test-session");
        assert!(config.profile.is_none());
        assert!(!config.auto_save);
    }

    #[test]
    fn test_session_config_builder() {
        let config = SessionConfig::new("s1")
            .with_profile("my-profile")
            .with_auto_save();
        assert_eq!(config.session_id, "s1");
        assert_eq!(config.profile.as_deref(), Some("my-profile"));
        assert!(config.auto_save);
    }

    #[test]
    fn test_session_state_save_load() {
        let state = SessionState::new();
        let path = "/tmp/chaser-oxide-test-state.json";
        state.save(path).unwrap();

        let loaded = SessionState::load(path).unwrap();
        assert!(loaded.cookies.is_empty());

        // Clean up
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_launch_options_default() {
        let opts = LaunchOptions::default();
        assert!(opts.headless);
        assert_eq!(opts.viewport_width, 1280);
        assert_eq!(opts.viewport_height, 720);
        assert!(opts.args.is_empty());
    }

    #[test]
    fn test_launch_options_builder() {
        let opts = LaunchOptions::new()
            .headless(false)
            .viewport(1920, 1080)
            .stealth()
            .arg("--no-sandbox")
            .extension("/path/to/ext");

        assert!(!opts.headless);
        assert_eq!(opts.viewport_width, 1920);
        assert_eq!(opts.viewport_height, 1080);
        assert!(opts.stealth);

        let args = opts.to_browser_args();
        assert!(args.contains(&"--no-sandbox".to_string()));
        assert!(args.iter().any(|a| a.contains("--load-extension")));
    }

    #[test]
    fn test_launch_options_proxy() {
        let opts = LaunchOptions::new().proxy(ProxyConfig {
            server: "http://proxy:8080".to_string(),
            username: None,
            password: None,
        });

        let args = opts.to_browser_args();
        assert!(args
            .iter()
            .any(|a| a.contains("--proxy-server=http://proxy:8080")));
    }

    #[test]
    fn test_generic_cdp_provider() {
        let provider = GenericCdpProvider::new("ws://localhost:9222");
        assert_eq!(provider.name(), "generic-cdp");
        assert_eq!(provider.endpoint().unwrap(), "ws://localhost:9222");
    }

    #[test]
    fn test_resolve_unknown_provider() {
        let result = resolve_provider("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_data_dir_default() {
        let config = SessionConfig::new("test");
        let dir = config.data_dir();
        assert!(dir.to_string_lossy().contains("test"));
    }
}
