use anyhow::Result;

static DATABASE: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../database.bin"));

pub(crate) unsafe fn open() -> Result<&'static [u8]> {
    Ok(&DATABASE)
}
