#[cfg(feature = "bundle-database")]
#[path = "bundle.rs"]
mod r#impl;

#[cfg(not(feature = "bundle-database"))]
#[path = "file_system.rs"]
mod r#impl;

pub(crate) use self::r#impl::*;
