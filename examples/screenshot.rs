use chaser_oxide::browser::{Browser, BrowserConfig};
use chaser_oxide::page::ScreenshotParams;
use chromiumoxide_cdp::cdp::browser_protocol::page::CaptureScreenshotFormat;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (browser, mut handler) = Browser::launch(BrowserConfig::builder().build()?).await?;

    let handle = tokio::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    let page = browser.new_page("https://news.ycombinator.com/").await?;

    // take a screenshot of the page
    page.save_screenshot(
        ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .full_page(true)
            .omit_background(true)
            .build(),
        "hn-page.png",
    )
    .await?;

    // get the top post and save a screenshot of it
    page.find_element("table.itemlist tr")
        .await?
        .save_screenshot(CaptureScreenshotFormat::Png, "top-post.png")
        .await?;

    handle.await?;
    Ok(())
}
