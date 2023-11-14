use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[cfg(not(unix))]
static mut DATABASE: musli_zerocopy::AlignedBuf = AlignedBuf::new();

#[cfg(not(unix))]
pub(super) unsafe fn open() -> Result<&'static [u8]> {
    use musli_zerocopy::AlignedBuf;
    use std::io::Read;

    let root = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").context("missing CARGO_MANIFEST_DIR")?,
    );

    let path = manifest_dir.join("..").join("..").join("database.bin");

    tracing::info!("Reading from {}", path.display());

    fn read(path: &Path, output: &mut AlignedBuf) -> io::Result<()> {
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
static mut DATABASE: Option<memmap::Mmap> = None;

#[cfg(unix)]
pub(crate) unsafe fn open() -> Result<&'static [u8]> {
    use core::mem::ManuallyDrop;

    use memmap::MmapOptions;

    let path = match std::env::var_os("CARGO_MANIFEST_DIR") {
        Some(manifest_dir) => {
            let mut path = PathBuf::from(manifest_dir);
            path.push("..");
            path.push("..");
            path.push("database.bin");
            path
        }
        None => PathBuf::from("/usr/share/jpv/database.bin"),
    };

    tracing::info!("Reading from {}", path.display());

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
