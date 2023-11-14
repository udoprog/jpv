use std::fs::File;
use std::io;
use std::path::Path;

use anyhow::{Context, Result};

use crate::dirs::Dirs;
use crate::Args;

#[cfg(not(unix))]
pub(super) unsafe fn open(args: &Args, dirs: &Dirs) -> Result<&'static [u8]> {
    static mut DATABASE: musli_zerocopy::OwnedBuf = OwnedBuf::new();

    use std::io::Read;
    use std::path::PathBuf;

    use musli_zerocopy::OwnedBuf;

    let path = match &args.dictionary {
        Some(path) => path.clone(),
        None => dirs.dictionary(),
    };

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

    read(&path, &mut DATABASE).with_context(|| path.display().to_string())?;
    Ok(DATABASE.as_slice())
}

#[cfg(unix)]
pub(crate) unsafe fn open(args: &Args, dirs: &Dirs) -> Result<&'static [u8]> {
    static mut DATABASE: Option<memmap::Mmap> = None;

    use core::mem::ManuallyDrop;

    use memmap::MmapOptions;

    let path = match &args.dictionary {
        Some(path) => path.clone(),
        None => dirs.dictionary(),
    };

    tracing::info!("Loading dictionary: {}", path.display());

    fn read(path: &Path) -> io::Result<&'static [u8]> {
        let f = ManuallyDrop::new(File::open(path)?);

        let mmap = unsafe { MmapOptions::new().map(&f)? };

        unsafe {
            DATABASE = Some(mmap);

            match &DATABASE {
                Some(mmap) => Ok(&mmap[..]),
                None => unreachable!(),
            }
        }
    }

    let slice = read(&path).with_context(|| path.display().to_string())?;
    Ok(slice)
}
