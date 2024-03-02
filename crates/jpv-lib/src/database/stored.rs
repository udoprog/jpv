use musli_zerocopy::endian::Little;
use musli_zerocopy::{slice, swiss, trie, Ref, ZeroCopy};

use crate::PartOfSpeech;

use super::{InflectionData, KanjiIndex, NameIndex, PhraseIndex};

pub struct CompactTrie;

impl trie::Flavor for CompactTrie {
    type String = slice::Packed<[u8], u32, u8>;
    type Values<T> = slice::Packed<[T], u32, u16> where T: ZeroCopy;
    type Children<T> = slice::Packed<[T], u32, u16> where T: ZeroCopy;
}

#[derive(ZeroCopy)]
#[repr(C)]
pub(super) struct GlobalHeader {
    pub(super) magic: u32,
    pub(super) version: u32,
    pub(super) index: Ref<IndexHeader, Little>,
}

#[derive(Clone, Copy, ZeroCopy)]
#[repr(C)]
pub(super) struct IndexHeader {
    pub(super) name: Ref<str>,
    pub(super) lookup: trie::TrieRef<Id, CompactTrie>,
    pub(super) by_pos: swiss::MapRef<PartOfSpeech, Ref<[PhrasePos]>>,
    pub(super) by_kanji_literal: swiss::MapRef<Ref<str>, u32>,
    pub(super) radicals: swiss::MapRef<Ref<str>, u32>,
    pub(super) radicals_to_kanji: swiss::MapRef<Ref<str>, Ref<[u32]>>,
    pub(super) by_sequence: swiss::MapRef<u32, PhrasePos>,
    pub(super) inflections: Ref<[InflectionData]>,
}

/// Extra information about an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(u8)]
pub(super) enum Source {
    /// Indexed due to a kanji.
    Kanji { index: KanjiIndex },
    /// Indexed due to a phrase.
    Phrase { index: PhraseIndex },
    /// Indexed due to an inflection. The exact kind of inflection is identifier
    /// by its index, which fits into a u16.
    Inflection { inflection: u16 },
    /// Indexed to to a name.
    Name { index: NameIndex },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(C)]
pub(super) struct PhrasePos {
    pub(super) offset: u32,
    pub(super) reading: PhraseIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(C)]
pub(super) struct Id {
    pub(super) offset: u32,
    pub(super) source: Source,
}

impl Id {
    pub(super) fn phrase(offset: u32, index: PhraseIndex) -> Self {
        Self {
            offset,
            source: Source::Phrase { index },
        }
    }

    pub(super) fn name(offset: u32, index: NameIndex) -> Self {
        Self {
            offset,
            source: Source::Name { index },
        }
    }

    pub(super) fn kanji(offset: u32, index: KanjiIndex) -> Self {
        Self {
            offset,
            source: Source::Kanji { index },
        }
    }

    pub(super) fn inflection(offset: u32, inflection: u16) -> Self {
        Self {
            offset,
            source: Source::Inflection { inflection },
        }
    }
}
