#[cfg(feature = "zip0")]
mod zip0;
#[cfg(feature = "zip0")]
pub use zip0::ZipArchive;

#[cfg(feature = "zip8")]
mod zip8;
#[cfg(feature = "zip8")]
pub use zip8::ZipArchive;
