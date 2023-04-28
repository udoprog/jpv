use std::collections::{HashMap, HashSet};

use anyhow::Result;
use musli::{Decode, Encode};

use crate::database::Id;
use crate::PartOfSpeech;

#[derive(Default)]
pub(super) struct Index {
    pub(super) lookup: HashMap<Vec<u8>, Vec<Id>>,
    pub(super) by_pos: HashMap<PartOfSpeech, HashSet<usize>>,
    pub(super) by_sequence: HashMap<u64, usize>,
}

/// How the index is stored.
#[derive(Default, Encode, Decode)]
#[musli(packed)]
pub(super) struct Data {
    pub(super) lookup: HashMap<(usize, usize), Vec<Id>>,
    pub(super) by_pos: HashMap<PartOfSpeech, HashSet<usize>>,
    pub(super) by_sequence: HashMap<u64, usize>,
}
