#[cfg(feature = "dbus")]
#[path = "real.rs"]
mod r#impl;

#[cfg(not(feature = "dbus"))]
#[path = "fake.rs"]
mod r#impl;

pub(crate) use r#impl::{send_clipboard, setup, shutdown};
