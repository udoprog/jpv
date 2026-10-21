use std::fs::File;
use std::io;
use std::path::Path;

use musli_zerocopy::{Buf, OwnedBuf};

pub struct Data {
    buf: OwnedBuf,
}

impl Data {
    /// Get a slice to the underlying data.
    pub fn as_buf(&self) -> &Buf {
        &self.buf
    }
}

pub fn open<P>(path: P) -> io::Result<Data>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    tracing::info!("Loading database from {}", path.display());

    let mut file = File::open(path)?;
    let len = usize::try_from(file.metadata()?.len()).map_err(io::Error::other)?;
    let mut buf = OwnedBuf::with_capacity(len).map_err(io::Error::other)?;
    io::copy(&mut file, &mut buf)?;
    Ok(Data { buf })
}
