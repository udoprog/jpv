use std::fmt;
use std::sync::{Arc, RwLock};

use lib::api;
use lib::reporter::{Reporter, TracingReporter};

use crate::background::BackgroundInner;
use crate::system::{Event, SystemEvents};

pub(crate) struct EventsReporter {
    pub(crate) parent: TracingReporter,
    pub(crate) inner: Arc<RwLock<BackgroundInner>>,
    pub(crate) system_events: SystemEvents,
}

impl EventsReporter {
    fn emit(&self, entry: api::LogEntry) {
        let mut inner = self.inner.write().unwrap();
        inner.log.push(entry.clone());
        let _ = self.system_events.0.send(Event::LogEntry(entry));
    }
}

impl Reporter for EventsReporter {
    fn info(&self, module_path: &'static str, value: &dyn fmt::Display) {
        self.parent.info(module_path, value);
        self.emit(api::LogEntry {
            target: module_path.into(),
            level: "info".into(),
            text: value.to_string(),
        });
    }

    fn warn(&self, module_path: &'static str, value: &dyn fmt::Display) {
        self.parent.warn(module_path, value);
        self.emit(api::LogEntry {
            target: module_path.into(),
            level: "warn".into(),
            text: value.to_string(),
        });
    }

    fn error(&self, module_path: &'static str, value: &dyn fmt::Display) {
        self.parent.error(module_path, value);
        self.emit(api::LogEntry {
            target: module_path.into(),
            level: "error".into(),
            text: value.to_string(),
        });
    }

    fn instrument_start(&self, _: &'static str, _: usize) -> u32 {
        0
    }

    fn instrument_progress(&self, _: u32, _: usize) {}

    fn instrument_end(&self, _: u32) {}
}
