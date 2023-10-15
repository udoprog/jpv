use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use musli_zerocopy::pointer::{Ref, Slice, Unsized};
use musli_zerocopy::swiss::MapRef;
use musli_zerocopy::ZeroCopy;

use crate::database::Id;
use crate::PartOfSpeech;

#[derive(ZeroCopy)]
#[repr(C)]
pub(super) struct Index {
    pub(super) lookup: MapRef<Unsized<str>, Slice<Id>>,
    pub(super) by_pos: MapRef<PartOfSpeech, Slice<Ref<Slice<u8>>>>,
    pub(super) by_sequence: MapRef<u64, Ref<Slice<u8>>>,
}

/// How the index is stored.
#[derive(Default)]
pub(super) struct Data<'a> {
    pub(super) lookup: HashMap<Cow<'a, str>, Vec<Id>>,
    pub(super) by_pos: HashMap<PartOfSpeech, HashSet<Ref<Slice<u8>>>>,
    pub(super) by_sequence: HashMap<u64, Ref<Slice<u8>>>,
}
