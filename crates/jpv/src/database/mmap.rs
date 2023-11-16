use std::fs::File;
use std::io;
use std::path::Path;

use memmap::MmapOptions;

pub struct Data {
    map: memmap::Mmap,
}

impl Data {
    /// Get a slice to the underlying data.
    pub fn as_slice(&self) -> &[u8] {
        &self.map[..]
    }
}

pub(crate) unsafe fn open<P>(path: P) -> io::Result<Data>
where
    P: AsRef<Path>,
{
    let f = File::open(path)?;
    let mmap = MmapOptions::new().map(&f)?;
    Ok(Data { map: mmap })
}
