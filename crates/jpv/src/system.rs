use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use lib::api;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::futures::Notified;

/// Service startup.
pub trait Start {
    fn start<'a>(
        &'a mut self,
        port: u16,
        shutdown: Notified<'a>,
        system_events: &'a SystemEvents,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>;
}

pub(crate) enum Setup {
    Start(Option<Box<dyn Start>>),
    #[allow(unused)]
    Port(u16),
    #[allow(unused)]
    Busy,
}

#[derive(Clone)]
pub(crate) struct SendClipboardData {
    pub(crate) mimetype: String,
    pub(crate) data: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct TaskProgress {
    pub(crate) name: &'static str,
    pub(crate) value: usize,
    pub(crate) total: Option<usize>,
    pub(crate) step: usize,
    pub(crate) steps: usize,
    pub(crate) text: String,
}

#[derive(Clone)]
pub(crate) struct TaskCompleted {
    pub(crate) name: &'static str,
}

#[derive(Clone)]
pub(crate) enum Event {
    #[cfg_attr(not(dbus), allow(unused))]
    SendClipboardData(SendClipboardData),
    LogEntry(api::LogEntry),
    TaskProgress(TaskProgress),
    TaskCompleted(TaskCompleted),
}

#[derive(Clone)]
pub(crate) struct SystemEvents(Sender<Event>);

impl SystemEvents {
    pub(crate) fn new() -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(16);
        Self(sender)
    }

    pub(crate) fn send(&self, value: Event) {
        if let Err(error) = self.0.send(value) {
            tracing::error!("{}", error);
        }
    }

    pub(crate) fn subscribe(&self) -> Receiver<Event> {
        self.0.subscribe()
    }
}
