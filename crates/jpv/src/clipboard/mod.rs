//! Capture the clipboard from inside the service.
//!
//! On Wayland compositors which support the `ext-data-control-v1` or
//! `zwlr-data-control-v1` protocols (such as KDE Plasma and wlroots-based
//! compositors) the service can watch the clipboard by itself, and forward each
//! new selection the same way as the D-Bus `SendClipboardData` method does.
//!
//! Elsewhere the watcher stays idle.

#[cfg(all(target_os = "linux", feature = "wayland"))]
#[path = "wayland.rs"]
mod r#impl;

#[cfg(not(all(target_os = "linux", feature = "wayland")))]
#[path = "fake.rs"]
mod r#impl;

pub(crate) use self::r#impl::run;
