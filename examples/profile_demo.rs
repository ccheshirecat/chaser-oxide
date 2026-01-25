//! Example demonstrating the ChaserProfile builder system.
//!
//! Shows how to create customized browser fingerprint profiles
//! with the ergonomic builder pattern.

use anyhow::Result;
use chaser_oxide::{Browser, BrowserConfig, ChaserPage, ChaserProfile, Gpu};
use futures::StreamExt;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // === Profile Builder Demo ===

    // Quick preset: Windows gamer with RTX 3080
    let windows_profile = ChaserProfile::windows().build();
    println!("Windows Profile:");
    println!("  UA: {}", windows_profile.user_agent());
    println!("  Platform: {}", windows_profile.os().platform());
    println!("  GPU: {}", windows_profile.gpu().renderer());

    // Customized: High-end Windows workstation
    let custom_profile = ChaserProfile::windows()
        .chrome_version(130)
        .gpu(Gpu::NvidiaRTX4080)
        .memory_gb(32)
        .cpu_cores(16)
        .locale("de-DE")
        .timezone("Europe/Berlin")
        .build();

    println!("\nCustom Profile:");
    println!("  UA: {}", custom_profile.user_agent());
    println!("  Memory: {}GB", custom_profile.memory_gb());
    println!("  Cores: {}", custom_profile.cpu_cores());
    println!("  Locale: {}", custom_profile.locale());

    // macOS profile
    let mac_profile = ChaserProfile::macos_arm()
        .gpu(Gpu::AppleM4Max)
        .memory_gb(64)
        .build();

    println!("\nmacOS Profile:");
    println!("  UA: {}", mac_profile.user_agent());
    println!("  GPU: {}", mac_profile.gpu().renderer());

    // === Live Test with Profile ===
    println!("\n=== Starting Browser Test ===");

    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .viewport(None)
            .build()
            .map_err(|e| anyhow::anyhow!(e))?,
    )
    .await?;

    tokio::spawn(async move { while let Some(_) = handler.next().await {} });

    // Create page with stealth
    let page = browser.new_page("about:blank").await?;
    page.enable_stealth_mode().await?;

    tokio::time::sleep(Duration::from_millis(100)).await;
    page.goto("https://bot.sannysoft.com").await?;

    tokio::time::sleep(Duration::from_secs(3)).await;

    let chaser = ChaserPage::new(page);
    let mut rng = rand::rng();

    // Demonstrate click_human (combines bezier + click)
    println!("\nTesting click_human()...");
    chaser.click_human(400.0, 300.0, &mut rng).await?;

    // Demonstrate type_text_with_typos
    println!("Testing type_text_with_typos()...");
    // chaser.type_text_with_typos("Hello world", &mut rng).await?;

    tokio::time::sleep(Duration::from_secs(3)).await;

    println!("\n✅ Profile system demo complete!");

    Ok(())
}
