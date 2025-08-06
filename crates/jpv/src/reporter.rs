use std::fmt;
use std::sync::{Arc, Mutex};

use lib::reporter::Reporter;

use crate::background::BackgroundTasks;
use crate::system::{Event, SystemEvents};
use crate::tasks::TaskName;

pub(crate) struct EventsReporter {
    pub(crate) tasks: Arc<Mutex<BackgroundTasks>>,
    pub(crate) system_events: SystemEvents,
    pub(crate) name: Option<TaskName>,
}

impl Reporter for EventsReporter {
    fn instrument_start(&self, _: &'static str, text: &dyn fmt::Display, total: Option<usize>) {
        use std::fmt::Write;

        let Some(name) = &self.name else {
            return;
        };

        let progress = {
            let mut inner = self.tasks.lock().unwrap();

            let Some(progress) = inner.progress.get_mut(name) else {
                return;
            };

            progress.text.clear();
            write!(progress.text, "{text}").unwrap();
            progress.value = 0;
            progress.total = total;
            progress.clone()
        };

        self.system_events.send(Event::TaskProgress(progress));
    }

    fn instrument_progress(&self, stride: usize) {
        let Some(name) = &self.name else {
            return;
        };

        let progress = {
            let mut inner = self.tasks.lock().unwrap();

            let Some(progress) = inner.progress.get_mut(name) else {
                return;
            };

            progress.value = progress.value.wrapping_add(stride);
            progress.clone()
        };

        self.system_events.send(Event::TaskProgress(progress));
    }

    fn instrument_end(&self, total: usize) {
        let Some(name) = &self.name else {
            return;
        };

        let progress = {
            let mut tasks = self.tasks.lock().unwrap();

            let Some(progress) = tasks.progress.get_mut(name) else {
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
