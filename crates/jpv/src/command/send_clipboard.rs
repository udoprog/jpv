use std::ffi::OsString;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub(crate) struct SendClipboardArgs {
    /// The mime type of the data to send.
    #[arg(long = "type", name = "type")]
    ty: Option<String>,
    /// The data to send.
    data: OsString,
}

pub(crate) fn run(args: &SendClipboardArgs) -> Result<()> {
    crate::dbus::send_clipboard(args.ty.as_deref(), &args.data)?;
    Ok(())
}
