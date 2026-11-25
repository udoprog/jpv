//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/jpv-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/jpv)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-jpv--tesseract-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/jpv-tesseract)
//! [<img alt="chat on discord" src="https://img.shields.io/discord/558644981137670144.svg?logo=discord&style=flat-square" height="20">](https://discord.gg/v5AeNkT)
//!
//! Bindings to tesseract.
//!
//! The backend depends on the platform:
//! * On Windows tesseract is loaded at runtime from the [UB-Mannheim/tesseract]
//!   installation found in the registry.
//! * On Linux and macOS with the `linked` feature tesseract is linked
//!   when building.
//! * On Linux and macOS without it tesseract is loaded at runtime by
//!   its library name, so a binary built this way starts whether tesseract is
//!   installed or not, and only has OCR when it is.
//!
//! Where tesseract cannot be loaded, `open` returns an error.
//!
//! [UB-Mannheim/tesseract]: https://github.com/UB-Mannheim/tesseract/wiki

/// Result alias for this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub use self::error::Error;
mod error;

#[cfg(any(windows, all(unix, not(feature = "linked"))))]
mod dynamic;

// Cargo cannot leave out a dependency when a feature is enabled, so libloading
// is still built with `linked` even though only the runtime-loaded backend uses
// it.
#[cfg(all(unix, feature = "linked"))]
use libloading as _;

#[cfg_attr(windows, path = "dll.rs")]
#[cfg_attr(all(unix, feature = "linked"), path = "linked.rs")]
#[cfg_attr(all(unix, not(feature = "linked")), path = "unix.rs")]
#[cfg_attr(not(any(unix, windows)), path = "fake.rs")]
mod r#impl;
pub use self::r#impl::{open, Tesseract, TesseractString};
