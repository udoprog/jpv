use std::ffi::CString;
use std::path::Path;

use libloading::os::unix::Library;

use crate::error::Error;
use crate::error::ErrorKind::*;
use crate::Result;

pub use crate::dynamic::{Tesseract, TesseractString};

/// Names to try loading tesseract by, in order.
///
/// The soname differs between builds: autotools builds (Debian, Ubuntu) use
/// `libtesseract.so.5`, CMake builds (Fedora) use `libtesseract.so.5.<minor>`.
/// The C API used here is the same in tesseract 4, so that is tried last.
#[cfg(not(target_os = "macos"))]
const CANDIDATES: &[&str] = &[
    "libtesseract.so.5",
    "libtesseract.so.5.9",
    "libtesseract.so.5.8",
    "libtesseract.so.5.7",
    "libtesseract.so.5.6",
    "libtesseract.so.5.5",
    "libtesseract.so.5.4",
    "libtesseract.so.5.3",
    "libtesseract.so.5.2",
    "libtesseract.so.5.1",
    "libtesseract.so.5.0",
    "libtesseract.so",
    "libtesseract.so.4",
];

/// Names to try loading tesseract by, in order.
///
/// Homebrew and MacPorts install outside of the default search path of
/// `dlopen`, so their prefixes are tried by full path.
#[cfg(target_os = "macos")]
const CANDIDATES: &[&str] = &[
    "libtesseract.5.dylib",
    "/opt/homebrew/lib/libtesseract.5.dylib",
    "/usr/local/lib/libtesseract.5.dylib",
    "/opt/local/lib/libtesseract.5.dylib",
    "libtesseract.dylib",
    "/opt/homebrew/lib/libtesseract.dylib",
    "/usr/local/lib/libtesseract.dylib",
    "/opt/local/lib/libtesseract.dylib",
];

/// Open the tesseract library.
///
/// Tesseract finds its own language data, either through `TESSDATA_PREFIX` or
/// the location it was built with.
pub fn open(language: &str) -> Result<Tesseract> {
    let language = CString::new(language)?;

    let mut last_error = None;

    for &name in CANDIDATES {
        // SAFETY: Loading tesseract runs its initializers, which are not
        // expected to have any requirements on the caller.
        let lib = match unsafe { Library::new(name) } {
            Ok(lib) => lib,
            Err(error) => {
                last_error = Some(error);
                continue;
            }
        };

        // SAFETY: A library by this name is expected to be tesseract.
        return unsafe {
            crate::dynamic::open(lib.into(), Path::new(name).into(), None, &language)
        };
    }

    Err(Error::new(NotInstalled(last_error)))
}
