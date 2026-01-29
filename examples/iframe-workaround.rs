// This example is for checking the iframe workaround.
// a problem with the iframe workaround is that it will always fail to load the page
// and goto will cause a timeout.

use chaser_oxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (mut browser, mut handler) =
        Browser::launch(BrowserConfig::builder().with_head().build()?).await?;

    let handle = tokio::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    let page = browser
        .new_page("about:blank")
        .await
        .expect("failed to create page");

    let _ = page
        .goto("https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe")
        .await
        .expect("failed to navigate");

    browser.close().await?;
    handle.await?;

    Ok(())
}
