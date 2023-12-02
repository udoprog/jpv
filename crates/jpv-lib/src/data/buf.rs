use std::fs::File;
use std::io;
use std::io::Read;
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

    fn read(path: &Path, output: &mut OwnedBuf) -> io::Result<()> {
        let mut f = File::open(path)?;

        let mut chunk = [0; 1024];

        loop {
            let n = f.read(&mut chunk[..])?;

            if n == 0 {
                break;
            }

            output.extend_from_slice(&chunk[..n]);
        }

        Ok(())
    }

    let mut buf = OwnedBuf::new();
    read(&path, &mut buf)?;
    Ok(Data { buf })
}
