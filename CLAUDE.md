# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

chaser-oxide is a Rust-based fork of `chromiumoxide` specialized for hardened browser automation. It provides protocol-level stealth modifications to the Chrome DevTools Protocol (CDP) client to reduce the detection footprint of automated browser sessions.

## Build & Test Commands

```bash
# Build the project
cargo build

# Run unit tests (library only)
cargo test --lib

# Run integration tests (requires Chrome/Chromium installed)
RUST_TEST_THREADS=1 cargo test --test '*'

# Run a specific test
cargo test --lib test_name

# Format code
cargo fmt

# Run lints (CI requires no warnings)
cargo clippy --all -- -D warnings

# Check examples compile
cargo check --examples --features tokio-runtime,bytes

# Run an example
cargo run --example stealth_bot
cargo run --example profile_demo
```

## Workspace Structure

This is a Cargo workspace with multiple crates:

- **`chaser-oxide`** (root) - Main library with stealth automation API
- **`chromiumoxide_cdp`** - Generated CDP protocol definitions (~60K lines, generated at build time)
- **`chromiumoxide_pdl`** - PDL (Protocol Definition Language) parser for generating CDP bindings
- **`chromiumoxide_types`** - Shared types across crates
- **`chromiumoxide_fetcher`** - Optional browser binary fetcher

## Architecture

### Core API Layers

1. **`ChaserPage`** (`src/chaser.rs`) - High-level stealth wrapper around `Page`
   - Stealth JS execution via `Page.createIsolatedWorld` (avoids `Runtime.enable` detection)
   - Human-like input simulation (Bezier mouse curves, variable typing delays)
   - Request interception via Fetch domain
   - Access underlying `Page` via `raw_page()`

2. **`ChaserProfile`** (`src/profiles.rs`) - Browser fingerprint profiles
   - Builder pattern for customizing OS, GPU, hardware specs
   - Generates User-Agent and bootstrap JS script for spoofing
   - Presets: `windows()`, `linux()`, `macos_arm()`, `macos_intel()`

3. **`Page`** (`src/page.rs`) - Base CDP page abstraction (from chromiumoxide)
   - `enable_stealth_mode()` - Quick stealth setup with sensible defaults
   - Standard CDP operations (navigate, screenshot, cookies, etc.)

4. **`StealthProfile` trait** (`src/stealth.rs`) - Legacy trait-based profile system
   - Pre-built profiles: `WindowsNvidiaProfile`, `MacOSProfile`, `LinuxProfile`

### Key Design Decisions

- **Stealth execution**: `ChaserPage.evaluate()` uses `Page.createIsolatedWorld` to run JS without triggering `Runtime.enable`, which anti-bots detect. The regular `Page.evaluate()` triggers detection.

- **Profile consistency**: All fingerprint values (User-Agent, platform, WebGL, hardware) must be internally consistent. A Windows UA with MacOS platform is immediately flagged.

- **Properties on prototypes**: Stealth overrides are set on `Navigator.prototype` rather than `navigator` instance to avoid `getOwnPropertyNames` detection.

## Runtime Features

Default features use tokio. For async-std:
```bash
cargo build --features async-std-runtime --no-default-features
```

## Usage Pattern

```rust
// 1. Create profile
let profile = ChaserProfile::windows().chrome_version(130).build();

// 2. Launch browser
let (browser, mut handler) = Browser::launch(BrowserConfig::builder().build()?).await?;
tokio::spawn(async move { while let Some(_) = handler.next().await {} });

// 3. Create page and wrap
let page = browser.new_page("about:blank").await?;
let chaser = ChaserPage::new(page);

// 4. Apply profile BEFORE navigation
chaser.apply_profile(&profile).await?;

// 5. Navigate and interact
chaser.goto("https://example.com").await?;
let title: Option<Value> = chaser.evaluate("document.title").await?;
```

## Minimum Supported Rust Version

MSRV is 1.75 (checked in CI).
