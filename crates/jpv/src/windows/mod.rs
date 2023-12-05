#[cfg(windows)]
#[path = "real.rs"]
mod r#impl;

#[cfg(not(windows))]
#[path = "fake.rs"]
mod r#impl;

pub(crate) use self::r#impl::setup;
