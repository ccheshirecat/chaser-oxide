//! Stealth profile system for customizable browser fingerprints.
//!
//! This module provides an ergonomic builder pattern for creating consistent
//! browser "personalities" that bypass anti-bot detection.
//!
//! # Example
//!
//! ```rust
//! use chaser-oxide::profiles::{ChaserProfile, Gpu};
//!
//! let profile = ChaserProfile::windows()
//!     .chrome_version(130)
//!     .gpu(Gpu::NvidiaRTX4080)
//!     .memory_gb(16)
//!     .cpu_cores(12)
//!     .build();
//! ```

use std::fmt;

/// GPU presets for WebGL spoofing
#[derive(Debug, Clone, Copy)]
pub enum Gpu {
    /// NVIDIA GeForce RTX 3080 (high-trust gaming GPU)
    NvidiaRTX3080,
    /// NVIDIA GeForce RTX 4080 (newer gaming GPU)
    NvidiaRTX4080,
    /// NVIDIA GeForce GTX 1660 (mid-range GPU)
    NvidiaGTX1660,
    /// Intel UHD Graphics 630 (common laptop GPU)
    IntelUHD630,
    /// Intel Iris Xe (modern laptop GPU)
    IntelIrisXe,
    /// Apple M1 Pro
    AppleM1Pro,
    /// Apple M2 Max
    AppleM2Max,
    /// Apple M4 Max
    AppleM4Max,
    /// AMD Radeon RX 6800
    AmdRadeonRX6800,
}

impl Gpu {
    /// Returns the WebGL vendor string
    pub fn vendor(&self) -> &'static str {
        match self {
            Gpu::NvidiaRTX3080 | Gpu::NvidiaRTX4080 | Gpu::NvidiaGTX1660 => "Google Inc. (NVIDIA)",
            Gpu::IntelUHD630 | Gpu::IntelIrisXe => "Google Inc. (Intel)",
            Gpu::AppleM1Pro | Gpu::AppleM2Max | Gpu::AppleM4Max => "Google Inc. (Apple)",
            Gpu::AmdRadeonRX6800 => "Google Inc. (AMD)",
        }
    }

    /// Returns the WebGL renderer string
    pub fn renderer(&self) -> &'static str {
        match self {
            Gpu::NvidiaRTX3080 => {
                "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0)"
            }
            Gpu::NvidiaRTX4080 => {
                "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Direct3D11 vs_5_0 ps_5_0)"
            }
            Gpu::NvidiaGTX1660 => {
                "ANGLE (NVIDIA, NVIDIA GeForce GTX 1660 SUPER Direct3D11 vs_5_0 ps_5_0)"
            }
            Gpu::IntelUHD630 => "ANGLE (Intel, Intel(R) UHD Graphics 630 Direct3D11 vs_5_0 ps_5_0)",
            Gpu::IntelIrisXe => {
                "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0)"
            }
            Gpu::AppleM1Pro => "ANGLE (Apple, Apple M1 Pro, OpenGL 4.1)",
            Gpu::AppleM2Max => "ANGLE (Apple, Apple M2 Max, OpenGL 4.1)",
            Gpu::AppleM4Max => {
                "ANGLE (Apple, ANGLE Metal Renderer: Apple M4 Max, Unspecified Version)"
            }
            Gpu::AmdRadeonRX6800 => "ANGLE (AMD, AMD Radeon RX 6800 XT Direct3D11 vs_5_0 ps_5_0)",
        }
    }
}

/// Operating system presets
#[derive(Debug, Clone, Copy)]
pub enum Os {
    /// Windows 10/11 64-bit
    Windows,
    /// macOS (Intel)
    MacOSIntel,
    /// macOS (Apple Silicon)
    MacOSArm,
    /// Linux x86_64
    Linux,
}

impl Os {
    /// Returns the navigator.platform value
    pub fn platform(&self) -> &'static str {
        match self {
            Os::Windows => "Win32",
            Os::MacOSIntel | Os::MacOSArm => "MacIntel",
            Os::Linux => "Linux x86_64",
        }
    }

    /// Returns the client hints platform
    pub fn hints_platform(&self) -> &'static str {
        match self {
            Os::Windows => "Windows",
            Os::MacOSIntel | Os::MacOSArm => "macOS",
            Os::Linux => "Linux",
        }
    }
}

/// A builder for creating consistent browser fingerprint profiles.
///
/// # Example
///
/// ```rust
/// use chaser-oxide::profiles::{ChaserProfile, Gpu, Os};
///
/// // Quick preset
/// let profile = ChaserProfile::windows().build();
///
/// // Customized
/// let profile = ChaserProfile::new(Os::Windows)
///     .chrome_version(130)
///     .gpu(Gpu::NvidiaRTX4080)
///     .memory_gb(32)
///     .cpu_cores(16)
///     .locale("de-DE")
///     .timezone("Europe/Berlin")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ChaserProfile {
    os: Os,
    chrome_version: u32,
    gpu: Gpu,
    memory_gb: u32,
    cpu_cores: u32,
    locale: String,
    timezone: String,
    screen_width: u32,
    screen_height: u32,
    device_pixel_ratio: f32,
}

impl Default for ChaserProfile {
    fn default() -> Self {
        Self::windows().build()
    }
}

impl ChaserProfile {
    /// Create a new profile builder with the specified OS
    #[allow(clippy::new_ret_no_self)]
    pub fn new(os: Os) -> ChaserProfileBuilder {
        // OS-specific defaults for consistency
        let (screen_width, screen_height, device_pixel_ratio, cpu_cores) = match os {
            Os::Windows => (1920, 1080, 1.0, 8),
            Os::MacOSIntel => (1440, 900, 2.0, 8),
            Os::MacOSArm => (1728, 1117, 2.0, 14), // M4 Max defaults
            Os::Linux => (1920, 1080, 1.0, 8),
        };

        ChaserProfileBuilder {
            os,
            chrome_version: 131, // Keep reasonably current
            gpu: match os {
                Os::Windows => Gpu::NvidiaRTX3080,
                Os::MacOSIntel => Gpu::AppleM1Pro,
                Os::MacOSArm => Gpu::AppleM4Max,
                Os::Linux => Gpu::NvidiaGTX1660,
            },
            memory_gb: 8,
            cpu_cores,
            locale: "en-US".to_string(),
            timezone: "America/New_York".to_string(),
            screen_width,
            screen_height,
            device_pixel_ratio,
        }
    }

    /// Create a Windows profile with sensible defaults (RTX 3080, 8 cores)
    pub fn windows() -> ChaserProfileBuilder {
        Self::new(Os::Windows)
    }

    /// Create a macOS Intel profile (realistic MacBook Pro defaults)
    pub fn macos_intel() -> ChaserProfileBuilder {
        Self::new(Os::MacOSIntel)
    }

    /// Create a macOS Apple Silicon profile (M4 Max defaults from real device)
    pub fn macos_arm() -> ChaserProfileBuilder {
        Self::new(Os::MacOSArm)
    }

    /// Create a Linux profile
    pub fn linux() -> ChaserProfileBuilder {
        Self::new(Os::Linux)
    }

    // Getters
    pub fn os(&self) -> Os {
        self.os
    }
    pub fn chrome_version(&self) -> u32 {
        self.chrome_version
    }
    pub fn gpu(&self) -> Gpu {
        self.gpu
    }
    pub fn memory_gb(&self) -> u32 {
        self.memory_gb
    }
    pub fn cpu_cores(&self) -> u32 {
        self.cpu_cores
    }
    pub fn locale(&self) -> &str {
        &self.locale
    }
    pub fn timezone(&self) -> &str {
        &self.timezone
    }
    pub fn screen_width(&self) -> u32 {
        self.screen_width
    }
    pub fn screen_height(&self) -> u32 {
        self.screen_height
    }
    pub fn device_pixel_ratio(&self) -> f32 {
        self.device_pixel_ratio
    }

    /// Configure a BrowserConfigBuilder with this profile's recommended settings.
    /// 
    /// This sets:
    /// - Window size to match screen dimensions (prevents geometric leaks)
    /// - Stealth args for anti-detection
    /// 
    /// # Example
    /// ```rust
    /// let profile = ChaserProfile::windows().build();
    /// let config = profile.configure_browser(BrowserConfig::builder())
    ///     .with_head()
    ///     .build()?;
    /// ```
    pub fn configure_browser(
        &self,
        builder: crate::browser::BrowserConfigBuilder,
    ) -> crate::browser::BrowserConfigBuilder {
        builder
            .window_size(self.screen_width, self.screen_height)
            .args(vec![
                // Hide automation indicators
                "--disable-blink-features=AutomationControlled".to_string(),
                // Hide the automation infobar
                "--disable-infobars".to_string(),
                // Explicit window size as backup (belt and suspenders)
                format!("--window-size={},{}", self.screen_width, self.screen_height),
            ])
    }

    /// Generate the User-Agent string for this profile
    pub fn user_agent(&self) -> String {
        let os_part = match self.os {
            Os::Windows => "Windows NT 10.0; Win64; x64",
            Os::MacOSIntel | Os::MacOSArm => "Macintosh; Intel Mac OS X 10_15_7",
            Os::Linux => "X11; Linux x86_64",
        };
        format!(
            "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36",
            os_part, self.chrome_version
        )
    }

    /// Generate the complete JavaScript bootstrap script for this profile
    /// Single source of truth for ALL stealth - no separate chrome_runtime_mock needed
    pub fn bootstrap_script(&self) -> String {
        format!(
            r#"
            (function() {{
                // === chaser-oxide "GOD MODE" STEALTH (UNIFIED) ===
                // Profile: {ua}

                // ========== HELPER: Make functions appear native ==========
                // Recursive toString protection - prevents func.toString.toString() leak
                const makeNative = (func, name) => {{
                    Object.defineProperty(func, 'name', {{ value: name }});
                    const nativeStr = `function ${{name}}() {{ [native code] }}`;
                    const newToString = function() {{ return nativeStr; }};
                    Object.defineProperty(newToString, 'toString', {{
                        value: function() {{ return "function toString() {{ [native code] }}"; }}
                    }});
                    Object.defineProperty(newToString, 'name', {{ value: 'toString' }});
                    Object.defineProperty(func, 'toString', {{
                        value: newToString,
                        writable: true, enumerable: false, configurable: true
                    }});
                    return func;
                }};

                // ========== CDP/AUTOMATION MARKER CLEANUP (ONCE AT STARTUP) ==========
                const cleanCDPMarkers = () => {{
                    // Use getOwnPropertyNames for more thorough enumeration
                    const windowProps = Object.getOwnPropertyNames(window);
                    for (const prop of windowProps) {{
                        if (prop.match(/^cdc_|^\\$cdc_|^__webdriver|^__selenium|^__driver|^\\$chrome_/)) {{
                            try {{ delete window[prop]; }} catch(e) {{}}
                        }}
                    }}
                    const docProps = Object.getOwnPropertyNames(document);
                    for (const prop of docProps) {{
                        if (prop.match(/^cdc_|^\\$cdc_|^__webdriver|^__selenium|^__driver|^\\$chrome_/)) {{
                            try {{ delete document[prop]; }} catch(e) {{}}
                        }}
                    }}
                }};
                // Run exactly once before site loads - NO setInterval
                cleanCDPMarkers();

                // Get Navigator prototype
                const navProto = Object.getPrototypeOf(navigator);

                // ========== 1. PLATFORM & HARDWARE ==========
                Object.defineProperty(navProto, 'platform', {{
                    get: makeNative(function() {{ return '{platform}'; }}, 'get platform'),
                    configurable: true, enumerable: true
                }});
                Object.defineProperty(navProto, 'hardwareConcurrency', {{
                    get: makeNative(function() {{ return {cores}; }}, 'get hardwareConcurrency'),
                    configurable: true, enumerable: true
                }});
                Object.defineProperty(navProto, 'deviceMemory', {{
                    get: makeNative(function() {{ return {memory}; }}, 'get deviceMemory'),
                    configurable: true, enumerable: true
                }});
                Object.defineProperty(navProto, 'maxTouchPoints', {{
                    get: makeNative(function() {{ return 0; }}, 'get maxTouchPoints'),
                    configurable: true, enumerable: true
                }});

                // ========== 2. SCREEN & DPR ==========
                Object.defineProperty(window, 'devicePixelRatio', {{
                    get: makeNative(function() {{ return {dpr}; }}, 'get devicePixelRatio'),
                    configurable: true, enumerable: true
                }});
                Object.defineProperty(screen, 'width', {{
                    get: makeNative(function() {{ return {screen_w}; }}, 'get width'),
                    configurable: true
                }});
                Object.defineProperty(screen, 'height', {{
                    get: makeNative(function() {{ return {screen_h}; }}, 'get height'),
                    configurable: true
                }});
                Object.defineProperty(screen, 'availWidth', {{
                    get: makeNative(function() {{ return {screen_w}; }}, 'get availWidth'),
                    configurable: true
                }});
                Object.defineProperty(screen, 'availHeight', {{
                    get: makeNative(function() {{ return {screen_h}; }}, 'get availHeight'),
                    configurable: true
                }});

                // Spoof outerWidth/outerHeight to match (prevents TARDIS effect)
                // outerWidth should be >= innerWidth, add ~100px for browser chrome
                Object.defineProperty(window, 'outerWidth', {{
                    get: makeNative(function() {{ return {screen_w}; }}, 'get outerWidth'),
                    configurable: true
                }});
                Object.defineProperty(window, 'outerHeight', {{
                    get: makeNative(function() {{ return {screen_h} + 85; }}, 'get outerHeight'),
                    configurable: true
                }});

                // ========== 3. WEBGL ==========
                const spoofWebGL = (proto) => {{
                    const originalGetParameter = proto.getParameter;
                    proto.getParameter = makeNative(function(parameter) {{
                        try {{
                            if (parameter === 37445) return '{webgl_vendor}';
                            if (parameter === 37446) return '{webgl_renderer}';
                            return originalGetParameter.apply(this, arguments);
                        }} catch(e) {{
                            if (e && e.stack) {{
                                e.stack = e.stack.split('\\n').filter(line => 
                                    !line.includes('Object.apply') && !line.includes('<anonymous>')
                                ).join('\\n');
                            }}
                            throw e;
                        }}
                    }}, 'getParameter');
                }};
                try {{
                    spoofWebGL(WebGLRenderingContext.prototype);
                    if (typeof WebGL2RenderingContext !== 'undefined') {{
                        spoofWebGL(WebGL2RenderingContext.prototype);
                    }}
                }} catch(e) {{}}

                // ========== 4. CLIENT HINTS (userAgentData) ==========
                Object.defineProperty(navProto, 'userAgentData', {{
                    get: makeNative(function() {{
                        return {{
                            brands: [
                                {{ brand: "Google Chrome", version: "{chrome_ver}" }},
                                {{ brand: "Chromium", version: "{chrome_ver}" }},
                                {{ brand: "Not=A?Brand", version: "24" }}
                            ],
                            mobile: false,
                            platform: "{hints_platform}",
                            getHighEntropyValues: makeNative(async function(hints) {{
                                return {{
                                    architecture: "x86",
                                    bitness: "64",
                                    brands: [
                                        {{ brand: "Google Chrome", version: "{chrome_ver}" }},
                                        {{ brand: "Chromium", version: "{chrome_ver}" }},
                                        {{ brand: "Not=A?Brand", version: "24" }}
                                    ],
                                    fullVersionList: [
                                        {{ brand: "Google Chrome", version: "{chrome_ver}.0.0.0" }},
                                        {{ brand: "Chromium", version: "{chrome_ver}.0.0.0" }},
                                        {{ brand: "Not=A?Brand", version: "24.0.0.0" }}
                                    ],
                                    mobile: false,
                                    model: "",
                                    platform: "{hints_platform}",
                                    platformVersion: "10.0.0",
                                    uaFullVersion: "{chrome_ver}.0.0.0"
                                }};
                            }}, 'getHighEntropyValues'),
                            toJSON: makeNative(function() {{
                                return {{
                                    brands: [
                                        {{ brand: "Google Chrome", version: "{chrome_ver}" }},
                                        {{ brand: "Chromium", version: "{chrome_ver}" }},
                                        {{ brand: "Not=A?Brand", version: "24" }}
                                    ],
                                    mobile: false,
                                    platform: "{hints_platform}"
                                }};
                            }}, 'toJSON')
                        }};
                    }}, 'get userAgentData'),
                    configurable: true, enumerable: true
                }});

                // ========== 5. VIDEO CODECS ==========
                const originalCanPlayType = HTMLMediaElement.prototype.canPlayType;
                HTMLMediaElement.prototype.canPlayType = makeNative(function(type) {{
                    if (!type) return originalCanPlayType.apply(this, arguments);
                    if (type.includes('avc1') || type.includes('mp4a.40') || type === 'video/mp4' || type === 'audio/mp4') {{
                        return 'probably';
                    }}
                    return originalCanPlayType.apply(this, arguments);
                }}, 'canPlayType');

                // ========== 6. WEBDRIVER (SET TO FALSE) ==========
                // Deleting creates undefined which is DETECTABLE by rebrowser-bot-detector!
                // Real browsers have navigator.webdriver = false
                Object.defineProperty(navProto, 'webdriver', {{
                    get: makeNative(function() {{ return false; }}, 'get webdriver'),
                    configurable: true, enumerable: true
                }});

                // ========== 7. TIMEZONE & LOCALE ==========
                Object.defineProperty(navProto, 'language', {{
                    get: makeNative(function() {{ return '{locale}'; }}, 'get language'),
                    configurable: true, enumerable: true
                }});
                Object.defineProperty(navProto, 'languages', {{
                    get: makeNative(function() {{ return ['{locale}', 'en']; }}, 'get languages'),
                    configurable: true, enumerable: true
                }});

                // Mock Intl.DateTimeFormat for timezone
                const originalDateTimeFormat = Intl.DateTimeFormat;
                Intl.DateTimeFormat = makeNative(function(locales, options) {{
                    const opts = options || {{}};
                    if (!opts.timeZone) opts.timeZone = '{timezone}';
                    const formatter = new originalDateTimeFormat(locales || '{locale}', opts);
                    const origResolved = formatter.resolvedOptions.bind(formatter);
                    formatter.resolvedOptions = makeNative(function() {{
                        const result = origResolved();
                        result.timeZone = '{timezone}';
                        result.locale = '{locale}';
                        return result;
                    }}, 'resolvedOptions');
                    return formatter;
                }}, 'DateTimeFormat');
                Intl.DateTimeFormat.prototype = originalDateTimeFormat.prototype;
                Intl.DateTimeFormat.supportedLocalesOf = originalDateTimeFormat.supportedLocalesOf;

                // ========== 8. WINDOW.CHROME (complete) ==========
                if (!window.chrome) {{
                    Object.defineProperty(window, 'chrome', {{
                        writable: true, enumerable: true, configurable: false, value: {{}}
                    }});
                }}
                if (!window.chrome.runtime) {{
                    Object.defineProperty(window.chrome, 'runtime', {{
                        writable: true, enumerable: true, configurable: false, value: {{}}
                    }});
                }}
                if (!window.chrome.runtime.connect) {{
                    Object.defineProperty(window.chrome.runtime, 'connect', {{
                        configurable: false, enumerable: true, writable: true,
                        value: makeNative(function() {{
                            return {{
                                name: '',
                                onDisconnect: {{ addListener: function(){{}}, removeListener: function(){{}}, hasListener: function(){{}}, hasListeners: function(){{}}, dispatch: function(){{}} }},
                                onMessage: {{ addListener: function(){{}}, removeListener: function(){{}}, hasListener: function(){{}}, hasListeners: function(){{}}, dispatch: function(){{}} }},
                                postMessage: function(){{}},
                                disconnect: function(){{}}
                            }};
                        }}, 'connect')
                    }});
                }}
                if (!window.chrome.runtime.sendMessage) {{
                    Object.defineProperty(window.chrome.runtime, 'sendMessage', {{
                        configurable: false, enumerable: true, writable: true,
                        value: makeNative(function() {{ return; }}, 'sendMessage')
                    }});
                }}
                if (!window.chrome.csi) {{
                    Object.defineProperty(window.chrome, 'csi', {{
                        configurable: false, enumerable: true, writable: true,
                        value: makeNative(function() {{
                            return {{ startE: Date.now(), onloadT: Date.now(), pageT: Date.now(), tran: 15 }};
                        }}, 'csi')
                    }});
                }}
                if (!window.chrome.loadTimes) {{
                    Object.defineProperty(window.chrome, 'loadTimes', {{
                        configurable: false, enumerable: true, writable: true,
                        value: makeNative(function() {{
                            return {{
                                requestTime: Date.now() / 1000, startLoadTime: Date.now() / 1000,
                                commitLoadTime: Date.now() / 1000, finishDocumentLoadTime: Date.now() / 1000,
                                finishLoadTime: Date.now() / 1000, firstPaintTime: Date.now() / 1000,
                                firstPaintAfterLoadTime: 0, navigationType: "Other",
                                wasFetchedViaSpdy: false, wasNpnNegotiated: false,
                                npnNegotiatedProtocol: "", wasAlternateProtocolAvailable: false,
                                connectionInfo: "http/1.1"
                            }};
                        }}, 'loadTimes')
                    }});
                }}
                if (!window.chrome.app) {{
                    Object.defineProperty(window.chrome, 'app', {{
                        configurable: false, enumerable: true, writable: true,
                        value: {{
                            isInstalled: false,
                            InstallState: {{ DISABLED: 'disabled', INSTALLED: 'installed', NOT_INSTALLED: 'not_installed' }},
                            RunningState: {{ CANNOT_RUN: 'cannot_run', READY_TO_RUN: 'ready_to_run', RUNNING: 'running' }},
                            getIsInstalled: makeNative(function() {{ return false; }}, 'getIsInstalled'),
                            getDetails: makeNative(function() {{ return null; }}, 'getDetails')
                        }}
                    }});
                }}
                if (!window.chrome.webstore) {{
                    Object.defineProperty(window.chrome, 'webstore', {{
                        configurable: false, enumerable: true, writable: true,
                        value: {{ onInstallStageChanged: {{}}, onDownloadProgress: {{}} }}
                    }});
                }}

                // ========== 9. PLUGINS ==========
                const makePlugin = (name, filename, description) => {{
                    const plugin = Object.create(Plugin.prototype);
                    Object.defineProperties(plugin, {{
                        name: {{ value: name, enumerable: true }},
                        filename: {{ value: filename, enumerable: true }},
                        description: {{ value: description, enumerable: true }},
                        length: {{ value: 1, enumerable: true }},
                        0: {{ value: {{ type: 'application/pdf', suffixes: 'pdf', description }}, enumerable: true }}
                    }});
                    return plugin;
                }};
                const fakePlugins = Object.create(PluginArray.prototype);
                const pluginList = [
                    makePlugin('PDF Viewer', 'internal-pdf-viewer', 'Portable Document Format'),
                    makePlugin('Chrome PDF Viewer', 'internal-pdf-viewer', 'Portable Document Format'),
                    makePlugin('Chromium PDF Viewer', 'internal-pdf-viewer', 'Portable Document Format'),
                    makePlugin('Microsoft Edge PDF Viewer', 'internal-pdf-viewer', 'Portable Document Format'),
                    makePlugin('WebKit built-in PDF', 'internal-pdf-viewer', 'Portable Document Format')
                ];
                pluginList.forEach((p, i) => {{
                    Object.defineProperty(fakePlugins, i, {{ value: p, enumerable: true }});
                }});
                Object.defineProperty(fakePlugins, 'length', {{ value: pluginList.length, enumerable: true }});
                Object.defineProperty(fakePlugins, 'item', {{ 
                    value: makeNative(function(index) {{ return this[index] || null; }}, 'item'),
                    enumerable: false
                }});
                Object.defineProperty(fakePlugins, 'namedItem', {{ 
                    value: makeNative(function(name) {{ 
                        for (let i = 0; i < this.length; i++) if (this[i].name === name) return this[i];
                        return null;
                    }}, 'namedItem'),
                    enumerable: false
                }});
                Object.defineProperty(fakePlugins, 'refresh', {{ 
                    value: makeNative(function() {{}}, 'refresh'),
                    enumerable: false 
                }});
                Object.defineProperty(fakePlugins, Symbol.iterator, {{
                    value: function* () {{ for (let i = 0; i < this.length; i++) yield this[i]; }},
                    enumerable: false
                }});
                Object.defineProperty(navProto, 'plugins', {{ 
                    get: makeNative(function() {{ return fakePlugins; }}, 'get plugins'),
                    configurable: true, enumerable: true
                }});

                // ========== 10. PERMISSIONS ==========
                try {{
                    const originalQuery = window.navigator.permissions.query;
                    Object.defineProperty(window.navigator.permissions.__proto__, 'query', {{
                        value: makeNative(function(parameters) {{
                            return parameters.name === 'notifications'
                                ? Promise.resolve({{ state: Notification.permission }})
                                : originalQuery.call(this, parameters);
                        }}, 'query'),
                        writable: true, configurable: true
                    }});
                }} catch(e) {{}}

                // ========== 11. IFRAME PROTECTION (Whitelisted for Turnstile) ==========
                const originalCreateElement = document.createElement;
                document.createElement = makeNative(function(...args) {{
                    const element = originalCreateElement.apply(this, args);
                    if (args[0] && args[0].toLowerCase() === 'iframe') {{
                        element.addEventListener('load', () => {{
                            try {{
                                // CRITICAL: Do NOT touch Cloudflare/Turnstile frames
                                // Accessing cross-origin frames triggers detection or CORS errors
                                if (element.src && (
                                    element.src.includes('cloudflare.com') ||
                                    element.src.includes('challenges.cloudflare.com') ||
                                    element.src.includes('turnstile')
                                )) {{
                                    return; // Leave Turnstile alone!
                                }}
                                
                                // For other frames (same-origin), inject chrome
                                if (element.contentWindow && !element.contentWindow.chrome) {{
                                    element.contentWindow.chrome = window.chrome;
                                }}
                            }} catch(e) {{}}
                        }});
                    }}
                    return element;
                }}, 'createElement');

            }})();
        "#,
            ua = self.user_agent(),
            platform = self.os.platform(),
            cores = self.cpu_cores,
            memory = self.memory_gb,
            dpr = self.device_pixel_ratio,
            screen_w = self.screen_width,
            screen_h = self.screen_height,
            webgl_vendor = self.gpu.vendor(),
            webgl_renderer = self.gpu.renderer(),
            chrome_ver = self.chrome_version,
            hints_platform = self.os.hints_platform(),
            locale = self.locale,
            timezone = self.timezone,
        )
    }
}

impl fmt::Display for ChaserProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ChaserProfile({:?}, Chrome {}, {:?})",
            self.os, self.chrome_version, self.gpu
        )
    }
}

/// Builder for constructing `ChaserProfile` instances
#[derive(Debug, Clone)]
pub struct ChaserProfileBuilder {
    os: Os,
    chrome_version: u32,
    gpu: Gpu,
    memory_gb: u32,
    cpu_cores: u32,
    locale: String,
    timezone: String,
    screen_width: u32,
    screen_height: u32,
    device_pixel_ratio: f32,
}

impl ChaserProfileBuilder {
    /// Set the Chrome version (default: 129)
    pub fn chrome_version(mut self, version: u32) -> Self {
        self.chrome_version = version;
        self
    }

    /// Set the GPU for WebGL spoofing
    pub fn gpu(mut self, gpu: Gpu) -> Self {
        self.gpu = gpu;
        self
    }

    /// Set device memory in GB (default: 8)
    pub fn memory_gb(mut self, gb: u32) -> Self {
        self.memory_gb = gb;
        self
    }

    /// Set CPU core count (default: 8)
    pub fn cpu_cores(mut self, cores: u32) -> Self {
        self.cpu_cores = cores;
        self
    }

    /// Set the locale (e.g., "en-US", "de-DE")
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }

    /// Set the timezone (e.g., "America/New_York", "Europe/Berlin")
    pub fn timezone(mut self, tz: impl Into<String>) -> Self {
        self.timezone = tz.into();
        self
    }

    /// Set screen resolution
    pub fn screen(mut self, width: u32, height: u32) -> Self {
        self.screen_width = width;
        self.screen_height = height;
        self
    }

    /// Set device pixel ratio (1.0 for standard, 2.0 for Retina/HiDPI)
    pub fn device_pixel_ratio(mut self, dpr: f32) -> Self {
        self.device_pixel_ratio = dpr;
        self
    }

    /// Build the final profile
    pub fn build(self) -> ChaserProfile {
        ChaserProfile {
            os: self.os,
            chrome_version: self.chrome_version,
            gpu: self.gpu,
            memory_gb: self.memory_gb,
            cpu_cores: self.cpu_cores,
            locale: self.locale,
            timezone: self.timezone,
            screen_width: self.screen_width,
            screen_height: self.screen_height,
            device_pixel_ratio: self.device_pixel_ratio,
        }
    }
}
