use std::time::Duration;

use crate::test;
use chromiumoxide::browser::Browser;
use serde_json::Value;
use tokio::time::sleep;

#[tokio::test]
#[ignore] // Bot tests are flaky but a good reference
async fn bot_detection() {
    test(async |browser: &mut Browser| {
        let page = browser.new_page("about:blank").await.unwrap();
        page.enable_stealth_mode().await.unwrap();

        page.goto("https://bot.incolumitas.com").await.unwrap();

        sleep(Duration::from_secs(1)).await; // Wait 1 second to finish the tests

        let new_test_raw = page
            .find_element("#new-tests")
            .await
            .unwrap()
            .inner_text()
            .await
            .unwrap()
            .unwrap_or_else(|| "{}".to_string());

        let new_test_json: Value = serde_json::from_str(&new_test_raw).unwrap();

        let old_test_raw = page
            .find_element("#detection-tests")
            .await
            .unwrap()
            .inner_text()
            .await
            .unwrap()
            .unwrap_or_else(|| "{}".to_string());
        let old_test_json: Value = serde_json::from_str(&old_test_raw).unwrap();

        let new_failed: Vec<String> = new_test_json
            .as_object()
            .unwrap()
            .iter()
            .filter_map(|(k, v)| if v == "FAIL" { Some(k.clone()) } else { None })
            .collect();
        assert!(
            new_failed.is_empty(),
            "New test FAIL: {}",
            new_failed.join(", ")
        );

        let mut old_failed = Vec::new();
        for (group, checks) in old_test_json.as_object().unwrap() {
            for (key, value) in checks.as_object().unwrap() {
                if group == "fpscanner" && key == "WEBDRIVER" {
                    continue;
                } //false alarm
                if value == "FAIL" {
                    old_failed.push(format!("{}.{}", group, key));
                }
            }
        }
        assert!(
            old_failed.is_empty(),
            "Old test FAIL: {}",
            old_failed.join(", ")
        );
    })
    .await;
}
