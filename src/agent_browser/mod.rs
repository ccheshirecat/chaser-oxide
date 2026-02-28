//! Agent-Browser integration module for AI-first browser automation.
//!
//! This module provides compatibility with the [agent-browser](https://agent-browser.dev)
//! CLI interface, enabling AI agents to interact with web pages using a streamlined
//! command protocol and the innovative Snapshot + Refs system.
//!
//! # Key Features
//!
//! - **Snapshot + Refs System**: Dramatically reduces AI context usage by up to 93%
//!   by providing element references (`@e1`, `@e2`) instead of full DOM trees.
//! - **Semantic Locators**: Find elements by role, text, label, placeholder, etc.
//! - **108+ Commands**: Full feature parity with agent-browser CLI.
//! - **JSON Protocol**: Machine-readable command/response format for AI agents.
//! - **Tab Management**: Multi-tab/window support via AgentBrowser.
//! - **Recording & Tracing**: Trace/HAR recording for debugging and replay.
//! - **Session Management**: Isolated sessions with state persistence.
//! - **CLI Interface**: JSON-over-stdin protocol for AI agent integration.
//!
//! # Example
//!
//! ```no_run
//! use chaser_oxide::agent_browser::{AgentPage, SnapshotOptions};
//! use chaser_oxide::{Browser, BrowserConfig, ChaserPage};
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let (browser, mut handler) = Browser::launch(BrowserConfig::builder().build()?).await?;
//! tokio::spawn(async move { while handler.next().await.is_some() {} });
//!
//! let page = browser.new_page("about:blank").await?;
//! let chaser = ChaserPage::new(page);
//! let mut agent = AgentPage::new(chaser);
//!
//! // Navigate to a page
//! agent.navigate("https://example.com").await?;
//!
//! // Get a snapshot with refs - perfect for AI consumption
//! let snapshot = agent.snapshot(SnapshotOptions::default().interactive_only()).await?;
//! println!("{}", snapshot.tree);
//!
//! // Click using a ref from the snapshot
//! agent.click("@e1").await?;
//!
//! // Or use semantic locators
//! agent.click_by_role("button", Some("Submit")).await?;
//! # Ok(())
//! # }
//! ```

// Core modules
mod commands;
mod locator;
mod refs;
mod response;
mod snapshot;

// Phase modules
pub mod agent_page;
pub mod browser_manager;
pub mod cli;
pub mod clipboard;
pub mod perf;
pub mod raw_input;
pub mod recording;
pub mod session;
pub mod streaming;

// Re-export main types
pub use agent_page::AgentPage;
pub use browser_manager::AgentBrowser;
pub use cli::{CliCommand, CliResponse};
pub use clipboard::{ClipboardAction, ClipboardResult};
pub use commands::*;
pub use locator::{Locator, LocatorStrategy};
pub use perf::{PerfTracker, SnapshotCache};
pub use recording::{HarLog, HarRecorder, TraceEntry, TraceRecorder};
pub use refs::{RefId, RefInfo, RefMap};
pub use response::{AgentError, AgentResponse, AgentResult};
pub use session::{CloudProvider, LaunchOptions, SessionConfig, SessionState};
pub use snapshot::{AccessibilityNode, Snapshot, SnapshotOptions};
pub use streaming::{ScreencastController, ScreencastFrame, StreamConfig};
