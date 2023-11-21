use anyhow::{bail, Result};
use tokio::sync::broadcast::Sender;
use tokio::sync::futures::Notified;

use crate::command::service::ServiceArgs;
use crate::system::{Event, Setup};

pub(crate) async fn send_clipboard(_: Option<&str>, _: &[u8]) -> Result<()> {
    bail!("Sending the clipboard is not supported")
}

pub(crate) async fn shutdown() -> Result<()> {
    Ok(())
}

pub(crate) async fn setup<'a>(
    _: &ServiceArgs,
    _: u16,
    _: Notified<'a>,
    _: Sender<Event>,
) -> Result<Setup<'a>> {
    Ok(Setup::Future(None))
}
