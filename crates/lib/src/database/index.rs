use musli_zerocopy::pointer::{Slice, Unsized};
use musli_zerocopy::swiss::MapRef;
use musli_zerocopy::ZeroCopy;

use crate::PartOfSpeech;

use super::Id;

#[derive(ZeroCopy)]
#[repr(C)]
pub(super) struct Index {
    pub(super) lookup: MapRef<Unsized<str>, Slice<Id>>,
    pub(super) by_pos: MapRef<PartOfSpeech, Slice<Slice<u8>>>,
    pub(super) by_sequence: MapRef<u64, Slice<u8>>,
}
