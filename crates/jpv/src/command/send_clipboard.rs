use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub(crate) struct SendClipboardArgs {
    /// The mime type of the data to send.
    #[arg(long = "type", name = "type")]
    ty: Option<String>,
    /// A secondary argument to send.
    #[arg(long)]
    secondary: Option<String>,
    /// The data to send.
    data: OsString,
}

pub(crate) async fn run(args: &SendClipboardArgs) -> Result<()> {
    match args.ty.as_deref() {
        Some("application/json") => {
            let json = lib::api::SendClipboardJson {
                primary: args.data.to_string_lossy().into_owned(),
                secondary: args.secondary.clone(),
            };

            let data = musli::storage::to_vec(&json)?;
            crate::dbus::send_clipboard(args.ty.as_deref(), &data).await?;
        }
        _ => {
            let data = to_bytes(&args.data);
            crate::dbus::send_clipboard(args.ty.as_deref(), data.as_ref()).await?;
        }
    }

    Ok(())
}

#[cfg(unix)]
fn to_bytes(data: &OsStr) -> Cow<'_, [u8]> {
    use std::os::unix::ffi::OsStrExt;
    Cow::Borrowed(data.as_bytes())
}

#[cfg(not(unix))]
fn to_bytes(data: &OsStr) -> Cow<'_, [u8]> {
    Cow::Owned(data.to_string_lossy().into_owned().into_bytes())
}
