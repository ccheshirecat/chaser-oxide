use chromiumoxide_fetcher::{
    BrowserFetcherOptions, BrowserHost, BrowserKind, BrowserVersion, BuildInfo, Platform, Revision,
};
use reqwest::{IntoUrl, Response, StatusCode};
use tokio::process::Command;

pub async fn head<T: IntoUrl>(url: T) -> reqwest::Result<Response> {
    reqwest::Client::builder().build()?.head(url).send().await
}

// Check if the chosen revision has a build available for all platforms.
// That not always the case, that is why we need to make sure of it.
#[tokio::test]
async fn verify_chromium_revision_available() {
    let host = BrowserHost::current(BrowserKind::Chromium);
    let BrowserVersion::Revision(revision) = BrowserVersion::current(BrowserKind::Chromium) else {
        panic!("Chromium revision is not available");
    };
    let build_info = BuildInfo::revision(revision);
    for platform in Platform::all() {
        let res = head(&BrowserKind::Chromium.download_url(*platform, &build_info, &host))
            .await
            .unwrap();

        if res.status() != StatusCode::OK {
            panic!("Revision {revision} is not available for {platform}");
        }
    }
}

#[ignore]
#[tokio::test]
async fn find_chromium_revision_available() {
    let min = 1583927; // Enter the minimum revision
    let max = 1586699; // Enter the maximum revision

    let host = BrowserHost::current(BrowserKind::Chromium);
    'outer: for revision in (min..=max).rev() {
        println!("Checking revision {}", revision);

        let build_info = BuildInfo::revision(Revision::from(revision));
        for platform in Platform::all() {
            let res = head(&BrowserKind::Chromium.download_url(*platform, &build_info, &host))
                .await
                .unwrap();

            if res.status() != StatusCode::OK {
                println!("Revision {revision} is not available for {platform}");
                continue 'outer;
            }
        }

        println!("Found revision {revision}");
        break;
    }
}

#[ignore]
#[tokio::test]
async fn download_chromium_revision() {
    let path = "./.cache";

    tokio::fs::create_dir_all(path).await.unwrap();

    for platform in Platform::all() {
        let revision = chromiumoxide_fetcher::BrowserFetcher::new(
            BrowserFetcherOptions::builder()
                .with_kind(BrowserKind::Chromium)
                .with_path(path)
                .with_platform(*platform)
                .build()
                .unwrap(),
        )
        .fetch()
        .await
        .unwrap();

        println!("Downloaded revision {revision} for {platform}");
    }
}

#[tokio::test]
async fn test_chromium() {
    let path = "./.cache";

    tokio::fs::create_dir_all(path).await.unwrap();

    // Download the browser
    let revision = chromiumoxide_fetcher::BrowserFetcher::new(
        BrowserFetcherOptions::builder()
            .with_kind(BrowserKind::Chromium)
            .with_path(path)
            .build()
            .unwrap(),
    )
    .fetch()
    .await
    .unwrap();

    println!(
        "Launching browser from {}",
        revision.executable_path.display()
    );

    // Launch the browser
    let mut child = Command::new(&revision.executable_path)
        .spawn()
        .expect("Failed to start Chrome executable");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    child.kill().await.expect("Failed to kill Chrome process");
}
