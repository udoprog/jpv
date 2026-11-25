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
    #[error("Failed to create the tesseract API")]
    #[cfg(any(windows, all(unix, not(feature = "linked"))))]
    Create,
    #[error("Failed to initialize tesseract for language `{0}`, is its language data installed?")]
    #[cfg(any(unix, windows))]
    Initialize(Box<str>),
    #[error("Bytes per pixel must be a smaller non-zero multiple of width")]
    #[cfg(any(unix, windows))]
    IllegalBytesPerPixel,
    #[error("Image data is smaller than its dimensions")]
    #[cfg(any(windows, all(unix, not(feature = "linked"))))]
    ImageTooSmall,
    #[error("Failed to recognize text")]
    #[cfg(any(unix, windows))]
    Recognize,
    #[error("Recognized text is not valid UTF-8")]
    #[cfg(any(unix, windows))]
    InvalidUtf8,
    #[error("Failed to load dynamic library")]
    #[cfg(windows)]
    LoadLibrary(#[source] libloading::Error),
    #[error("Missing symbol `{symbol}` in dynamic library")]
    #[cfg(any(windows, all(unix, not(feature = "linked"))))]
    MissingSymbol {
        error: libloading::Error,
        symbol: &'static str,
    },
    #[error("Failed to open registry key")]
    #[cfg(windows)]
    OpenRegistryKey(#[source] io::Error),
    #[error("Failed to get `Path` key from registry")]
    #[cfg(windows)]
    GetRegistryPath(#[source] io::Error),
    #[error("Failed to get `CurrentVersion` key from registry")]
    #[cfg(windows)]
    GetRegistryCurrentVersion(#[source] io::Error),
    #[error("Unsupported version `{0}, expected major version `{1}`")]
    #[cfg(windows)]
    UnsupportedMajorVersion(Box<str>, u32),
    #[error("Not installed, please install it from https://github.com/UB-Mannheim/tesseract/wiki")]
    #[cfg(windows)]
    NotInstalled,
    #[error("Not installed, please install tesseract and its Japanese language data")]
    #[cfg(all(unix, not(feature = "linked")))]
    NotInstalled(#[source] Option<libloading::Error>),
    #[error("Missing language data ({0}), please install it through the installer")]
    #[cfg(windows)]
    MissingLanguage(Box<Path>),
    #[error("Platform is not supported")]
    #[cfg(not(any(unix, windows)))]
    Unsupported,
}
