use anyhow::Result;
use chaser_oxide::{Browser, BrowserConfig, ChaserPage};
use futures::StreamExt;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Launching chaser-oxide Stealth Browser...");
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .viewport(None)
            .build()
            .map_err(|e| anyhow::anyhow!(e))?,
    )
    .await?;

    tokio::spawn(async move { while let Some(_) = handler.next().await {} });

    // CRITICAL: Create page with about:blank FIRST
    println!("Creating page...");
    let page = browser.new_page("about:blank").await?;

    // Apply stealth patches BEFORE navigation
    // This registers the scripts for all future document loads
    println!("Applying stealth patches...");
    page.enable_stealth_mode().await?;

    // Small delay to ensure scripts are registered
    tokio::time::sleep(Duration::from_millis(100)).await;

    // NOW navigate to the detection test
    println!("Navigating to detection test...");
    page.goto("https://bot.sannysoft.com").await?;

    // Wait for page to fully load
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Upgrade to ChaserPage
    let chaser = ChaserPage::new(page);

    // Human-like mouse movement
    println!("Simulating human mouse movement...");
    chaser.move_mouse_human(500.0, 300.0).await?;

    // Test stealth execution
    println!("\nReading values from the PAGE (main world sees spoofed values):");

    // Read what the site's JavaScript sees
    let user_agent = chaser.evaluate_stealth("navigator.userAgent").await?;
    println!("  navigator.userAgent = {:?}", user_agent);

    // Wait and take screenshot
    println!("\nWaiting for page to render...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    chaser
        .raw_page()
        .save_screenshot(
            chaser_oxide::page::ScreenshotParams::builder().build(),
            "stealth_test.png",
        )
        .await?;
    println!("Screenshot saved to stealth_test.png");

    println!("\nBrowser will close in 5 seconds...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}
