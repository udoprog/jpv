use std::fmt;
use std::sync::{Arc, RwLock};

use lib::reporter::Reporter;

use crate::background::Mutable;
use crate::system::{Event, SystemEvents};

pub(crate) struct EventsReporter {
    pub(crate) inner: Arc<RwLock<Mutable>>,
    pub(crate) system_events: SystemEvents,
    pub(crate) name: Option<Box<str>>,
}

impl Reporter for EventsReporter {
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
