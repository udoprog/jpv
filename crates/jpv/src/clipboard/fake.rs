use anyhow::Result;
use tokio::sync::watch;

use crate::system::SystemEvents;

/// Clipboard capture is not supported on this platform, or without the
/// `wayland` feature.
pub(crate) async fn run(_: watch::Receiver<bool>, _: SystemEvents) -> Result<()> {
    tracing::debug!("Clipboard capture is not supported by this build");
    Ok(())
}
