use std::fs::File;
use std::io;
use std::path::Path;

use memmap::MmapOptions;
use musli_zerocopy::Buf;

pub struct Data {
    map: memmap::Mmap,
}

impl Data {
    /// Get a slice to the underlying data.
    pub fn as_buf(&self) -> &Buf {
        Buf::new(&self.map[..])
    }
}

/// Open the given path as data.
pub fn open<P>(path: P) -> io::Result<Data>
where
    P: AsRef<Path>,
{
    let f = File::open(path)?;
    let mmap = unsafe { MmapOptions::new().map(&f)? };
    Ok(Data { map: mmap })
}
