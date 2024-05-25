// Use a better method for launching URIs on GIO-enabled platforms like GNOME.
#[cfg(all(unix, feature = "gio"))]
pub(crate) fn open(uri: &str) {
    let _ = gio::AppInfo::launch_default_for_uri(uri, gio::AppLaunchContext::NONE);
}

/// Fallback to webbrowser.
///
/// Preferably we don't want to use this where a better alternative is available,
/// because it spawns the browser as a child process to the current one which
/// does not detach itself.
#[cfg(not(all(unix, feature = "gio")))]
pub(crate) fn open(uri: &str) {
    let _ = webbrowser::open(uri);
}
