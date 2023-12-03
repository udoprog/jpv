//! A custom reporter that can be plugged in for certain components.

use std::fmt;
use std::sync::Arc;

#[macro_export]
macro_rules! report_info {
    ($reporter:expr, $($arg:tt)*) => {
        $crate::reporter::Reporter::info(&$reporter, module_path!(), &format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! report_warn {
    ($reporter:expr, $($arg:tt)*) => {
        $crate::reporter::Reporter::warn(&$reporter, module_path!(), &format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! report_error {
    ($reporter:expr, $($arg:tt)*) => {
        $crate::reporter::Reporter::error(&$reporter, module_path!(), &format_args!($($arg)*));
    }
}

/// The level being reported.
pub enum Level {
    Info,
    Warn,
}

pub trait Reporter: Send + Sync {
    /// Perform info logging.
    fn info(&self, module_path: &'static str, value: &dyn fmt::Display);

    /// Perform warning logging.
    fn warn(&self, module_path: &'static str, value: &dyn fmt::Display);

    /// Perform error logging.
    fn error(&self, module_path: &'static str, value: &dyn fmt::Display);

    /// Start instrumenting.
    fn instrument_start(&self, what: &'static str, total: usize) -> u32;

    /// Report instrumenting progress.
    fn instrument_progress(&self, id: u32, current: usize);

    /// Start instrumenting.
    fn instrument_end(&self, id: u32);
}

impl<T> Reporter for &T
where
    T: ?Sized + Reporter,
{
    #[inline]
    fn info(&self, module_path: &'static str, value: &dyn fmt::Display) {
        (*self).info(module_path, value);
    }

    #[inline]
    fn warn(&self, module_path: &'static str, value: &dyn fmt::Display) {
        (*self).warn(module_path, value);
    }

    #[inline]
    fn error(&self, module_path: &'static str, value: &dyn fmt::Display) {
        (*self).error(module_path, value);
    }

    #[inline]
    fn instrument_start(&self, what: &'static str, total: usize) -> u32 {
        (*self).instrument_start(what, total)
    }

    #[inline]
    fn instrument_progress(&self, id: u32, current: usize) {
        (*self).instrument_progress(id, current)
    }

    #[inline]
    fn instrument_end(&self, id: u32) {
        (*self).instrument_end(id)
    }
}

impl<T> Reporter for Arc<T>
where
    T: ?Sized + Reporter,
{
    #[inline]
    fn info(&self, module_path: &'static str, value: &dyn fmt::Display) {
        (**self).info(module_path, value);
    }

    #[inline]
    fn warn(&self, module_path: &'static str, value: &dyn fmt::Display) {
        (**self).warn(module_path, value);
    }

    #[inline]
    fn error(&self, module_path: &'static str, value: &dyn fmt::Display) {
        (**self).error(module_path, value);
    }

    #[inline]
    fn instrument_start(&self, what: &'static str, total: usize) -> u32 {
        (**self).instrument_start(what, total)
    }

    #[inline]
    fn instrument_progress(&self, id: u32, current: usize) {
        (**self).instrument_progress(id, current)
    }

    #[inline]
    fn instrument_end(&self, id: u32) {
        (**self).instrument_end(id)
    }
}

pub struct TracingReporter;

impl Reporter for TracingReporter {
    #[inline]
    fn info(&self, module_path: &'static str, value: &dyn fmt::Display) {
        tracing::event!(tracing::Level::INFO, "{module_path}: {}", value);
    }

    #[inline]
    fn warn(&self, module_path: &'static str, value: &dyn fmt::Display) {
        tracing::event!(tracing::Level::WARN, "{module_path}: {}", value);
    }

    #[inline]
    fn error(&self, module_path: &'static str, value: &dyn fmt::Display) {
        tracing::event!(tracing::Level::ERROR, "{module_path}: {}", value);
    }

    #[inline]
    fn instrument_start(&self, _: &'static str, _: usize) -> u32 {
        0
    }

    #[inline]
    fn instrument_progress(&self, _: u32, _: usize) {}

    #[inline]
    fn instrument_end(&self, _: u32) {}
}
