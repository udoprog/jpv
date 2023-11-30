use anyhow::{bail, Result};

use crate::command::service::ServiceArgs;
use crate::system::Setup;

pub(crate) async fn send_clipboard(_: Option<&str>, _: &[u8]) -> Result<()> {
    bail!("Sending the clipboard is not supported")
}

pub(crate) async fn shutdown() -> Result<()> {
    Ok(())
}

pub(crate) async fn setup(_: &ServiceArgs) -> Result<Setup> {
    Ok(Setup::Start(None))
}
