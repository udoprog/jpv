use std::ops::Deref;
use std::path::Path;

use crate::error::{Error, ErrorKind};

/// Open the tesseract API, all though it is never supported with the fake implementation.
pub fn open(_: &str) -> Result<Tesseract, Error> {
    Err(Error::new(ErrorKind::Unsupported))
}

/// Fake tesseract String.
pub struct TesseractString;

impl Deref for TesseractString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        ""
    }
}

/// Fake tesseract API.
pub struct Tesseract;

impl Tesseract {
    /// The path where tesseract is loaded from.
    pub fn path(&self) -> Option<&Path> {
        None
    }

    /// Perform OCR recognition on a frame of image data.
    pub fn image_to_text(
        &self,
        _frame_data: &[u8],
        _width: usize,
        _height: usize,
        _bytes_per_pixel: usize,
    ) -> Result<TesseractString, Error> {
        Err(Error::new(ErrorKind::Unsupported))
    }
}
