use core::fmt;
use std::error;
use std::ffi::NulError;
use std::num::TryFromIntError;

#[derive(Debug)]
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

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.kind.source()
    }
}

#[derive(Debug, thiserror::Error)]
pub(super) enum ErrorKind {
    #[error("{0}")]
    NulError(
        #[from]
        #[source]
        NulError,
    ),
    #[error("{0}")]
    TryFromIntError(
        #[from]
        #[source]
        TryFromIntError,
    ),
    #[error("failed to initialize")]
    Initialize,
    #[error("bytes per pixel must be a smaller non-zero multiple of width")]
    IllegalBytesPerPixel,
}
