use anyhow::Result;
use std::future::Future;
use tokio::sync::futures::Notified;

use crate::system::Setup;

pub fn setup() -> Result<Setup> {
    Ok(Setup::Start(None))
}
