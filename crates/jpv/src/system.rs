use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use lib::api;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::futures::Notified;

/// Service startup.
pub(crate) trait Start {
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
    pub(crate) name: Box<str>,
    pub(crate) value: usize,
    pub(crate) total: Option<usize>,
    pub(crate) step: usize,
    pub(crate) steps: usize,
    pub(crate) text: String,
}

#[derive(Clone)]
pub(crate) struct TaskCompleted {
    pub(crate) name: Box<str>,
}

#[derive(Clone)]
#[allow(unused)]
pub(crate) enum Event {
    SendClipboardData(SendClipboardData),
    /// Send a dynamic image directly.
    SendDynamicImage(image::DynamicImage),
    /// Send text to analyze.
    SendText(String),
    /// Emit a log entry.
    LogEntry(api::OwnedLogEntry),
    /// Emit task progress.
    TaskProgress(TaskProgress),
    /// Emit that a task has completed.
    TaskCompleted(TaskCompleted),
    /// Indicate that clients should refresh their state.
    Refresh,
}

#[derive(Clone)]
pub(crate) struct SystemEvents(Sender<Event>);

impl SystemEvents {
    pub(crate) fn new() -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(16);
        Self(sender)
    }

    pub(crate) fn send(&self, value: Event) {
        let _ = self.0.send(value);
    }

    pub(crate) fn subscribe(&self) -> Receiver<Event> {
        self.0.subscribe()
    }
}
