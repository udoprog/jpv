//! Tesseract bindings through a library loaded at runtime.
//!
//! This is shared by the Windows backend, which finds the library through the
//! registry, and the unix backend, which searches the usual library names.

use std::ffi::c_void;
use std::ffi::CStr;
use std::ffi::{c_char, c_int};
use std::ops::Deref;
use std::path::Path;
use std::ptr;
use std::slice;
use std::str;
use std::sync::Arc;

use libloading::Library;

use crate::error::Error;
use crate::error::ErrorKind::*;
use crate::Result;

#[repr(transparent)]
struct BaseApiPtr(c_void);

type TessBaseApiCreate = unsafe extern "C" fn() -> *mut BaseApiPtr;
type TessBaseApiInit3 =
    unsafe extern "C" fn(*mut BaseApiPtr, *const c_char, *const c_char) -> c_int;
type TessBaseApiDelete = unsafe extern "C" fn(*mut BaseApiPtr);
type TessBaseApiSetImage =
    unsafe extern "C" fn(*mut BaseApiPtr, *const u8, c_int, c_int, c_int, c_int);
type TessBaseApiGetUtf8Text = unsafe extern "C" fn(*mut BaseApiPtr) -> *mut c_char;
type TessDeleteText = unsafe extern "C" fn(*mut c_char);

/// The functions of the tesseract C API that are used, and the library they
/// were loaded from.
struct Api {
    tess_base_api_create: TessBaseApiCreate,
    tess_base_api_init3: TessBaseApiInit3,
    tess_base_api_delete: TessBaseApiDelete,
    tess_base_api_set_image: TessBaseApiSetImage,
    tess_base_api_get_utf8_text: TessBaseApiGetUtf8Text,
    tess_delete_text: TessDeleteText,
    // The function pointers above point into this library, so it must be kept
    // alive for as long as they are.
    _lib: Library,
}

impl Api {
    /// Look up the functions used in the given library.
    ///
    /// # Safety
    ///
    /// The library must be a tesseract library exporting the tesseract C API
    /// with the expected signatures.
    unsafe fn load(lib: Library) -> Result<Self> {
        macro_rules! symbol {
            ($name:literal) => {
                *lib.get(concat!($name, "\0").as_bytes())
                    .map_err(|error| MissingSymbol {
                        error,
                        symbol: $name,
                    })?
            };
        }

        Ok(Self {
            tess_base_api_create: symbol!("TessBaseAPICreate"),
            tess_base_api_init3: symbol!("TessBaseAPIInit3"),
            tess_base_api_delete: symbol!("TessBaseAPIDelete"),
            tess_base_api_set_image: symbol!("TessBaseAPISetImage"),
            tess_base_api_get_utf8_text: symbol!("TessBaseAPIGetUTF8Text"),
            tess_delete_text: symbol!("TessDeleteText"),
            _lib: lib,
        })
    }
}

/// Set up a tesseract base API for `language` from a loaded library.
///
/// A `tessdata` of `None` lets tesseract find its data itself, through
/// `TESSDATA_PREFIX` or the location it was built with.
///
/// # Safety
///
/// The library must be a tesseract library exporting the tesseract C API with
/// the expected signatures.
pub(crate) unsafe fn open(
    lib: Library,
    path: Box<Path>,
    tessdata: Option<&CStr>,
    language: &CStr,
) -> Result<Tesseract> {
    let inner = Arc::new(Api::load(lib)?);

    let base = (inner.tess_base_api_create)();

    if base.is_null() {
        return Err(Error::new(Create));
    }

    // Constructed before initialization so that the base API is deleted if it
    // fails.
    let tesseract = Tesseract { path, inner, base };

    let tessdata = tessdata.map_or(ptr::null(), CStr::as_ptr);

    if (tesseract.inner.tess_base_api_init3)(tesseract.base, tessdata, language.as_ptr()) != 0 {
        return Err(Error::new(Initialize(
            language.to_string_lossy().into_owned().into(),
        )));
    }

    Ok(tesseract)
}

/// A managed tesseract string.
///
/// This derferences to `str`.
pub struct TesseractString {
    inner: Arc<Api>,
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
        // SAFETY: The text is checked to be valid UTF-8 when it is constructed,
        // and is owned by this string until it is dropped.
        unsafe {
            let slice = slice::from_raw_parts(self.base.cast(), self.len);
            str::from_utf8_unchecked(slice)
        }
    }
}

/// A base API instance, associated with a specific language.
pub struct Tesseract {
    path: Box<Path>,
    inner: Arc<Api>,
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

        let bytes_per_line = width
            .checked_mul(bytes_per_pixel)
            .ok_or(IllegalBytesPerPixel)?;

        if bytes_per_line
            .checked_mul(height)
            .is_none_or(|len| len > frame_data.len())
        {
            return Err(Error::new(ImageTooSmall));
        }

        let width = c_int::try_from(width)?;
        let height = c_int::try_from(height)?;
        let bytes_per_pixel = c_int::try_from(bytes_per_pixel)?;
        let bytes_per_line = c_int::try_from(bytes_per_line)?;

        unsafe {
            (self.inner.tess_base_api_set_image)(
                self.base,
                frame_data.as_ptr(),
                width,
                height,
                bytes_per_pixel,
                bytes_per_line,
            );
        }

        self.get_utf8_text()
    }

    fn get_utf8_text(&self) -> Result<TesseractString> {
        unsafe {
            let base = (self.inner.tess_base_api_get_utf8_text)(self.base);

            if base.is_null() {
                return Err(Error::new(Recognize));
            }

            let bytes = CStr::from_ptr(base).to_bytes();
            let valid = str::from_utf8(bytes).is_ok();

            // Constructed before validation so that the text is deleted if it
            // is not valid.
            let string = TesseractString {
                inner: self.inner.clone(),
                base,
                len: bytes.len(),
            };

            if !valid {
                return Err(Error::new(InvalidUtf8));
            }

            Ok(string)
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
