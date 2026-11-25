use std::ffi::{c_char, c_int};
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::path::Path;
use std::ptr;
use std::slice;
use std::str;

use tesseract_sys::{
    TessBaseAPICreate, TessBaseAPIDelete, TessBaseAPIGetUTF8Text, TessBaseAPIInit3,
    TessBaseAPISetImage, TessDeleteText,
};

use crate::error::{Error, ErrorKind};
use crate::Result;

/// Try and open the tesseract API.
pub fn open(language: &str) -> Result<Tesseract> {
    let c_language = CString::new(language)?;

    unsafe {
        // Constructed before initialization so that the base API is deleted if
        // it fails.
        let tesseract = Tesseract {
            base: TessBaseAPICreate(),
        };

        if TessBaseAPIInit3(tesseract.base, ptr::null_mut(), c_language.as_ptr()) != 0 {
            return Err(Error::new(ErrorKind::Initialize(language.into())));
        }

        Ok(tesseract)
    }
}

/// A managed tesseract string.
///
/// This derferences to `str`.
pub struct TesseractString(*mut c_char, usize);

unsafe impl Send for TesseractString {}
unsafe impl Sync for TesseractString {}

impl Drop for TesseractString {
    fn drop(&mut self) {
        unsafe {
            TessDeleteText(self.0);
        }
    }
}

impl Deref for TesseractString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let slice = slice::from_raw_parts(self.0.cast(), self.1);
            str::from_utf8_unchecked(slice)
        }
    }
}

/// Tesseract handler.
pub struct Tesseract {
    base: *mut tesseract_sys::TessBaseAPI,
}

impl Tesseract {
    /// The path where tesseract is loaded from.
    pub fn path(&self) -> Option<&Path> {
        None
    }

    /// Perform OCR recognition on a frame of image data.
    pub fn image_to_text(
        &mut self,
        frame_data: &[u8],
        width: usize,
        height: usize,
        bytes_per_pixel: usize,
    ) -> Result<TesseractString, Error> {
        if bytes_per_pixel == 0 {
            return Err(Error::new(ErrorKind::IllegalBytesPerPixel));
        }

        let bytes_per_line = width * bytes_per_pixel;

        let width = c_int::try_from(width)?;
        let height = c_int::try_from(height)?;
        let bytes_per_pixel = c_int::try_from(bytes_per_pixel)?;
        let bytes_per_line = c_int::try_from(bytes_per_line)?;

        self.set_image(frame_data, width, height, bytes_per_pixel, bytes_per_line)?;
        self.get_utf8_text()
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
            TessBaseAPISetImage(
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

    fn get_utf8_text(&self) -> Result<TesseractString> {
        unsafe {
            let text = TessBaseAPIGetUTF8Text(self.base);

            if text.is_null() {
                return Err(Error::new(ErrorKind::Recognize));
            }

            let bytes = CStr::from_ptr(text).to_bytes();
            let valid = str::from_utf8(bytes).is_ok();

            // Constructed before validation so that the text is deleted if it
            // is not valid.
            let string = TesseractString(text, bytes.len());

            if !valid {
                return Err(Error::new(ErrorKind::InvalidUtf8));
            }

            Ok(string)
        }
    }
}

impl Drop for Tesseract {
    fn drop(&mut self) {
        unsafe { TessBaseAPIDelete(self.base) }
    }
}

unsafe impl Send for Tesseract {}
unsafe impl Sync for Tesseract {}
