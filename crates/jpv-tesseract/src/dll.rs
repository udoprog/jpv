use std::ffi::CString;
use std::io;
use std::path::PathBuf;

use libloading::os::windows::LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR;

use crate::error::Error;
use crate::error::ErrorKind::*;
use crate::Result;

pub use crate::dynamic::{Tesseract, TesseractString};

/// Open the tesseract library.
pub fn open(language: &str) -> Result<Tesseract> {
    let key = match winctx::OpenRegistryKey::local_machine().open("Software\\Tesseract-OCR") {
        Ok(key) => key,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(Error::new(NotInstalled)),
        Err(e) => return Err(Error::new(OpenRegistryKey(e))),
    };

    let path = PathBuf::from(key.get_string("Path").map_err(GetRegistryPath)?);

    let version = key
        .get_string("CurrentVersion")
        .map_err(GetRegistryCurrentVersion)?;

    let version = version.to_string_lossy();

    let Some((major @ "5", _)) = version.split_once('.') else {
        return Err(Error::new(UnsupportedMajorVersion(
            version.as_ref().into(),
            5,
        )));
    };

    let dll = path.join(format!("libtesseract-{major}.dll"));
    let tessdata = path.join("tessdata");

    let expected_data = tessdata.join(format!("{}.traineddata", language));

    if !expected_data.is_file() {
        return Err(Error::new(MissingLanguage(expected_data.into())));
    }

    let tessdata = tessdata.into_os_string();
    let tessdata = tessdata.to_string_lossy();

    let language = CString::new(language)?;
    let tessdata = CString::new(tessdata.as_ref())?;

    unsafe {
        let lib = libloading::os::windows::Library::load_with_flags(
            dll,
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
        )
        .map_err(LoadLibrary)?;

        crate::dynamic::open(lib.into(), path.into(), Some(&tessdata), &language)
    }
}
