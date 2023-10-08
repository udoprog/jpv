use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use anyhow::Result;
use musli::{Decode, Encode};

use crate::database::Id;
use crate::PartOfSpeech;

pub(super) struct Pair<'a> {
    prefix: &'a [u8],
    suffix: &'a [u8],
}

impl<'a> Pair<'a> {
    pub(super) fn new(prefix: &'a [u8], suffix: &'a [u8]) -> Self {
        Self { prefix, suffix }
    }

    fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.prefix.iter().chain(self.suffix).copied()
    }
}

impl PartialEq for Pair<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl Eq for Pair<'_> {}

impl Hash for Pair<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for b in self.iter() {
            b.hash(state);
        }
    }
}

#[derive(Default)]
pub(super) struct Index<'a> {
    pub(super) lookup: HashMap<Pair<'a>, Vec<Id>>,
    pub(super) by_pos: HashMap<PartOfSpeech, HashSet<u32>>,
    pub(super) by_sequence: HashMap<u64, u32>,
}

/// How the index is stored.
#[derive(Default, Encode, Decode)]
#[musli(packed)]
pub(super) struct Data {
    pub(super) lookup: HashMap<(u32, u32), Vec<Id>>,
    pub(super) by_pos: HashMap<PartOfSpeech, HashSet<u32>>,
    pub(super) by_sequence: HashMap<u64, u32>,
}
