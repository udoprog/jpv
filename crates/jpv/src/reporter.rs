use std::fmt;

use lib::api;
use lib::reporter::{Reporter, TracingReporter};

use crate::system::{Event, SystemEvents};

pub(crate) struct EventsReporter {
    pub(crate) parent: TracingReporter,
    pub(crate) system_events: SystemEvents,
}

impl Reporter for EventsReporter {
    fn info(&self, value: &dyn fmt::Display) {
        self.parent.info(value);
        let _ = self.system_events.0.send(Event::LogEntry(api::LogEntry {
            level: "info".into(),
            text: value.to_string(),
        }));
    }

    fn warn(&self, value: &dyn fmt::Display) {
        self.parent.warn(value);
        let _ = self.system_events.0.send(Event::LogEntry(api::LogEntry {
            level: "warn".into(),
            text: value.to_string(),
        }));
    }

    fn error(&self, value: &dyn fmt::Display) {
        self.parent.error(value);
        let _ = self.system_events.0.send(Event::LogEntry(api::LogEntry {
            level: "error".into(),
            text: value.to_string(),
        }));
    }
}
