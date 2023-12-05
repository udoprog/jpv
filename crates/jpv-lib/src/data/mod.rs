//! Helper to open paths as [`Data`].

pub(crate) use self::r#impl::Data;

#[cfg(feature = "memmap")]
#[path = "mmap.rs"]
mod r#impl;

#[cfg(not(feature = "memmap"))]
#[path = "buf.rs"]
mod r#impl;

pub use self::r#impl::open;

use std::path::PathBuf;

use anyhow::Result;

use crate::database::Location;
use crate::dirs::Dirs;

/// Open a database using the default method based on current arguments and directories.
pub fn open_from_args(indexes: &[PathBuf], dirs: &Dirs) -> Result<Vec<(Data, Location)>> {
    let found;

    let paths = match indexes {
        [] => {
            found = dirs.indexes()?;
            &found[..]
        }
        rest => rest,
    };

    let mut output = Vec::new();

    for path in paths {
        let data = r#impl::open(path)?;
        output.push((data, Location::Path(path.as_path().into())));
    }

    Ok(output)
}
