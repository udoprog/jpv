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
    pub(crate) name: Option<Box<str>>,
}

impl EventsReporter {
    fn emit(&self, entry: api::OwnedLogEntry) {
        let mut inner = self.inner.write().unwrap();
        inner.log.push(entry.clone());
        self.system_events.send(Event::LogEntry(entry));
    }
}

impl Reporter for EventsReporter {
    fn info(&self, module_path: &'static str, value: &dyn fmt::Display) {
        self.parent.info(module_path, value);

        self.emit(api::OwnedLogEntry {
            target: module_path.into(),
            level: "info".into(),
            text: value.to_string(),
        });
    }

    fn warn(&self, module_path: &'static str, value: &dyn fmt::Display) {
        self.parent.warn(module_path, value);

        self.emit(api::OwnedLogEntry {
            target: module_path.into(),
            level: "warn".into(),
            text: value.to_string(),
        });
    }

    fn error(&self, module_path: &'static str, value: &dyn fmt::Display) {
        self.parent.error(module_path, value);

        self.emit(api::OwnedLogEntry {
            target: module_path.into(),
            level: "error".into(),
            text: value.to_string(),
        });
    }

    fn instrument_start(&self, _: &'static str, text: &dyn fmt::Display, total: Option<usize>) {
        use std::fmt::Write;

        let Some(name) = self.name.as_deref() else {
            return;
        };

        let progress = {
            let mut inner = self.inner.write().unwrap();

            let Some(progress) = inner.tasks.get_mut(name) else {
                return;
            };

            progress.text.clear();
            write!(progress.text, "{}", text).unwrap();
            progress.value = 0;
            progress.total = total;
            progress.clone()
        };

        self.system_events.send(Event::TaskProgress(progress));
    }

    fn instrument_progress(&self, stride: usize) {
        let Some(name) = self.name.as_deref() else {
            return;
        };

        let progress = {
            let mut inner = self.inner.write().unwrap();

            let Some(progress) = inner.tasks.get_mut(name) else {
                return;
            };

            progress.value = progress.value.wrapping_add(stride);
            progress.clone()
        };

        self.system_events.send(Event::TaskProgress(progress));
    }

    fn instrument_end(&self, total: usize) {
        let Some(name) = self.name.as_deref() else {
            return;
        };

        let progress = {
            let mut inner = self.inner.write().unwrap();

            let Some(progress) = inner.tasks.get_mut(name) else {
                return;
            };

            progress.value = total;
            progress.total = Some(total);
            progress.step += 1;
            progress.clone()
        };

        self.system_events.send(Event::TaskProgress(progress));
    }
}
