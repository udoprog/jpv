use std::ffi::NulError;
#[cfg(windows)]
use std::io;
use std::num::TryFromIntError;
#[cfg(windows)]
use std::path::Path;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub(crate) fn new<K>(kind: K) -> Self
    where
        ErrorKind: From<K>,
    {
        Self { kind: kind.into() }
    }
}

impl<K> From<K> for Error
where
    ErrorKind: From<K>,
{
    #[inline]
    fn from(value: K) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub(super) enum ErrorKind {
    #[error("String is not null terminated")]
    NulError(
        #[from]
        #[source]
        NulError,
    ),
    #[error("Failed integer conversion")]
    TryFromIntError(
        #[from]
        #[source]
        TryFromIntError,
    ),
    #[cfg(any(windows, feature = "tesseract-sys"))]
    #[error("Failed to initialize")]
    Initialize,
    #[cfg(any(windows, feature = "tesseract-sys"))]
    #[error("Bytes per pixel must be a smaller non-zero multiple of width")]
    IllegalBytesPerPixel,
    #[cfg(windows)]
    #[error("Failed to load dynamic library")]
    LoadLibrary(#[source] libloading::Error),
    #[cfg(windows)]
    #[error("Missing symbol `{symbol}` in dynamic library")]
    MissingSymbol {
        error: libloading::Error,
        symbol: &'static str,
    },
    #[cfg(windows)]
    #[error("Failed to open registry key")]
    OpenRegistryKey(#[source] io::Error),
    #[cfg(windows)]
    #[error("Failed to get `Path` key from registry")]
    GetRegistryPath(#[source] io::Error),
    #[cfg(windows)]
    #[error("Failed to get `CurrentVersion` key from registry")]
    GetRegistryCurrentVersion(#[source] io::Error),
    #[cfg(windows)]
    #[error("Unsupported version `{0}, expected major version `{1}`")]
    UnsupportedMajorVersion(Box<str>, u32),
    #[cfg(windows)]
    #[error("Not installed, please install it from https://github.com/UB-Mannheim/tesseract/wiki")]
    NotInstalled,
    #[cfg(windows)]
    #[error("Missing language data ({0}), please install it through the installer")]
    MissingLanguage(Box<Path>),
    #[cfg(all(not(windows), not(feature = "tesseract-sys")))]
    #[error("Platform is not supported")]
    Unsupported,
}
