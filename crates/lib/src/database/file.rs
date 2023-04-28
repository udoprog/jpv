use musli::{Decode, Encode};

/// The size of the header.
pub(super) const HEADER_SIZE: usize = 12;

/// Database header.
#[derive(Default, Debug, Encode, Decode)]
#[musli(packed)]
pub(super) struct Header {
    pub(super) entries: usize,
    // start index for index.
    pub(super) index: usize,
    // Start index for strings.
    pub(super) strings: usize,
}
