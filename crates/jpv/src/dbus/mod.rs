#[cfg(all(unix, feature = "dbus"))]
#[path = "real.rs"]
mod r#impl;

#[cfg(not(all(unix, feature = "dbus")))]
#[path = "fake.rs"]
mod r#impl;

pub(crate) use r#impl::shutdown;
pub(crate) use r#impl::{send_clipboard, setup};
