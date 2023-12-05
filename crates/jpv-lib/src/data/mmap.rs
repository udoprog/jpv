use std::fs::OpenOptions;
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
    let mut options = OpenOptions::new();
    options.read(true);

    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // Allow the file to be deleted, which can be used to update the index
        // while the process is running. We also need shared reading, since we
        // open the same file multiple times as the database is being re-loaded.
        //
        // This is the default behavior on Linux.
        options.share_mode(4u32 | 1u32);
    }

    let f = options.open(path)?;
    let mmap = unsafe { MmapOptions::new().map(&f)? };
    Ok(Data { map: mmap })
}
