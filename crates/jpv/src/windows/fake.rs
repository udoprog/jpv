use anyhow::Result;

use crate::system::Setup;

pub fn setup() -> Result<Setup> {
    Ok(Setup::Start(None))
}
