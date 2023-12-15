use std::ffi::c_void;
use std::ffi::CString;
use std::ffi::{c_char, c_int};
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::ptr;
use std::slice;
use std::str;
use std::sync::Arc;

use libloading::os::windows::{Symbol, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR};

use crate::error::Error;
use crate::error::ErrorKind::*;
use crate::Result;

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

        macro_rules! symbol {
            ($name:literal) => {
                lib.get(concat!($name, "\0").as_bytes())
                    .map_err(|error| MissingSymbol {
                        error,
                        symbol: $name,
                    })?
            };
        }

        let tess_base_api_create = symbol!("TessBaseAPICreate");
        let tess_base_api_init3 = symbol!("TessBaseAPIInit3");
        let tess_base_api_delete = symbol!("TessBaseAPIDelete");
        let tess_base_api_set_image = symbol!("TessBaseAPISetImage");
        let tess_base_api_get_utf8_text = symbol!("TessBaseAPIGetUTF8Text");
        let tess_delete_text = symbol!("TessDeleteText");

        let inner = Arc::new(Inner {
            tess_base_api_create,
            tess_base_api_init3,
            tess_base_api_delete,
            tess_base_api_set_image,
            tess_base_api_get_utf8_text,
            tess_delete_text,
            _lib: lib,
        });

        let base = (inner.tess_base_api_create)();

        if (inner.tess_base_api_init3)(base, tessdata.as_ptr(), language.as_ptr()) != 0 {
            return Err(Error::new(Initialize));
        }

        Ok(Tesseract {
            path: path.into(),
            inner: inner.clone(),
            base,
        })
    }
}

struct Inner {
    tess_base_api_create: Symbol<unsafe extern "C" fn() -> *mut BaseApiPtr>,
    tess_base_api_init3:
        Symbol<unsafe extern "C" fn(*mut BaseApiPtr, *const c_char, *const c_char) -> c_int>,
    tess_base_api_delete: Symbol<unsafe extern "C" fn(*mut BaseApiPtr)>,
    tess_base_api_set_image:
        Symbol<unsafe extern "C" fn(*mut BaseApiPtr, *const u8, c_int, c_int, c_int, c_int)>,
    tess_base_api_get_utf8_text: Symbol<unsafe extern "C" fn(*mut BaseApiPtr) -> *mut c_char>,
    tess_delete_text: Symbol<unsafe extern "C" fn(*mut c_char)>,
    _lib: libloading::os::windows::Library,
}

/// A managed tesseract string.
///
/// This derferences to `str`.
pub struct TesseractString {
    inner: Arc<Inner>,
    base: *mut c_char,
    len: usize,
}

unsafe impl Send for TesseractString {}
unsafe impl Sync for TesseractString {}

impl Drop for TesseractString {
    fn drop(&mut self) {
        unsafe {
            (self.inner.tess_delete_text)(self.base);
        }
    }
}

impl Deref for TesseractString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let slice = slice::from_raw_parts(self.base.cast(), self.len);
            str::from_utf8_unchecked(slice)
        }
    }
}

#[repr(transparent)]
struct BaseApiPtr(c_void);

/// A base API instance, associated with a specific language.
pub struct Tesseract {
    path: Box<Path>,
    inner: Arc<Inner>,
    base: *mut BaseApiPtr,
}

impl Tesseract {
    /// The path where tesseract is loaded from.
    pub fn path(&self) -> Option<&Path> {
        Some(&self.path)
    }

    /// Convert image data to text.
    pub fn image_to_text(
        &mut self,
        frame_data: &[u8],
        width: usize,
        height: usize,
        bytes_per_pixel: usize,
    ) -> Result<TesseractString, Error> {
        if bytes_per_pixel == 0 {
            return Err(Error::new(IllegalBytesPerPixel));
        }

        let bytes_per_line = width * bytes_per_pixel;

        let width = c_int::try_from(width)?;
        let height = c_int::try_from(height)?;
        let bytes_per_pixel = c_int::try_from(bytes_per_pixel)?;
        let bytes_per_line = c_int::try_from(bytes_per_line)?;

        self.set_image(frame_data, width, height, bytes_per_pixel, bytes_per_line)?;
        Ok(self.get_utf8_text())
    }

    fn set_image(
        &mut self,
        image_data: &[u8],
        width: c_int,
        height: c_int,
        bytes_per_pixel: c_int,
        bytes_per_line: c_int,
    ) -> Result<(), Error> {
        debug_assert!((height * bytes_per_line) as usize <= image_data.len());

        match bytes_per_pixel {
            0 => {
                debug_assert!(width <= bytes_per_line * 8);
            }
            _ => {
                debug_assert!(width * bytes_per_pixel <= bytes_per_line);
            }
        }

        unsafe {
            (self.inner.tess_base_api_set_image)(
                self.base,
                image_data.as_ptr(),
                width,
                height,
                bytes_per_pixel,
                bytes_per_line,
            );
        };

        Ok(())
    }

    fn get_utf8_text(&self) -> TesseractString {
        unsafe {
            let base = (self.inner.tess_base_api_get_utf8_text)(self.base);

            let mut len = 0;
            let mut cur = base;

            while ptr::read(cur) != 0 {
                cur = cur.add(1);
                len += 1;
            }

            TesseractString {
                inner: self.inner.clone(),
                base,
                len,
            }
        }
    }
}

impl Drop for Tesseract {
    fn drop(&mut self) {
        unsafe { (self.inner.tess_base_api_delete)(self.base) }
    }
}

unsafe impl Send for Tesseract {}
unsafe impl Sync for Tesseract {}
