use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use lib::api;
use tokio::sync::broadcast::Sender;
use tokio::sync::futures::Notified;

/// Service startup.
pub trait Start {
    fn start<'a>(
        &'a mut self,
        port: u16,
        shutdown: Notified<'a>,
        broadcast: Sender<Event>,
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
pub(crate) enum Event {
    #[cfg_attr(not(dbus), allow(unused))]
    SendClipboardData(SendClipboardData),
    LogEntry(api::LogEntry),
}

#[derive(Clone)]
pub(crate) struct SystemEvents(pub(crate) Sender<Event>);
