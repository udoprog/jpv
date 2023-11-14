use anyhow::Result;
use tokio::sync::broadcast::Sender;
use tokio::sync::futures::Notified;

use crate::system::{Event, Setup};
use crate::Args;

pub(crate) fn setup<'a>(_: &Args, _: u16, _: Notified<'a>, _: Sender<Event>) -> Result<Setup<'a>> {
    Ok(Setup::Future(Box::pin(std::future::pending())))
}
