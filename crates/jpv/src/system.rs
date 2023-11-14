use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use tokio::sync::broadcast::Sender;

pub(crate) enum Setup<'a> {
    Future(Pin<Box<dyn Future<Output = Result<()>> + 'a>>),
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
    #[allow(unused)]
    SendClipboardData(SendClipboardData),
}

#[derive(Clone)]
pub(crate) struct SystemEvents(pub(crate) Sender<Event>);
