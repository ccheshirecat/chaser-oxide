use anyhow::Result;
use chaser_oxide::{ChaserPage, Os};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Launching chaser-oxide Stealth Browser...");
    
    // ONE LINE - handles everything: window size, stealth args, profile
    let (_browser, chaser) = ChaserPage::launch_headed(Os::Windows).await?;

    // Small delay to ensure scripts are registered
    tokio::time::sleep(Duration::from_millis(100)).await;

    // NOW navigate to the detection test
    println!("Navigating to detection test...");
    chaser.goto("https://turnstile-demo.pages.dev").await?;

    // Wait for page to fully load
    tokio::time::sleep(Duration::from_secs(3)).await;

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
    tokio::time::sleep(Duration::from_secs(60)).await;

    chaser
        .raw_page()
        .save_screenshot(
            chaser_oxide::page::ScreenshotParams::builder().build(),
            "fingerprint_test.png",
        )
        .await?;
    println!("Screenshot saved to stealth_test.png");

    println!("\nBrowser will close in 5 seconds...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}
