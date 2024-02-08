//! A custom reporter that can be plugged in for certain components.

use std::fmt;
use std::sync::Arc;

/// The level being reported.
pub enum Level {
    Info,
    Warn,
}

pub trait Reporter: Send + Sync {
    /// Start instrumenting.
    fn instrument_start(
        &self,
        module_path: &'static str,
        what: &dyn fmt::Display,
        total: Option<usize>,
    );

    /// Report instrumenting progress.
    fn instrument_progress(&self, stride: usize);

    /// Start instrumenting.
    fn instrument_end(&self, total: usize);
}

impl<T> Reporter for &T
where
    T: ?Sized + Reporter,
{
    #[inline]
    fn instrument_start(
        &self,
        module_path: &'static str,
        what: &dyn fmt::Display,
        total: Option<usize>,
    ) {
        (*self).instrument_start(module_path, what, total)
    }

    #[inline]
    fn instrument_progress(&self, stride: usize) {
        (*self).instrument_progress(stride)
    }

    #[inline]
    fn instrument_end(&self, total: usize) {
        (*self).instrument_end(total)
    }
}

impl<T> Reporter for Arc<T>
where
    T: ?Sized + Reporter,
{
    #[inline]
    fn instrument_start(
        &self,
        module_path: &'static str,
        what: &dyn fmt::Display,
        total: Option<usize>,
    ) {
        (**self).instrument_start(module_path, what, total)
    }

    #[inline]
    fn instrument_progress(&self, stride: usize) {
        (**self).instrument_progress(stride)
    }

    #[inline]
    fn instrument_end(&self, total: usize) {
        (**self).instrument_end(total)
    }
}

pub struct EmptyReporter;

impl Reporter for EmptyReporter {
    #[inline]
    fn instrument_start(
        &self,
        module_path: &'static str,
        value: &dyn fmt::Display,
        _: Option<usize>,
    ) {
        tracing::event!(tracing::Level::INFO, "{module_path}: {}", value);
    }

    #[inline]
    fn instrument_progress(&self, _: usize) {}

    #[inline]
    fn instrument_end(&self, _: usize) {}
}
