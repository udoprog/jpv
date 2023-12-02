use std::fmt;

use lib::api;
use lib::reporter::{Reporter, TracingReporter};

use crate::system::{Event, SystemEvents};

pub(crate) struct EventsReporter {
    pub(crate) parent: TracingReporter,
    pub(crate) system_events: SystemEvents,
}

impl Reporter for EventsReporter {
    fn info(&self, module_path: &'static str, value: &dyn fmt::Display) {
        self.parent.info(module_path, value);
        let _ = self.system_events.0.send(Event::LogEntry(api::LogEntry {
            target: module_path.into(),
            level: "info".into(),
            text: value.to_string(),
        }));
    }

    fn warn(&self, module_path: &'static str, value: &dyn fmt::Display) {
        self.parent.warn(module_path, value);
        let _ = self.system_events.0.send(Event::LogEntry(api::LogEntry {
            target: module_path.into(),
            level: "warn".into(),
            text: value.to_string(),
        }));
    }

    fn error(&self, module_path: &'static str, value: &dyn fmt::Display) {
        self.parent.error(module_path, value);
        let _ = self.system_events.0.send(Event::LogEntry(api::LogEntry {
            target: module_path.into(),
            level: "error".into(),
            text: value.to_string(),
        }));
    }

    fn instrument_start(&self, what: &'static str, total: usize) -> u32 {
        0
    }

    fn instrument_progress(&self, id: u32, current: usize) {}

    fn instrument_end(&self, id: u32) {}
}
