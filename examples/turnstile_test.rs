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
    println!("Navigating to rebrowser bot detector...");
    chaser.goto("https://bot-detector.rebrowser.net/").await?;

    // Wait for page to fully load
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Human-like mouse movement
    println!("Simulating human mouse movement...");
    chaser.move_mouse_human(500.0, 300.0).await?;

    // ========== TRIGGER REBROWSER TESTS ==========
    println!("\nTriggering rebrowser detection tests...");
    
    // Test 1: dummyFn - tests main world access
    println!("  Testing dummyFn (main world access)...");
    let dummy_result = chaser.evaluate("typeof window.dummyFn === 'function' ? window.dummyFn() : 'no dummyFn'").await?;
    println!("    Result: {:?}", dummy_result);
    
    // Test 2: sourceUrlLeak - tests for pptr: or playwright: sourceURL
    println!("  Testing sourceUrlLeak...");
    let _ = chaser.evaluate("document.getElementById('detections-json')?.textContent || 'no element'").await?;
    
    // Test 3: mainWorldExecution - triggers if our code runs in main world
    println!("  Testing mainWorldExecution...");
    let _ = chaser.evaluate("document.getElementsByClassName('div').length").await?;

    // Wait for tests to complete
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Read the JSON results
    println!("\n========== REBROWSER BOT DETECTOR RESULTS ==========");
    let results = chaser.evaluate(r#"
        const json = document.getElementById('detections-json');
        json ? json.textContent : 'Results not found'
    "#).await?;
    
    // results is Option<Value>
    if let Some(val) = results {
        if let Some(json_str) = val.as_str() {
            println!("{}", json_str);
        } else {
            println!("Results: {:?}", val);
        }
    } else {
        println!("No results returned");
    }

    println!("\n=====================================================");
    println!("Check browser window for visual results.");
    println!("Press Ctrl+C to exit...");

    // Keep browser open for inspection
    tokio::time::sleep(Duration::from_secs(300)).await;

    Ok(())
}
