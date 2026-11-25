use std::fmt;
use std::str::Utf8Error;

use phrasebook::Phrases;

/// Result type in use.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Errors raised in this application.
pub struct Error {
    error: anyhow::Error,
    known: Option<Known>,
}

/// Errors the UI words itself, through the catalog, rather than showing
/// the error's own English text.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Known {
    /// A request was made while the socket was closed.
    NotConnected,
    /// The connection closed before a request was answered.
    ConnectionLost,
}

impl Error {
    /// One of the errors the UI words itself.
    pub(crate) fn known(known: Known) -> Self {
        let text = match known {
            Known::NotConnected => "Socket is not connected",
            Known::ConnectionLost => "Connection to the service was lost before it answered",
        };

        Self {
            error: anyhow::Error::msg(text),
            known: Some(known),
        }
    }

    /// The error as the UI shows it, in the current language when it is one
    /// the UI words itself. Others, such as those the service sends, are
    /// shown as written.
    pub(crate) fn describe(&self, phrases: &Phrases) -> String {
        match self.known {
            Some(Known::NotConnected) => phrases.text("error-not-connected").to_string(),
            Some(Known::ConnectionLost) => phrases.text("error-connection-lost").to_string(),
            None => self.error.to_string(),
        }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl From<anyhow::Error> for Error {
    #[inline]
    fn from(error: anyhow::Error) -> Self {
        Self { error, known: None }
    }
}

impl From<&'static str> for Error {
    #[inline]
    fn from(value: &'static str) -> Self {
        Self {
            error: anyhow::Error::msg(value),
            known: None,
        }
    }
}

impl From<wasm_bindgen::JsValue> for Error {
    #[inline]
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self {
            error: anyhow::Error::msg(format!("{value:?}")),
            known: None,
        }
    }
}

impl From<url::ParseError> for Error {
    #[inline]
    fn from(error: url::ParseError) -> Self {
        Self {
            error: anyhow::Error::from(error),
            known: None,
        }
    }
}

impl From<serde_json::Error> for Error {
    #[inline]
    fn from(error: serde_json::Error) -> Self {
        Self {
            error: anyhow::Error::from(error),
            known: None,
        }
    }
}

impl From<musli::storage::Error> for Error {
    #[inline]
    fn from(error: musli::storage::Error) -> Self {
        Self {
            error: anyhow::Error::from(error),
            known: None,
        }
    }
}

impl From<Utf8Error> for Error {
    #[inline]
    fn from(error: Utf8Error) -> Self {
        Self {
            error: anyhow::Error::from(error),
            known: None,
        }
    }
}
