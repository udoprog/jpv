use std::fs::File;
use std::io;
use std::path::Path;

#[cfg(unix)]
pub(crate) unsafe fn load_path(path: &Path) -> io::Result<&'static [u8]> {
    static mut DATABASE: Option<memmap::Mmap> = None;

    use core::mem::ManuallyDrop;
    use memmap::MmapOptions;

    tracing::info!("Loading dictionary: {}", path.display());

    let f = ManuallyDrop::new(File::open(path)?);

    let mmap = MmapOptions::new().map(&f)?;

    DATABASE = Some(mmap);

    match &DATABASE {
        Some(mmap) => Ok(&mmap[..]),
        None => unreachable!(),
    }
}

#[cfg(not(unix))]
pub(crate) unsafe fn load_path(path: &Path) -> io::Result<(&'static [u8])> {
    static mut DATABASE: musli_zerocopy::OwnedBuf = OwnedBuf::new();

    use std::io::Read;
    use std::path::PathBuf;

    use musli_zerocopy::OwnedBuf;

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

    read(&path, &mut DATABASE)?;
    Ok(DATABASE.as_slice())
}
