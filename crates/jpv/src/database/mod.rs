use std::mem;

pub(crate) use self::r#impl::{open, Data};

#[cfg(all(unix, feature = "mmap"))]
#[path = "mmap.rs"]
mod r#impl;

#[cfg(any(not(unix), not(feature = "mmap")))]
#[path = "buf.rs"]
mod r#impl;

use anyhow::Result;

use crate::dirs::Dirs;
use crate::Args;

use lib::database::Location;

static mut GUARDS: Vec<Data> = Vec::new();

/// Open a database using the default method based on current arguments and directories.
pub(crate) unsafe fn open_from_args(
    args: &Args,
    dirs: &Dirs,
) -> Result<Vec<(&'static [u8], Location)>> {
    let mut indexes = Vec::new();
    let found;

    let paths = match &args.index[..] {
        [] => {
            found = dirs.indexes()?;
            &found[..]
        }
        rest => rest,
    };

    for path in paths {
        let data = r#impl::open(path)?;
        let slice = mem::transmute(data.as_slice());
        GUARDS.push(data);
        indexes.push((slice, Location::Path(path.as_path().into())));
    }

    Ok(indexes)
}
