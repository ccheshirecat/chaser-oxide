# chaser-oxide

[![Crates.io](https://img.shields.io/crates/v/chaser-oxide.svg)](https://crates.io/crates/chaser-oxide)
[![Documentation](https://docs.rs/chaser-oxide/badge.svg)](https://docs.rs/chaser-oxide)
[![License](https://img.shields.io/crates/l/chaser-oxide.svg)](https://github.com/ccheshirecat/chaser-oxide)

**A Rust-based fork of `chromiumoxide` for hardened, undetectable browser automation.**

chaser-oxide modifies the Chrome DevTools Protocol (CDP) client at the transport and protocol layer to reduce the detection footprint of automated browser sessions. The default profile auto-detects your host OS, Chrome version, and RAM — no hardcoded Windows spoofing out of the box.

## Features

- **Protocol-Level Stealth**: Patches CDP at the transport layer, not via JavaScript wrappers
- **Native Profile**: Auto-detects host OS, real Chrome version, and system RAM by default
- **Fingerprint Profiles**: Pre-configured Windows, Linux, macOS profiles with consistent hardware fingerprints for explicit OS spoofing
- **Client Hints Sync**: Full `UserAgentMetadata` via `Emulation.setUserAgentOverride` so `Sec-CH-UA-*` headers match the spoofed UA
- **Human Interaction Engine**: Physics-based Bezier mouse movements and realistic typing patterns
- **Request Interception**: Built-in request modification and blocking
- **Low Memory Footprint**: ~50–100MB vs ~500MB+ for Node.js alternatives

## Installation

```bash
cargo add chaser-oxide tokio futures
```

Or in `Cargo.toml`:

```toml
[dependencies]
chaser-oxide = "0.2.3"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
anyhow = "1.0.102"
serde_json = "1.0.149"
```

## Requirements

- Rust 1.85+
- Chrome/Chromium browser installed
- Supported platforms: Windows, macOS, Linux

## Quick Start

### Native Profile (recommended)

Uses your real OS, real Chrome version, and real RAM. Just strips HeadlessChrome from the UA.

```rust
use chaser_oxide::{Browser, BrowserConfig, ChaserPage};
use futures::StreamExt;
use serde_json::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .new_headless_mode()
            .build()
            .map_err(|e| anyhow::anyhow!(e))?,
    )
    .await?;

    tokio::spawn(async move { while let Some(_) = handler.next().await {} });

    let page = browser.new_page("about:blank").await?;
    let chaser = ChaserPage::new(page);

    // Reads Chrome version from the live browser via CDP — accurate even
    // when using chromiumoxide_fetcher's downloaded binary
    chaser.apply_native_profile().await?;

    chaser.goto("https://example.com").await?;
    let title: Option<Value> = chaser.evaluate("document.title").await?;
    println!("{:?}", title);

    Ok(())
}
```

### Explicit OS Spoofing

Opt into a specific profile when you need to appear as a different OS:

```rust
use chaser_oxide::{Browser, BrowserConfig, ChaserPage, ChaserProfile, Gpu};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let profile = ChaserProfile::windows()
        .chrome_version(131)
        .gpu(Gpu::NvidiaRTX3080)
        .memory_gb(16)
        .cpu_cores(8)
        .locale("en-US")
        .timezone("America/New_York")
        .screen(1920, 1080)
        .build();

    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .build()
            .map_err(|e| anyhow::anyhow!(e))?,
    )
    .await?;

    tokio::spawn(async move { while let Some(_) = handler.next().await {} });

    let page = browser.new_page("about:blank").await?;
    let chaser = ChaserPage::new(page);

    // Apply BEFORE navigation
    chaser.apply_profile(&profile).await?;

    chaser.goto("https://example.com").await?;
    Ok(())
}
```

## API Reference

### ChaserPage

```rust
impl ChaserPage {
    fn new(page: Page) -> Self;

    // Profile — call BEFORE navigation
    async fn apply_native_profile(&self) -> Result<()>;
    async fn apply_profile(&self, profile: &ChaserProfile) -> Result<()>;

    // Navigation
    async fn goto(&self, url: &str) -> Result<()>;
    async fn content(&self) -> Result<String>;
    async fn url(&self) -> Result<Option<String>>;

    // JS evaluation (stealth — uses isolated world, no Runtime.enable leak)
    async fn evaluate(&self, script: &str) -> Result<Option<Value>>;

    // Human-like mouse (Bezier curves with acceleration)
    async fn move_mouse_human(&self, x: f64, y: f64) -> Result<()>;
    async fn click_human(&self, x: f64, y: f64) -> Result<()>;
    async fn scroll_human(&self, delta_y: i32) -> Result<()>;

    // Typing
    async fn type_text(&self, text: &str) -> Result<()>;
    async fn type_text_with_typos(&self, text: &str) -> Result<()>;
    async fn press_key(&self, key: &str) -> Result<()>;
    async fn press_enter(&self) -> Result<()>;
    async fn press_tab(&self) -> Result<()>;

    // Request interception
    async fn enable_request_interception(&self, pattern: &str, resource_type: Option<ResourceType>) -> Result<()>;
    async fn disable_request_interception(&self) -> Result<()>;
    async fn fulfill_request_html(&self, request_id: RequestId, html: &str, status: u16) -> Result<()>;
    async fn continue_request(&self, request_id: impl Into<String>) -> Result<()>;

    // Escape hatch — raw_page().evaluate() triggers Runtime.enable detection!
    fn raw_page(&self) -> &Page;
}
```

### ChaserProfile Builder

```rust
use chaser_oxide::{ChaserProfile, Gpu};

// Auto-detect from host environment
let native  = ChaserProfile::native().build();

// Explicit OS presets
let windows    = ChaserProfile::windows().build();
let linux      = ChaserProfile::linux().build();
let mac_arm    = ChaserProfile::macos_arm().build();
let mac_intel  = ChaserProfile::macos_intel().build();

// Builder options (chain onto any preset)
let custom = ChaserProfile::windows()
    .chrome_version(131)
    .gpu(Gpu::NvidiaRTX4080)
    .memory_gb(16)
    .cpu_cores(8)
    .locale("de-DE")
    .timezone("Europe/Berlin")
    .screen(2560, 1440)
    .build();
```

### Available GPUs

```rust
pub enum Gpu {
    // NVIDIA
    NvidiaRTX4090, NvidiaRTX4080, NvidiaRTX4070,
    NvidiaRTX3090, NvidiaRTX3080, NvidiaRTX3070, NvidiaRTX3060,
    NvidiaGTX1660, NvidiaGTX1080,
    // AMD
    AmdRX7900XTX, AmdRX6800XT, AmdRX6700XT,
    // Intel
    IntelUHD630, IntelIrisXe,
    // Apple
    AppleM1, AppleM1Pro, AppleM2, AppleM3, AppleM4Max,
}
```

### BrowserConfig

```rust
let config = BrowserConfig::builder()
    .chrome_executable("/path/to/chrome")
    .new_headless_mode()   // Headless (Chrome's new headless — less detectable)
    .with_head()           // Headed window
    .viewport(Viewport {
        width: 1920,
        height: 1080,
        device_scale_factor: None,
        emulating_mobile: false,
        is_landscape: false,
        has_touch: false,
    })
    .build()?;
```

## Stealth Details

### What `apply_native_profile()` does

1. Reads the live Chrome version from the browser via `Browser.getVersion` CDP
2. Detects host OS (including arm64 vs x86 on macOS) and system RAM
3. Calls `Emulation.setUserAgentOverride` with full `UserAgentMetadata` — this controls both the `User-Agent` header **and** all `Sec-CH-UA-*` client hint headers
4. Injects bootstrap JS via `Page.createIsolatedWorld` before first navigation

### What `enable_stealth_mode()` does (Page-level)

Applies basic automation signal removal without any OS or version spoofing:
- Removes CDP automation markers (`cdc_`, `$cdc_`, `__webdriver`, etc.)
- Sets `navigator.webdriver = false`
- Replaces "HeadlessChrome" with "Chrome" in the UA

### JavaScript stealth injected by bootstrap script

| Property | Behavior |
|---|---|
| `navigator.webdriver` | `false` (set on prototype, survives `getOwnPropertyNames`) |
| `navigator.platform` | Matches profile OS (e.g. `"Win32"`, `"MacIntel"`) |
| `navigator.hardwareConcurrency` | Profile CPU cores |
| `navigator.deviceMemory` | Profile RAM (spec-valid discrete value) |
| `navigator.userAgentData` | Full UA-CH object with brands, `getHighEntropyValues()` |
| `navigator.plugins` | 5-plugin fake set |
| WebGL vendor/renderer | Profile GPU strings |
| `window.chrome` | Full runtime object with `connect()`, `sendMessage()`, `csi()`, `loadTimes()`, `app` |
| CDP markers | Removed from `window` |

**Tested against**: Cloudflare Turnstile, Cloudflare WAF, bot.sannysoft.com, areyouheadless, deviceandbrowserinfo.com, CreepJS

## Technical Comparison

| Metric | chaser-oxide | Node.js Alternatives |
|---|---|---|
| **Language** | Rust | JavaScript |
| **Memory Footprint** | ~50–100MB | ~500MB+ |
| **Transport Patching** | Protocol-level (internal fork) | High-level (wrapper/plugin) |
| **Default Profile** | Native host OS + Chrome version | Usually hardcoded |

## Dependencies

- [chromiumoxide](https://github.com/mattsse/chromiumoxide) — Base CDP client (forked)
- [tokio](https://tokio.rs) — Async runtime
- [futures](https://docs.rs/futures) — Async utilities

## Acknowledgements

This project is a specialized fork of **[chromiumoxide](https://github.com/mattsse/chromiumoxide)**. The core CDP client and session management are derived from their excellent work.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
