//! A custom reporter that can be plugged in for certain components.

use std::fmt;

#[macro_export]
macro_rules! report_info {
    ($reporter:expr, $($arg:tt)*) => {
        $crate::reporter::Reporter::info($reporter, &format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! report_warn {
    ($reporter:expr, $($arg:tt)*) => {
        $crate::reporter::Reporter::warn($reporter, &format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! report_error {
    ($reporter:expr, $($arg:tt)*) => {
        $crate::reporter::Reporter::error($reporter, &format_args!($($arg)*));
    }
}

/// The level being reported.
pub enum Level {
    Info,
    Warn,
}

pub trait Reporter {
    /// Perform info logging.
    fn info(&self, value: &dyn fmt::Display);

    /// Perform warning logging.
    fn warn(&self, value: &dyn fmt::Display);

    /// Perform error logging.
    fn error(&self, value: &dyn fmt::Display);
}

impl<T> Reporter for &T
where
    T: ?Sized + Reporter,
{
    #[inline]
    fn info(&self, value: &dyn fmt::Display) {
        (*self).info(value);
    }

    #[inline]
    fn warn(&self, value: &dyn fmt::Display) {
        (*self).warn(value);
    }

    #[inline]
    fn error(&self, value: &dyn fmt::Display) {
        (*self).error(value);
    }
}

pub struct TracingReporter;

impl Reporter for TracingReporter {
    #[inline]
    fn info(&self, value: &dyn fmt::Display) {
        tracing::event!(tracing::Level::INFO, "{}", value);
    }

    #[inline]
    fn warn(&self, value: &dyn fmt::Display) {
        tracing::event!(tracing::Level::WARN, "{}", value);
    }

    #[inline]
    fn error(&self, value: &dyn fmt::Display) {
        tracing::event!(tracing::Level::ERROR, "{}", value);
    }
}
