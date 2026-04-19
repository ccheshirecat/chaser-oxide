use anyhow::Result;
use chaser_oxide::{Browser, BrowserConfig, ChaserPage};
use futures::StreamExt;
use serde_json::Value;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 HEADLESS BOT DETECTION TEST");
    println!("================================\n");

    // Launch HEADLESS browser (use new_headless_mode())
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .new_headless_mode() // NEW headless mode (more realistic)
            .build()
            .map_err(|e| anyhow::anyhow!("{}", e))?,
    )
    .await?;

    tokio::spawn(async move { while let Some(_) = handler.next().await {} });

    // Create page and apply native profile (reads actual Chrome version via CDP,
    // detects host OS and RAM — no hardcoded Windows/version spoofing)
    let page = browser.new_page("about:blank").await?;
    let chaser = ChaserPage::new(page);
    chaser.apply_native_profile().await?;
    println!("✅ Native profile applied in HEADLESS mode\n");

    // ========== TEST 1: Sannysoft Bot Detection ==========
    println!("📊 TEST 1: Sannysoft Bot Detector");
    println!("   URL: https://bot.sannysoft.com");
    chaser.goto("https://bot.sannysoft.com").await?;
    tokio::time::sleep(Duration::from_secs(4)).await;

    // Count red/green flags
    let red_count: u32 = extract_number(
        &chaser
            .evaluate(
                r#"
        (() => {
            const rows = Array.from(document.querySelectorAll('tr td'));
            return rows.filter(cell =>
                cell.style.color === 'red' || cell.className.includes('failed')
            ).length;
        })()
    "#,
            )
            .await?,
    );

    let green_count: u32 = extract_number(
        &chaser
            .evaluate(
                r#"
        (() => {
            const rows = Array.from(document.querySelectorAll('tr td'));
            return rows.filter(cell =>
                cell.style.color === 'green' || cell.className.includes('passed')
            ).length;
        })()
    "#,
            )
            .await?,
    );

    println!("   🚩 Red flags: {}", red_count);
    println!("   ✅ Green flags: {}", green_count);

    // Check specific items
    let webdriver: String = extract_string(&chaser.evaluate("String(navigator.webdriver)").await?);
    println!("   navigator.webdriver: {}", webdriver);

    let chrome_check: String = extract_string(&chaser.evaluate("String(!!window.chrome)").await?);
    println!("   window.chrome exists: {}", chrome_check);

    println!();

    // ========== TEST 2: Are You Headless ==========
    println!("📊 TEST 2: AreYouHeadless Detection");
    println!("   URL: https://arh.antoinevastel.com/bots/areyouheadless");
    chaser
        .goto("https://arh.antoinevastel.com/bots/areyouheadless")
        .await?;
    tokio::time::sleep(Duration::from_secs(3)).await;

    let headless_result = extract_string(
        &chaser
            .evaluate(
                r#"
        (() => {
            const pre = document.querySelector('pre');
            return pre ? pre.textContent : 'Loading...';
        })()
    "#,
            )
            .await?,
    );

    println!("   Result: {}", headless_result);
    println!();

    // ========== TEST 3: Winna.com Turnstile ==========
    println!("📊 TEST 3: Cloudflare Turnstile (winna.com)");
    println!("   URL: https://winna.com");
    chaser.goto("https://winna.com").await?;
    tokio::time::sleep(Duration::from_secs(6)).await;

    let turnstile_status = extract_string(
        &chaser
            .evaluate(
                r#"
        (() => {
            const widget = document.querySelector('iframe[src*="turnstile"]');
            if (!widget) return 'No Turnstile widget found';

            const parent = widget.parentElement;
            if (!parent) return 'Widget found but no parent';

            // Check for success class
            if (parent.className.includes('success') ||
                parent.getAttribute('data-theme') === 'success') {
                return 'PASSED ✅';
            }

            // Check for error
            const error = document.querySelector('.cf-error-details');
            if (error) return 'FAILED: ' + error.textContent;

            return 'LOADING or UNKNOWN';
        })()
    "#,
            )
            .await?,
    );

    println!("   Status: {}", turnstile_status);
    println!();

    // ========== Test 4: DeviceAndBrowserInfo Bot Detection ==========

    println!("📊 TEST 4: DeviceAndBrowserInfo Bot Detection");
    println!("   URL: https://deviceandbrowserinfo.com/are_you_a_bot");
    chaser
        .goto("https://deviceandbrowserinfo.com/are_you_a_bot")
        .await?;
    tokio::time::sleep(Duration::from_secs(4)).await;
    let result = chaser.raw_page().find_element("#resultsBotTest").await;
    match result {
        Ok(element) => {
            let text = element
                .inner_text()
                .await?
                .unwrap_or_else(|| "❓Unknown".to_string());
            println!("   Detection Result: {}", text.replace("\n", ""));
        }
        Err(_) => {
            println!("   Detection Result: element not found.");
        }
    }

    println!();

    // ========== TEST 5: Manual Stealth Checks ==========
    println!("📊 TEST 5: Manual Stealth Checks");

    let ua = extract_string(&chaser.evaluate("navigator.userAgent").await?);
    println!("   User-Agent: {}", ua.chars().take(80).collect::<String>());

    let platform = extract_string(&chaser.evaluate("navigator.platform").await?);
    println!("   Platform: {}", platform);

    let cores = extract_number(&chaser.evaluate("navigator.hardwareConcurrency").await?);
    println!("   Hardware Concurrency: {}", cores);

    let memory = extract_number(&chaser.evaluate("navigator.deviceMemory").await?);
    println!("   Device Memory: {}GB", memory);

    let plugins = extract_number(&chaser.evaluate("navigator.plugins.length").await?);
    println!("   Plugins: {}", plugins);

    let languages = extract_string(
        &chaser
            .evaluate("JSON.stringify(navigator.languages)")
            .await?,
    );
    println!("   Languages: {}", languages);

    // Check for CDP markers
    let cdp_count = extract_number(
        &chaser
            .evaluate(
                r#"
        (() => {
            let count = 0;
            for (const prop of Object.getOwnPropertyNames(window)) {
                if (/^cdc_|^\$cdc_|^__webdriver|^__selenium|^__driver|^\$chrome_/.test(prop)) {
                    count++;
                }
            }
            return count;
        })()
    "#,
            )
            .await?,
    );
    println!("   CDP markers found: {}", cdp_count);

    // Check chrome APIs
    let chrome_runtime = extract_string(
        &chaser
            .evaluate("String(!!window.chrome?.runtime?.connect)")
            .await?,
    );
    println!("   chrome.runtime.connect: {}", chrome_runtime);

    let chrome_csi = extract_string(&chaser.evaluate("String(!!window.chrome?.csi)").await?);
    println!("   chrome.csi: {}", chrome_csi);

    println!();

    // ========== SUMMARY ==========
    println!("================================");
    println!("🎯 TEST SUMMARY");
    println!("================================");

    let webdriver_clean = webdriver == "false";
    let chrome_present = chrome_check == "true";
    let low_red_flags = red_count < 5;
    let no_cdp_markers = cdp_count == 0;

    println!(
        "✓ navigator.webdriver = false: {}",
        if webdriver_clean { "✅" } else { "❌" }
    );
    println!(
        "✓ window.chrome present: {}",
        if chrome_present { "✅" } else { "❌" }
    );
    println!(
        "✓ Low red flags (<5): {}",
        if low_red_flags { "✅" } else { "❌" }
    );
    println!(
        "✓ No CDP markers: {}",
        if no_cdp_markers { "✅" } else { "❌" }
    );

    let score = [
        webdriver_clean,
        chrome_present,
        low_red_flags,
        no_cdp_markers,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    println!("\nOverall Score: {}/4", score);

    if score >= 3 {
        println!("🎉 HEADLESS STEALTH: EXCELLENT");
    } else if score >= 2 {
        println!("⚠️  HEADLESS STEALTH: GOOD");
    } else {
        println!("❌ HEADLESS STEALTH: NEEDS IMPROVEMENT");
    }

    Ok(())
}

fn extract_string(value: &Option<Value>) -> String {
    match value {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Null) => "null".to_string(),
        Some(v) => v.to_string(),
        None => "undefined".to_string(),
    }
}

fn extract_number(value: &Option<Value>) -> u32 {
    match value {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0) as u32,
        Some(Value::String(s)) => s.parse().unwrap_or(0),
        _ => 0,
    }
}
