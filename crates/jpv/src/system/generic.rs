use anyhow::Result;
use tokio::sync::broadcast::Sender;

pub(crate) fn setup<'a>(
    _: u16,
    _: Notified<'a>,
    _: Sender<SystemEvent>,
) -> Result<Option<impl Future<Output = Result<()>> + 'a>> {
    Ok(Some(Box::pin(std::future::pending())))
}
