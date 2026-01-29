use chaser_oxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (browser, mut handler) =
        Browser::launch(BrowserConfig::builder().with_head().build()?).await?;

    let handle = tokio::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    let page = browser.new_page("about:blank").await?;

    // Add the init script BEFORE navigation
    page.evaluate_on_new_document(
        r#"
        Object.defineProperty(navigator, 'webdriver', {
            get: () => undefined
        });
    "#,
    )
    .await?;

    // Navigate to a page
    page.goto("https://www.wikipedia.org")
        .await?
        .find_element("h1")
        .await?;

    let _html = page.wait_for_navigation().await?.content().await?;

    handle.await?;
    Ok(())
}
