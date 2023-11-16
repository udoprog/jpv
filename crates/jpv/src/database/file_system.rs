use std::fs::File;
use std::io;
use std::path::Path;

#[cfg(all(unix, feature = "mmap"))]
pub struct Data {
    mmap: memmap::Mmap,
}

#[cfg(all(unix, feature = "mmap"))]
impl Data {
    pub fn as_slice(&self) -> &[u8] {
        &self.mmap[..]
    }
}

#[cfg(any(not(unix), not(feature = "mmap")))]
pub struct Data {
    buf: musli_zerocopy::OwnedBuf,
}

#[cfg(any(not(unix), not(feature = "mmap")))]
impl Data {
    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..]
    }
}

#[cfg(all(unix, feature = "mmap"))]
pub(crate) unsafe fn load_path<P>(path: P) -> io::Result<Data>
where
    P: AsRef<Path>,
{
    use core::mem::ManuallyDrop;
    use memmap::MmapOptions;

    let path = path.as_ref();

    tracing::info!("Loading path: {}", path.display());

    let f = ManuallyDrop::new(File::open(path)?);

    let mmap = MmapOptions::new().map(&f)?;

    let data = Data { mmap };

    Ok(data)
}

#[cfg(any(not(unix), not(feature = "mmap")))]
pub(crate) unsafe fn load_path<P>(path: P) -> io::Result<Data>
where
    P: AsRef<Path>,
{
    use musli_zerocopy::OwnedBuf;
    use std::io::Read;

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
