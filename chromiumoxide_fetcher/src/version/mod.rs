use std::str::FromStr;

pub use self::channel::Channel;
use self::error::Result;
pub use self::error::VersionError;
pub use self::milestone::Milestone;
pub use self::revision::Revision;
pub use self::version::Version;
use crate::{BrowserHost, BrowserKind, BuildInfo, Platform};

mod channel;
mod error;
mod metadata;
mod milestone;
mod revision;
#[allow(clippy::module_inception)]
mod version;

/// Represents a version of a browser.
///
/// The version can be a channel, a revision, a build, or a milestone.
/// Not all combinations are valid for all browser kinds.
#[derive(Clone, Copy, Debug)]
pub enum BrowserVersion {
    Channel(Channel),
    Revision(Revision),
    Version(Version),
    Milestone(Milestone),
}

impl BrowserVersion {
    #[doc(hidden)] // internal API
    pub fn current(kind: BrowserKind) -> Self {
        // The chromium revision is hard to get right and the relation to the CDP revision
        // even more so, so here are some guidances.
        //
        // We used to use the revision of Puppeteer, but they switched to chrome-for-testing.
        // This means we have to check things ourself. The chromium revision should at least
        // as great as the CDP revision otherwise they won't be compatible.
        // Not all revisions of chromium have builds for all platforms.
        //
        // This is essentially a bruteforce process. You can use the test `find_revision_available`
        // to find a revision that is available for all platforms. We recommend setting the `min`
        // to the current CDP revision and the max to max revision of stable chromium.
        // See https://chromiumdash.appspot.com/releases for the latest stable revision.
        //
        // In general, we should also try to ship as close as a stable version of chromium if possible.
        // The CDP should also be a bit older than that stable version.
        // To map a revision to a chromium version you can use the site https://chromiumdash.appspot.com/commits.

        match kind {
            BrowserKind::Chromium => Self::Revision(Revision::new(1585606)),
            BrowserKind::Chrome => Self::Channel(Channel::Stable),
            BrowserKind::ChromeHeadlessShell => Self::Channel(Channel::Stable),
        }
    }

    pub(crate) async fn resolve(
        &self,
        kind: BrowserKind,
        platform: Platform,
        host: &BrowserHost,
    ) -> Result<BuildInfo> {
        match self {
            Self::Revision(revision) => revision.resolve(kind, host).await,
            Self::Channel(channel) => channel.resolve(kind, platform, host).await,
            Self::Version(build) => build.resolve(kind, host).await,
            Self::Milestone(milestone) => milestone.resolve(kind, host).await,
        }
    }
}

impl FromStr for BrowserVersion {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(revision) = s.parse::<Revision>() {
            return Ok(Self::Revision(revision));
        }

        if let Ok(channel) = s.parse::<Channel>() {
            return Ok(Self::Channel(channel));
        }

        if let Ok(build) = s.parse::<Version>() {
            return Ok(Self::Version(build));
        }

        if let Ok(milestone) = s.parse::<Milestone>() {
            return Ok(Self::Milestone(milestone));
        }

        Err(VersionError::InvalidVersion(s.to_string()))
    }
}

impl TryFrom<String> for BrowserVersion {
    type Error = VersionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<Channel> for BrowserVersion {
    fn from(channel: Channel) -> Self {
        Self::Channel(channel)
    }
}

impl From<Revision> for BrowserVersion {
    fn from(revision: Revision) -> Self {
        Self::Revision(revision)
    }
}

impl From<Version> for BrowserVersion {
    fn from(build: Version) -> Self {
        Self::Version(build)
    }
}

impl From<Milestone> for BrowserVersion {
    fn from(milestone: Milestone) -> Self {
        Self::Milestone(milestone)
    }
}
