use anyhow::Result;

pub(crate) fn setup<'a>(
    _: u16,
    _: Notified<'a>,
) -> Result<Option<impl Future<Output = Result<()>> + 'a>> {
    Ok(Some(Box::pin(std::future::pending())))
}
