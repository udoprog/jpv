/// Load the given path.
///
/// # Safety
///
/// Only one path may be loaded at a time, and the caller must ensure that they
/// only access it before any references that has been returned from it
/// previously are used.
pub(crate) use self::file_system::load_path;
mod file_system;

use std::fmt;
use std::path::Path;

use anyhow::Result;

use crate::dirs::Dirs;
use crate::Args;

/// Used for diagnostics to indicate where a dictionary was loaded from.
#[non_exhaustive]
pub(crate) enum Location {
    /// The dictionary was loaded from the given path.
    Path(Box<Path>),
    /// The dictionary was loaded from memory.
    #[allow(unused)]
    Memory(usize),
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Path(path) => path.display().fmt(f),
            Location::Memory(address) => write!(f, "<memory at {address:08x}>"),
        }
    }
}

/// Open a database using the default method based on current arguments and directories.
pub(crate) unsafe fn open(args: &Args, dirs: &Dirs) -> Result<(&'static [u8], Location)> {
    let path = match &args.dictionary {
        Some(path) => path.clone(),
        None => dirs.dictionary(),
    };

    let data = load_path(&path)?;
    Ok((data, Location::Path(path.into())))
}
